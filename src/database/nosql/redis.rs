use async_trait::async_trait;
use redis::{Client, aio::Connection, AsyncCommands, RedisResult};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct RedisConnection {
    client: Client,
    // Each Redis connection is not thread-safe, so we use a mutex for async access
    conn: Arc<Mutex<Connection>>,
    db_name: String,
}

impl RedisConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = if let Some(conn_str) = &config.connection_string {
            conn_str.clone()
        } else {
            // Build connection string from components
            let host = config.host.as_deref().unwrap_or("localhost");
            let port = config.port.unwrap_or(6379);
            let username = config.username.as_deref();
            let password = config.password.as_deref();
            
            let mut url = format!("redis://{}:{}", host, port);
            
            if let (Some(u), Some(p)) = (username, password) {
                url = format!("redis://{}:{}@{}:{}", u, p, host, port);
            }
            
            url
        };

        let client = Client::open(connection_string.clone())
            .map_err(|e| Error::DatabaseConnectionError(format!("Redis connection error: {}", e)))?;
            
        let mut conn = client.get_async_connection().await
            .map_err(|e| Error::DatabaseConnectionError(format!("Redis async connection error: {}", e)))?;
            
        // Set a timeout if configured
        if let Some(timeout) = config.timeout_seconds {
            redis::cmd("CONFIG")
                .arg("SET")
                .arg("timeout")
                .arg(timeout.to_string())
                .query_async(&mut conn)
                .await
                .map_err(|e| Error::DatabaseConnectionError(format!("Redis timeout setting error: {}", e)))?;
        }
        
        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to generate a key for a record in a collection
    fn generate_key(&self, collection: &str, id: &str) -> String {
        format!("{}:{}", collection, id)
    }
    
    // Helper method to extract collection and id from a key
    fn extract_collection_id(&self, key: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = key.splitn(2, ':').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
}

