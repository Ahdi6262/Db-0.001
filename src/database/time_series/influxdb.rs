use async_trait::async_trait;
use influxdb::{Client, InfluxDbWriteable, ReadQuery, Timestamp, TimestampOptions, WriteQuery};
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct InfluxDbConnection {
    client: Client,
    db_name: String,
}

impl InfluxDbConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let url = if let Some(conn_str) = &config.connection_string {
            conn_str.clone()
        } else {
            // Build connection string from components
            let host = config.host.as_deref().unwrap_or("localhost");
            let port = config.port.unwrap_or(8086);
            
            format!("http://{}:{}", host, port)
        };

        let username = config.username.clone();
        let password = config.password.clone();
        
        let mut client = Client::new(url, &config.name);
        
        if let (Some(username), Some(password)) = (username, password) {
            client = client.with_auth(username, password);
        }
        
        // Set a timeout if configured
        if let Some(timeout) = config.timeout_seconds {
            client = client.with_timeout(Duration::from_secs(timeout));
        }
        
        // Test the connection
        client.ping().await
            .map_err(|e| Error::DatabaseConnectionError(format!("InfluxDB connection error: {}", e)))?;
            
        Ok(Self {
            client,
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to convert a record to InfluxDB point
    fn record_to_point(&self, collection: &str, record: &Record) -> Result<WriteQuery> {
        let timestamp = match record.data.get("timestamp") {
            Some(ts) => {
                match ts {
                    serde_json::Value::String(ts_str) => {
                        match ts_str.parse::<DateTime<Utc>>() {
                            Ok(dt) => dt.timestamp_nanos() as u128,
                            Err(_) => Utc::now().timestamp_nanos() as u128,
                        }
                    },
                    serde_json::Value::Number(n) => {
                        if let Some(ts) = n.as_u64() {
                            ts as u128
                        } else {
                            Utc::now().timestamp_nanos() as u128
                        }
                    },
                    _ => Utc::now().timestamp_nanos() as u128,
                }
            },
            None => Utc::now().timestamp_nanos() as u128,
        };
        
        let mut fields = HashMap::new();
        let mut tags = HashMap::new();
        
        // Add id as a tag
        tags.insert("id".to_string(), record.id.clone());
        
        if let Some(obj) = record.data.as_object() {
            for (key, value) in obj {
                if key == "timestamp" {
                    continue; // Skip timestamp as it's handled separately
                }
                
                match value {
                    serde_json::Value::Null => {
                        // Skip null values
                    },
                    serde_json::Value::Bool(b) => {
                        fields.insert(key.clone(), *b as i64);
                    },
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            fields.insert(key.clone(), i);
                        } else if let Some(f) = n.as_f64() {
                            fields.insert(key.clone(), f);
                        }
                    },
                    serde_json::Value::String(s) => {
                        tags.insert(key.clone(), s.clone());
                    },
                    _ => {
                        // Store complex values as string
                        let json_str = serde_json::to_string(value)
                            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON value: {}", e)))?;
                            
                        tags.insert(key.clone(), json_str);
                    },
                }
            }
        }
        
        // InfluxDB requires at least one field, so add a default if none was provided
        if fields.is_empty() {
            fields.insert("value".to_string(), 1i64);
        }
        
        // Create a query builder
        let mut query = WriteQuery::new(Timestamp::Nanoseconds(timestamp), collection);
        
        // Add tags
        for (key, value) in tags {
            query = query.add_tag(key, value);
        }
        
        // Add fields
        for (key, value) in fields {
            match value {
                serde_json::Value::Bool(b) => {
                    query = query.add_field(key, b);
                },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query = query.add_field(key, i);
                    } else if let Some(f) = n.as_f64() {
                        query = query.add_field(key, f);
                    }
                },
                serde_json::Value::String(s) => {
                    query = query.add_field(key, s);
                },
                _ => {
                    // For other types, serialize to string
                    let json_str = serde_json::to_string(&value)
                        .map_err(|e| Error::DataConversionError(format!("Failed to serialize field value: {}", e)))?;
                        
                    query = query.add_field(key, json_str);
                },
            }
        }
        
        Ok(query)
    }
    
    // Helper function to convert InfluxDB data to a record
    fn influxdb_to_record(&self, series: &influxdb::Series) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        
        let columns = &series.columns;
        
        for values in &series.values {
            if values.len() != columns.len() {
                continue; // Skip if column/value counts don't match
            }
            
            let mut data_map = serde_json::Map::new();
            let mut id = String::new();
            
            for (i, column) in columns.iter().enumerate() {
                let value = &values[i];
                
                if column == "id" {
                    if let Some(val) = value.as_str() {
                        id = val.to_string();
                    }
                    continue;
                }
                
                if column == "time" {
                    if let Some(val) = value.as_str() {
                        data_map.insert("timestamp".to_string(), serde_json::Value::String(val.to_string()));
                    }
                    continue;
                }
                
                let json_value = match value {
                    serde_json::Value::Null => serde_json::Value::Null,
                    serde_json::Value::Bool(b) => serde_json::Value::Bool(*b),
                    serde_json::Value::Number(n) => serde_json::Value::Number(n.clone()),
                    serde_json::Value::String(s) => {
                        // Try to parse as a more specific type
                        if let Ok(num) = s.parse::<i64>() {
                            serde_json::Value::Number(serde_json::Number::from(num))
                        } else if let Ok(num) = s.parse::<f64>() {
                            if let Some(n) = serde_json::Number::from_f64(num) {
                                serde_json::Value::Number(n)
                            } else {
                                serde_json::Value::String(s.clone())
                            }
                        } else if s == "true" {
                            serde_json::Value::Bool(true)
                        } else if s == "false" {
                            serde_json::Value::Bool(false)
                        } else if s == "null" {
                            serde_json::Value::Null
                        } else {
                            // Try to parse as JSON
                            match serde_json::from_str(s) {
                                Ok(parsed) => parsed,
                                Err(_) => serde_json::Value::String(s.clone()),
                            }
                        }
                    },
                    serde_json::Value::Array(arr) => serde_json::Value::Array(arr.clone()),
                    serde_json::Value::Object(obj) => serde_json::Value::Object(obj.clone()),
                };
                
                data_map.insert(column.clone(), json_value);
            }
            
            // Skip records without id
            if id.is_empty() {
                continue;
            }
            
            records.push(Record {
                id,
                data: serde_json::Value::Object(data_map),
            });
        }
        
        Ok(records)
    }
}

