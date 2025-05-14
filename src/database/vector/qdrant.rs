use async_trait::async_trait;
use qdrant_client::{
    client::{QdrantClient, QdrantClientConfig},
    qdrant::{
        vectors_config, CollectionOperationResponse, CreateCollection, Distance, PointStruct,
        SearchPoints, VectorParams, VectorsConfig, WithPayloadSelector, WithVectorsSelector,
        PointId, ScoredPoint, Filter, FieldCondition, Match, Value, Condition,
        DeletePoints, PointsSelector, SetPayloadPoints, GetPoints, Payload,
    },
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct QdrantConnection {
    client: QdrantClient,
    dimension: usize,
    db_name: String,
}

impl QdrantConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let url = if let Some(conn_str) = &config.connection_string {
            conn_str.clone()
        } else {
            // Build connection string from components
            let host = config.host.as_deref().unwrap_or("localhost");
            let port = config.port.unwrap_or(6333);
            
            format!("http://{}:{}", host, port)
        };

        let timeout = config.timeout_seconds.unwrap_or(30);
        
        let mut client_config = QdrantClientConfig::from_url(&url);
        client_config.timeout = Some(Duration::from_secs(timeout));
        
        let client = QdrantClient::new(Some(client_config))
            .map_err(|e| Error::DatabaseConnectionError(format!("Qdrant connection error: {}", e)))?;
            
        // Vector dimension is a required parameter for Qdrant collections
        // Here we use a default value of 768 (common for embedding models)
        let dimension = 768;
        
        Ok(Self {
            client,
            dimension,
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to ensure a collection exists
    async fn ensure_collection_exists(&self, collection: &str) -> Result<()> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == collection);
            
        if !collection_exists {
            // Create the collection
            let create_collection = CreateCollection {
                collection_name: collection.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(vectors_config::Config::Params(VectorParams {
                        size: self.dimension as u64,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                    })),
                }),
                replication_factor: None,
                write_consistency_factor: None,
                on_disk_payload: Some(true),
                hnsw_config: None,
                wal_config: None,
                optimizers_config: None,
                shard_number: None,
                on_disk: None,
                init_from: None,
                quantization_config: None,
                sharding_method: None,
                sparse_vectors_config: None,
            };
            
            self.client.create_collection(&create_collection).await
                .map_err(|e| Error::DatabaseQueryError(format!("Qdrant create collection error: {}", e)))?;
        }
        
        Ok(())
    }
    
    // Helper method to convert a record to Qdrant point
    fn record_to_point(&self, record: &Record) -> Result<PointStruct> {
        // Extract vector from record data
        let vector = match record.data.get("vector") {
            Some(v) => match v.as_array() {
                Some(arr) => {
                    let mut vector = Vec::with_capacity(arr.len());
                    for val in arr {
                        match val.as_f64() {
                            Some(f) => vector.push(f as f32),
                            None => return Err(Error::DataConversionError("Vector values must be numbers".to_string())),
                        }
                    }
                    vector
                },
                None => return Err(Error::DataConversionError("Vector must be an array".to_string())),
            },
            None => return Err(Error::DataConversionError("Record must contain a 'vector' field".to_string())),
        };
        
        // Create a payload from record data (excluding vector)
        let mut payload = Payload::new();
        
        if let Some(obj) = record.data.as_object() {
            for (key, value) in obj {
                if key == "vector" {
                    continue; // Skip vector field
                }
                
                let qdrant_value = match value {
                    serde_json::Value::Null => continue, // Skip null values
                    serde_json::Value::Bool(b) => Value { kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(*b)) },
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Value { kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) }
                        } else if let Some(f) = n.as_f64() {
                            Value { kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) }
                        } else {
                            continue;
                        }
                    },
                    serde_json::Value::String(s) => Value { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s.clone())) },
                    _ => {
                        // For complex types, convert to JSON string
                        let json_str = serde_json::to_string(value)
                            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON value: {}", e)))?;
                            
                        Value { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(json_str)) }
                    },
                };
                
                payload.insert(key.clone(), qdrant_value);
            }
        }
        
        // Parse the ID
        let id = match Uuid::parse_str(&record.id) {
            Ok(uuid) => PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) },
            Err(_) => {
                // Try as numeric ID
                if let Ok(num_id) = record.id.parse::<u64>() {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num_id)) }
                } else {
                    // Use string as UUID by creating a new UUID
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, record.id.as_bytes());
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) }
                }
            }
        };
        
        Ok(PointStruct {
            id: Some(id),
            vectors: Some(qdrant_client::qdrant::Vectors { 
                vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                    qdrant_client::qdrant::Vector { data: vector }
                ))
            }),
            payload,
        })
    }
    
    // Helper method to convert Qdrant point to record
    fn point_to_record(&self, point: ScoredPoint) -> Result<Record> {
        // Extract ID
        let id = match point.id {
            Some(id) => match id.point_id_options {
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(bytes)) => {
                    if bytes.len() == 16 {
                        let mut uuid_bytes = [0u8; 16];
                        uuid_bytes.copy_from_slice(&bytes[0..16]);
                        Uuid::from_bytes(uuid_bytes).to_string()
                    } else {
                        return Err(Error::DataConversionError("Invalid UUID bytes".to_string()));
                    }
                },
                Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => num.to_string(),
                None => return Err(Error::DataConversionError("Missing point ID".to_string())),
            },
            None => return Err(Error::DataConversionError("Missing point ID".to_string())),
        };
        
        // Create data object
        let mut data_map = serde_json::Map::new();
        
        // Add vector if available
        if let Some(vectors) = point.vectors {
            match vectors.vectors_options {
                Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(v)) => {
                    let vector_array = serde_json::Value::Array(
                        v.data.iter().map(|&f| serde_json::Value::Number(serde_json::Number::from_f64(f as f64).unwrap())).collect()
                    );
                    data_map.insert("vector".to_string(), vector_array);
                },
                _ => {},
            }
        }
        
        // Add score
        data_map.insert("score".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(point.score).unwrap_or(serde_json::Number::from(0))
        ));
        
        // Add payload fields
        if let Some(payload) = point.payload {
            for (key, value) in payload {
                let json_value = match value.kind {
                    Some(qdrant_client::qdrant::value::Kind::NullValue(_)) => serde_json::Value::Null,
                    Some(qdrant_client::qdrant::value::Kind::BoolValue(b)) => serde_json::Value::Bool(b),
                    Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) => serde_json::Value::Number(serde_json::Number::from(i)),
                    Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) => {
                        if let Some(n) = serde_json::Number::from_f64(f) {
                            serde_json::Value::Number(n)
                        } else {
                            serde_json::Value::Null
                        }
                    },
                    Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => {
                        // Try to parse as JSON
                        match serde_json::from_str(&s) {
                            Ok(v) => v,
                            Err(_) => serde_json::Value::String(s),
                        }
                    },
                    Some(qdrant_client::qdrant::value::Kind::StructValue(_)) |
                    Some(qdrant_client::qdrant::value::Kind::ListValue(_)) => {
                        // These are complex types, which we'd need to convert
                        // For simplicity, we'll skip them for now
                        continue;
                    },
                    None => continue,
                };
                
                data_map.insert(key, json_value);
            }
        }
        
        Ok(Record {
            id,
            data: serde_json::Value::Object(data_map),
        })
    }
    
    // Helper method to build a query filter from parameters
    fn build_filter(&self, params: &QueryParams) -> Option<Filter> {
        if params.query.is_null() || !params.query.is_object() {
            return None;
        }
        
        let obj = params.query.as_object().unwrap();
        if obj.is_empty() {
            return None;
        }
        
        let mut conditions = Vec::new();
        
        for (key, value) in obj {
            let condition = match value {
                serde_json::Value::Null => {
                    // Skip null values for now (Qdrant doesn't have a direct null check)
                    continue;
                },
                serde_json::Value::Bool(b) => {
                    FieldCondition {
                        key: key.clone(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Boolean(*b)),
                        }),
                        range: None,
                        geo_bounding_box: None,
                        geo_radius: None,
                        geo_polygon: None,
                        values_count: None,
                    }
                },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        FieldCondition {
                            key: key.clone(),
                            r#match: Some(Match {
                                match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Integer(i)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            geo_polygon: None,
                            values_count: None,
                        }
                    } else if let Some(f) = n.as_f64() {
                        FieldCondition {
                            key: key.clone(),
                            r#match: Some(Match {
                                match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Double(f)),
                            }),
                            range: None,
                            geo_bounding_box: None,
                            geo_radius: None,
                            geo_polygon: None,
                            values_count: None,
                        }
                    } else {
                        continue;
                    }
                },
                serde_json::Value::String(s) => {
                    FieldCondition {
                        key: key.clone(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(s.clone())),
                        }),
                        range: None,
                        geo_bounding_box: None,
                        geo_radius: None,
                        geo_polygon: None,
                        values_count: None,
                    }
                },
                _ => {
                    // For complex types, we skip for now
                    continue;
                },
            };
            
            conditions.push(Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(condition)),
            });
        }
        
        if conditions.is_empty() {
            return None;
        }
        
        Some(Filter {
            must: conditions,
            must_not: Vec::new(),
            should: Vec::new(),
            min_should: None,
        })
    }
}