#[async_trait]
impl DatabaseOperation for RedisConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        let mut conn = self.conn.lock().await;
        
        let key = self.generate_key(collection, &record.id);
        let value = serde_json::to_string(&record.data)
            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON data: {}", e)))?;
        
        // Store the record
        let _: () = conn.set(&key, value).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis set error: {}", e)))?;
            
        // Add the key to a set that tracks all keys for this collection
        let collection_set = format!("{}:keys", collection);
        let _: () = conn.sadd(&collection_set, &key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis sadd error: {}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        let mut conn = self.conn.lock().await;
        
        let key = self.generate_key(collection, id);
        
        // Check if the key exists
        let exists: bool = conn.exists(&key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis exists error: {}", e)))?;
            
        if !exists {
            return Ok(None);
        }
        
        // Get the value
        let value: String = conn.get(&key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis get error: {}", e)))?;
            
        // Parse the value as JSON
        let data = serde_json::from_str(&value)
            .map_err(|e| Error::DataConversionError(format!("Failed to parse JSON data: {}", e)))?;
            
        Ok(Some(Record {
            id: id.to_string(),
            data,
        }))
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        let mut conn = self.conn.lock().await;
        
        let key = self.generate_key(collection, &record.id);
        
        // Check if the key exists
        let exists: bool = conn.exists(&key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis exists error: {}", e)))?;
            
        if !exists {
            return Ok(false);
        }
        
        // Update the value
        let value = serde_json::to_string(&record.data)
            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON data: {}", e)))?;
            
        let _: () = conn.set(&key, value).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis set error: {}", e)))?;
            
        Ok(true)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        let mut conn = self.conn.lock().await;
        
        let key = self.generate_key(collection, id);
        
        // Check if the key exists
        let exists: bool = conn.exists(&key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis exists error: {}", e)))?;
            
        if !exists {
            return Ok(false);
        }
        
        // Delete the key
        let deleted: i32 = conn.del(&key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis del error: {}", e)))?;
            
        // Remove the key from the collection set
        let collection_set = format!("{}:keys", collection);
        let _: () = conn.srem(&collection_set, &key).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis srem error: {}", e)))?;
            
        Ok(deleted > 0)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        let mut conn = self.conn.lock().await;
        
        // Redis doesn't support complex queries natively, so we'll implement a simple filtering mechanism
        
        // Get all keys for the collection
        let collection_set = format!("{}:keys", collection);
        
        // Check if the collection set exists
        let exists: bool = conn.exists(&collection_set).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis exists error: {}", e)))?;
            
        if !exists {
            return Ok(Vec::new());
        }
        
        // Get all keys for the collection
        let keys: Vec<String> = conn.smembers(&collection_set).await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis smembers error: {}", e)))?;
            
        // Fetch all values
        let mut records = Vec::new();
        
        for key in keys {
            if let Some((_, id)) = self.extract_collection_id(&key) {
                // Get the value
                let value: RedisResult<String> = conn.get(&key).await;
                
                if let Ok(value) = value {
                    // Parse the value as JSON
                    match serde_json::from_str(&value) {
                        Ok(data) => {
                            let record = Record {
                                id,
                                data,
                            };
                            
                            // Filter records based on the query
                            if !params.query.is_null() && !params.query.as_object().map_or(true, |obj| obj.is_empty()) {
                                let mut include = true;
                                
                                if let (Some(query_obj), Some(data_obj)) = (params.query.as_object(), record.data.as_object()) {
                                    for (key, value) in query_obj {
                                        if let Some(data_value) = data_obj.get(key) {
                                            if data_value != value {
                                                include = false;
                                                break;
                                            }
                                        } else {
                                            include = false;
                                            break;
                                        }
                                    }
                                }
                                
                                if include {
                                    records.push(record);
                                }
                            } else {
                                // No filtering, include all records
                                records.push(record);
                            }
                        },
                        Err(_) => {
                            // Skip records that can't be parsed as JSON
                            continue;
                        }
                    }
                }
            }
        }
        
        // Handle sorting
        if let Some(sort_by) = &params.sort_by {
            records.sort_by(|a, b| {
                let a_val = a.data.get(sort_by);
                let b_val = b.data.get(sort_by);
                
                match (a_val, b_val) {
                    (Some(a_val), Some(b_val)) => {
                        if params.sort_order.as_deref() == Some("desc") {
                            b_val.partial_cmp(a_val).unwrap_or(std::cmp::Ordering::Equal)
                        } else {
                            a_val.partial_cmp(b_val).unwrap_or(std::cmp::Ordering::Equal)
                        }
                    },
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            });
        }
        
        // Handle pagination
        let mut start = params.offset.unwrap_or(0) as usize;
        let end = params.limit.map(|limit| start + limit as usize);
        
        if start > records.len() {
            start = records.len();
        }
        
        let records = if let Some(end) = end {
            if end < start {
                Vec::new()
            } else {
                records.into_iter().skip(start).take(end - start).collect()
            }
        } else {
            records.into_iter().skip(start).collect()
        };
        
        Ok(records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        let mut conn = self.conn.lock().await;
        
        // Start a Redis transaction
        let mut pipe = redis::pipe();
        pipe.atomic();
        
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    let key = self.generate_key(&collection, &record.id);
                    let value = serde_json::to_string(&record.data)
                        .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON data: {}", e)))?;
                    
                    // Store the record
                    pipe.cmd("SET").arg(&key).arg(value);
                    
                    // Add the key to a set that tracks all keys for this collection
                    let collection_set = format!("{}:keys", collection);
                    pipe.cmd("SADD").arg(&collection_set).arg(&key);
                },
                TransactionOperation::Update { collection, record } => {
                    let key = self.generate_key(&collection, &record.id);
                    let value = serde_json::to_string(&record.data)
                        .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON data: {}", e)))?;
                    
                    // Update the value
                    pipe.cmd("SET").arg(&key).arg(value);
                },
                TransactionOperation::Delete { collection, id } => {
                    let key = self.generate_key(&collection, &id);
                    
                    // Delete the key
                    pipe.cmd("DEL").arg(&key);
                    
                    // Remove the key from the collection set
                    let collection_set = format!("{}:keys", collection);
                    pipe.cmd("SREM").arg(&collection_set).arg(&key);
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        // Execute the transaction
        pipe.query_async(&mut *conn).await
            .map_err(|e| Error::DatabaseTransactionError(format!("Redis transaction error: {}", e)))?;
            
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let mut conn = self.conn.lock().await;
        
        // Get all keys and extract collection names
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("*:keys")
            .query_async(&mut *conn)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("Redis keys error: {}", e)))?;
            
        // Extract collection names from keys
        let collections = keys.into_iter()
            .filter_map(|key| {
                let parts: Vec<&str> = key.splitn(2, ':').collect();
                if parts.len() == 2 && parts[1] == "keys" {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect();
            
        Ok(DatabaseInfo {
            db_type: "redis".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for RedisConnection {
    fn get_db_type(&self) -> &str {
        "redis"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        let mut conn = self.conn.lock().await;
        
        let ping: String = redis::cmd("PING").query_async(&mut *conn).await
            .map_err(|e| Error::DatabaseConnectionError(format!("Redis health check failed: {}", e)))?;
            
        if ping != "PONG" {
            return Err(Error::DatabaseConnectionError("Redis health check failed: unexpected response".to_string()));
        }
        
        Ok(())
    }
}