#[async_trait]
impl DatabaseOperation for InfluxDbConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        let query = self.record_to_point(collection, record)?;
        
        self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB write error: {}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        let query = ReadQuery::new(format!("SELECT * FROM {} WHERE id = '{}' ORDER BY time DESC LIMIT 1", collection, id));
        
        let result = self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB read error: {}", e)))?;
            
        if result.is_empty() {
            return Ok(None);
        }
        
        let series = &result[0];
        let records = self.influxdb_to_record(series)?;
        
        if records.is_empty() {
            return Ok(None);
        }
        
        Ok(Some(records[0].clone()))
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        // InfluxDB doesn't support updating points directly, so we'll check if it exists
        // and append a new point with the updated data
        let query = ReadQuery::new(format!("SELECT * FROM {} WHERE id = '{}' LIMIT 1", collection, record.id));
        
        let result = self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB read error: {}", e)))?;
            
        if result.is_empty() {
            return Ok(false);
        }
        
        // Insert the updated record
        let write_query = self.record_to_point(collection, record)?;
        
        self.client.query(&write_query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB write error: {}", e)))?;
            
        Ok(true)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        // Check if the record exists
        let query = ReadQuery::new(format!("SELECT * FROM {} WHERE id = '{}' LIMIT 1", collection, id));
        
        let result = self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB read error: {}", e)))?;
            
        if result.is_empty() {
            return Ok(false);
        }
        
        // Delete the record (note: InfluxDB drop query needs time ranges)
        let delete_query = ReadQuery::new(format!("DELETE FROM {} WHERE id = '{}'", collection, id));
        
        self.client.query(&delete_query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB delete error: {}", e)))?;
            
        Ok(true)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        // Build query from parameters
        let mut conditions = vec![format!("\"_measurement\" = '{}'", collection)];
        
        if !params.query.is_null() && !params.query.as_object().map_or(true, |obj| obj.is_empty()) {
            if let Some(obj) = params.query.as_object() {
                for (key, value) in obj {
                    match value {
                        serde_json::Value::Null => {
                            // Skip null values
                        },
                        serde_json::Value::Bool(b) => {
                            conditions.push(format!("{} = {}", key, b));
                        },
                        serde_json::Value::Number(n) => {
                            conditions.push(format!("{} = {}", key, n));
                        },
                        serde_json::Value::String(s) => {
                            conditions.push(format!("{} = '{}'", key, s.replace("'", "''")));
                        },
                        _ => {
                            conditions.push(format!("{} = '{}'", key, value.to_string().replace("'", "''")));
                        },
                    }
                }
            }
        }
        
        let where_clause = conditions.join(" AND ");
        
        // Build order clause
        let order_clause = if let Some(sort_by) = &params.sort_by {
            let direction = params.sort_order.as_deref().unwrap_or("asc");
            format!("ORDER BY {} {}", sort_by, direction)
        } else {
            "ORDER BY time DESC".to_string()
        };
        
        // Build limit and offset
        let limit_clause = if let Some(limit) = params.limit {
            format!("LIMIT {}", limit)
        } else {
            "".to_string()
        };
        
        let offset_clause = if let Some(offset) = params.offset {
            format!("OFFSET {}", offset)
        } else {
            "".to_string()
        };
        
        // Build final query
        let query_str = format!(
            "SELECT * FROM {} WHERE {} {} {} {}",
            collection, where_clause, order_clause, limit_clause, offset_clause
        );
        
        let query = ReadQuery::new(query_str);
        
        let result = self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB query error: {}", e)))?;
            
        if result.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut all_records = Vec::new();
        
        for series in &result {
            let records = self.influxdb_to_record(series)?;
            all_records.extend(records);
        }
        
        Ok(all_records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        // InfluxDB doesn't support transactions, but we can batch write points
        let mut write_queries = Vec::new();
        
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    let query = self.record_to_point(&collection, &record)?;
                    write_queries.push(query);
                },
                TransactionOperation::Update { collection, record } => {
                    // For InfluxDB, update is the same as create (it adds a new point)
                    let query = self.record_to_point(&collection, &record)?;
                    write_queries.push(query);
                },
                TransactionOperation::Delete { collection, id } => {
                    // InfluxDB doesn't support deleting as part of a batch, 
                    // so we need to execute it separately
                    let delete_query = ReadQuery::new(format!("DELETE FROM {} WHERE id = '{}'", collection, id));
                    
                    self.client.query(&delete_query).await
                        .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB delete error: {}", e)))?;
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        // Execute batch write if needed
        if !write_queries.is_empty() {
            for query in write_queries {
                self.client.query(&query).await
                    .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB batch write error: {}", e)))?;
            }
        }
        
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        // Query to get measurements (collections)
        let query = ReadQuery::new("SHOW MEASUREMENTS");
        
        let result = self.client.query(&query).await
            .map_err(|e| Error::DatabaseQueryError(format!("InfluxDB show measurements error: {}", e)))?;
            
        let mut collections = Vec::new();
        
        if !result.is_empty() {
            let series = &result[0];
            
            for values in &series.values {
                if !values.is_empty() {
                    if let Some(name) = values[0].as_str() {
                        collections.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(DatabaseInfo {
            db_type: "influxdb".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for InfluxDbConnection {
    fn get_db_type(&self) -> &str {
        "influxdb"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        self.client.ping().await
            .map_err(|e| Error::DatabaseConnectionError(format!("InfluxDB health check failed: {}", e)))?;
            
        Ok(())
    }
}