#[async_trait]
impl DatabaseOperation for QdrantConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        // Ensure the collection exists
        self.ensure_collection_exists(collection).await?;
        
        // Convert record to Qdrant point
        let point = self.record_to_point(record)?;
        
        // Insert the point
        self.client.upsert_points(collection, None, vec![point], None).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant upsert error: {}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == collection);
            
        if !collection_exists {
            return Ok(None);
        }
        
        // Parse the ID
        let point_id = match Uuid::parse_str(id) {
            Ok(uuid) => PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) },
            Err(_) => {
                // Try as numeric ID
                if let Ok(num_id) = id.parse::<u64>() {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num_id)) }
                } else {
                    // Use string as UUID by creating a new UUID
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes());
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) }
                }
            }
        };
        
        // Create request to get the point
        let get_points_request = GetPoints {
            collection_name: collection.to_string(),
            ids: vec![point_id],
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(WithVectorsSelector {
                selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(true)),
            }),
            read_consistency: None,
        };
        
        // Execute the query
        let response = self.client.get_points(&get_points_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant get point error: {}", e)))?;
            
        if response.points.is_empty() {
            return Ok(None);
        }
        
        // Convert the point to a record
        let point = &response.points[0];
        let record = self.point_to_record(ScoredPoint {
            id: point.id.clone(),
            payload: point.payload.clone(),
            score: 0.0, // No score for direct retrieval
            vectors: point.vectors.clone(),
            shard_key: None,
            version: None,
        })?;
        
        Ok(Some(record))
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == collection);
            
        if !collection_exists {
            return Ok(false);
        }
        
        // Parse the ID
        let point_id = match Uuid::parse_str(&record.id) {
            Ok(uuid) => PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) },
            Err(_) => {
                // Try as numeric ID
                if let Ok(num_id) = record.id.parse::<u64>() {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num_id)) }
                } else {
                    // Use string as UUID by creating a new UUID
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, record.id.as_bytes());
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) }
                }
            }
        };
        
        // Check if the point exists
        let get_points_request = GetPoints {
            collection_name: collection.to_string(),
            ids: vec![point_id.clone()],
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(false)),
            }),
            with_vectors: Some(WithVectorsSelector {
                selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(false)),
            }),
            read_consistency: None,
        };
        
        let response = self.client.get_points(&get_points_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant get point error: {}", e)))?;
            
        if response.points.is_empty() {
            return Ok(false);
        }
        
        // If vector is changing, we need to use upsert
        if record.data.get("vector").is_some() {
            let point = self.record_to_point(record)?;
            
            self.client.upsert_points(collection, None, vec![point], None).await
                .map_err(|e| Error::DatabaseQueryError(format!("Qdrant upsert error: {}", e)))?;
                
            return Ok(true);
        }
        
        // If only payload is changing, we can use set_payload
        let mut payload = Payload::new();
        
        if let Some(obj) = record.data.as_object() {
            for (key, value) in obj {
                if key == "vector" {
                    continue; // Skip vector field
                }
                
                let qdrant_value = match value {
                    serde_json::Value::Null => continue, // Skip null values
                    serde_json::Value::Bool(b) => Value { kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(*b)) },
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Value { kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) }
                        } else if let Some(f) = n.as_f64() {
                            Value { kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) }
                        } else {
                            continue;
                        }
                    },
                    serde_json::Value::String(s) => Value { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(s.clone())) },
                    _ => {
                        // For complex types, convert to JSON string
                        let json_str = serde_json::to_string(value)
                            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON value: {}", e)))?;
                            
                        Value { kind: Some(qdrant_client::qdrant::value::Kind::StringValue(json_str)) }
                    },
                };
                
                payload.insert(key.clone(), qdrant_value);
            }
        }
        
        let set_payload_request = SetPayloadPoints {
            collection_name: collection.to_string(),
            payload,
            points_selector: Some(PointsSelector {
                points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                    qdrant_client::qdrant::PointsIdsList {
                        ids: vec![point_id],
                    }
                )),
            }),
            wait_until_indexed: None,
        };
        
        self.client.set_payload_points(&set_payload_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant set payload error: {}", e)))?;
            
        Ok(true)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == collection);
            
        if !collection_exists {
            return Ok(false);
        }
        
        // Parse the ID
        let point_id = match Uuid::parse_str(id) {
            Ok(uuid) => PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) },
            Err(_) => {
                // Try as numeric ID
                if let Ok(num_id) = id.parse::<u64>() {
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num_id)) }
                } else {
                    // Use string as UUID by creating a new UUID
                    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes());
                    PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid.as_bytes().to_vec())) }
                }
            }
        };
        
        // Check if the point exists
        let get_points_request = GetPoints {
            collection_name: collection.to_string(),
            ids: vec![point_id.clone()],
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(false)),
            }),
            with_vectors: Some(WithVectorsSelector {
                selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(false)),
            }),
            read_consistency: None,
        };
        
        let response = self.client.get_points(&get_points_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant get point error: {}", e)))?;
            
        if response.points.is_empty() {
            return Ok(false);
        }
        
        // Delete the point
        let delete_request = DeletePoints {
            collection_name: collection.to_string(),
            points_selector: Some(PointsSelector {
                points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Points(
                    qdrant_client::qdrant::PointsIdsList {
                        ids: vec![point_id],
                    }
                )),
            }),
            wait_until_indexed: None,
        };
        
        self.client.delete_points(&delete_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant delete error: {}", e)))?;
            
        Ok(true)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        // Check if collection exists
        let collections = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collection_exists = collections.collections
            .iter()
            .any(|c| c.name == collection);
            
        if !collection_exists {
            return Ok(Vec::new());
        }
        
        // For vector databases, we need a vector query
        // Check if the query contains a vector
        let query_vector = match params.query.get("vector") {
            Some(v) => match v.as_array() {
                Some(arr) => {
                    let mut vector = Vec::with_capacity(arr.len());
                    for val in arr {
                        match val.as_f64() {
                            Some(f) => vector.push(f as f32),
                            None => return Err(Error::InvalidQueryError("Vector values must be numbers".to_string())),
                        }
                    }
                    vector
                },
                None => return Err(Error::InvalidQueryError("Vector must be an array".to_string())),
            },
            None => {
                // If no vector is provided, we'll do a filter-only search
                // which is not ideal for vector databases, but we'll support it
                
                // Build filter from other query parameters
                let filter = self.build_filter(params);
                
                // Use scroll API to get all matching points
                let mut offset = None;
                let limit = params.limit.unwrap_or(100) as u64;
                let mut all_points = Vec::new();
                
                loop {
                    let scroll_request = qdrant_client::qdrant::ScrollPoints {
                        collection_name: collection.to_string(),
                        filter: filter.clone(),
                        limit,
                        offset,
                        with_payload: Some(WithPayloadSelector {
                            selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
                        }),
                        with_vectors: Some(WithVectorsSelector {
                            selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(true)),
                        }),
                        read_consistency: None,
                    };
                    
                    let response = self.client.scroll_points(&scroll_request).await
                        .map_err(|e| Error::DatabaseQueryError(format!("Qdrant scroll error: {}", e)))?;
                        
                    if response.points.is_empty() {
                        break;
                    }
                    
                    all_points.extend(response.points.clone());
                    
                    // Update offset for next page
                    offset = response.next_page_offset;
                    
                    // If no next page or we've reached the requested limit, break
                    if offset.is_none() || (params.limit.is_some() && all_points.len() >= params.limit.unwrap() as usize) {
                        break;
                    }
                }
                
                // Convert points to records
                let mut records = Vec::new();
                
                for point in all_points {
                    let scored_point = ScoredPoint {
                        id: point.id,
                        payload: point.payload,
                        score: 0.0, // No score for scroll queries
                        vectors: point.vectors,
                        shard_key: None,
                        version: None,
                    };
                    
                    let record = self.point_to_record(scored_point)?;
                    records.push(record);
                }
                
                return Ok(records);
            },
        };
        
        // Build filter from other query parameters
        let mut query_params_copy = params.query.clone();
        if let Some(obj) = query_params_copy.as_object_mut() {
            obj.remove("vector"); // Remove vector from filter params
        }
        
        let filter_params = QueryParams {
            query: query_params_copy,
            limit: params.limit,
            offset: params.offset,
            sort_by: params.sort_by.clone(),
            sort_order: params.sort_order.clone(),
        };
        
        let filter = self.build_filter(&filter_params);
        
        // Create search request
        let search_request = SearchPoints {
            collection_name: collection.to_string(),
            vector: query_vector,
            filter,
            limit: params.limit.unwrap_or(10) as u64,
            offset: params.offset.unwrap_or(0) as u64,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            with_vectors: Some(WithVectorsSelector {
                selector_options: Some(qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(true)),
            }),
            score_threshold: None,
            vector_name: None,
            read_consistency: None,
        };
        
        // Execute the search
        let response = self.client.search_points(&search_request).await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant search error: {}", e)))?;
            
        // Convert results to records
        let mut records = Vec::new();
        
        for point in response.result {
            let record = self.point_to_record(point)?;
            records.push(record);
        }
        
        Ok(records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        // Qdrant doesn't support transactions, so we'll execute operations sequentially
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    self.create(&collection, &record).await?;
                },
                TransactionOperation::Update { collection, record } => {
                    self.update(&collection, &record).await?;
                },
                TransactionOperation::Delete { collection, id } => {
                    self.delete(&collection, &id).await?;
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        // Get list of collections
        let collections_response = self.client.list_collections().await
            .map_err(|e| Error::DatabaseQueryError(format!("Qdrant list collections error: {}", e)))?;
            
        let collections = collections_response.collections
            .iter()
            .map(|c| c.name.clone())
            .collect();
            
        Ok(DatabaseInfo {
            db_type: "qdrant".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for QdrantConnection {
    fn get_db_type(&self) -> &str {
        "qdrant"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        // Just list collections to check if connection is alive
        self.client.list_collections().await
            .map_err(|e| Error::DatabaseConnectionError(format!("Qdrant health check failed: {}", e)))?;
            
        Ok(())
    }
}
