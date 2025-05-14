use async_trait::async_trait;
use cassandra_rs::{
    cluster::Cluster,
    consistency::Consistency,
    error::CassError,
    future::ResultFuture,
    session::Session,
    statement::Statement,
    query::Query,
    uuid::Uuid,
};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct CassandraConnection {
    session: Arc<Mutex<Session>>,
    keyspace: String,
    db_name: String,
}

impl CassandraConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let contact_points = if let Some(host) = &config.host {
            host.clone()
        } else {
            "127.0.0.1".to_string()
        };
        
        let port = config.port.unwrap_or(9042);
        
        // Create cluster config
        let mut cluster = Cluster::default();
        cluster.set_contact_points(contact_points.as_str())?;
        cluster.set_port(port as i32);
        
        // Set credentials if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            cluster.set_credentials(username, password);
        }
        
        // Build cluster and connect
        let session = cluster
            .connect()
            .map_err(|e| Error::DatabaseConnectionError(format!("Cassandra connection error: {:?}", e)))?;
            
        // Use keyspace/database name
        let keyspace = config.name.clone();
        
        // Create keyspace if it doesn't exist
        let create_keyspace_query = format!(
            "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}",
            keyspace
        );
        
        let create_keyspace_stmt = Statement::new(create_keyspace_query.as_str());
        
        session
            .execute(&create_keyspace_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra keyspace creation error: {:?}", e)))?;
            
        // Use the keyspace
        let use_keyspace_query = format!("USE {}", keyspace);
        let use_keyspace_stmt = Statement::new(use_keyspace_query.as_str());
        
        session
            .execute(&use_keyspace_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra keyspace selection error: {:?}", e)))?;
            
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
            keyspace,
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to ensure a table exists for a collection
    async fn ensure_table_exists(&self, collection: &str) -> Result<()> {
        let mut session = self.session.lock().await;
        
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (id text PRIMARY KEY, data text)",
            collection
        );
        
        let create_table_stmt = Statement::new(create_table_query.as_str());
        
        session
            .execute(&create_table_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra table creation error: {:?}", e)))?;
            
        Ok(())
    }
    
    // Helper method to convert a JSON record to Cassandra data
    fn record_to_cassandra(&self, record: &Record) -> Result<(String, String)> {
        let id = record.id.clone();
        let data = serde_json::to_string(&record.data)
            .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON data: {}", e)))?;
            
        Ok((id, data))
    }
    
    // Helper method to convert Cassandra data to a JSON record
    fn cassandra_to_record(&self, id: String, data: String) -> Result<Record> {
        let data_value = serde_json::from_str(&data)
            .map_err(|e| Error::DataConversionError(format!("Failed to parse JSON data: {}", e)))?;
            
        Ok(Record {
            id,
            data: data_value,
        })
    }
}

#[async_trait]
impl DatabaseOperation for CassandraConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        // Ensure the table exists
        self.ensure_table_exists(collection).await?;
        
        let mut session = self.session.lock().await;
        
        // Convert record to Cassandra data
        let (id, data) = self.record_to_cassandra(record)?;
        
        // Insert the record
        let insert_query = format!(
            "INSERT INTO {} (id, data) VALUES (?, ?)",
            collection
        );
        
        let mut insert_stmt = Statement::new(insert_query.as_str());
        insert_stmt.bind_string(0, &id)?;
        insert_stmt.bind_string(1, &data)?;
        
        session
            .execute(&insert_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra insert error: {:?}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        // Ensure the table exists
        self.ensure_table_exists(collection).await?;
        
        let mut session = self.session.lock().await;
        
        // Query the record
        let select_query = format!(
            "SELECT id, data FROM {} WHERE id = ?",
            collection
        );
        
        let mut select_stmt = Statement::new(select_query.as_str());
        select_stmt.bind_string(0, id)?;
        
        let result = session
            .execute(&select_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra select error: {:?}", e)))?;
            
        // Check if we got a result
        if result.row_count() == 0 {
            return Ok(None);
        }
        
        // Get the first row
        let row = result
            .first_row()
            .ok_or_else(|| Error::DataConversionError("Failed to get row from Cassandra result".to_string()))?;
            
        // Extract data
        let id = row.get_column(0)
            .ok_or_else(|| Error::DataConversionError("Failed to get id column".to_string()))?
            .get_string()
            .map_err(|e| Error::DataConversionError(format!("Failed to get id: {:?}", e)))?;
            
        let data = row.get_column(1)
            .ok_or_else(|| Error::DataConversionError("Failed to get data column".to_string()))?
            .get_string()
            .map_err(|e| Error::DataConversionError(format!("Failed to get data: {:?}", e)))?;
            
        // Convert to record
        let record = self.cassandra_to_record(id, data)?;
        
        Ok(Some(record))
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        // Ensure the table exists
        self.ensure_table_exists(collection).await?;
        
        let mut session = self.session.lock().await;
        
        // Convert record to Cassandra data
        let (id, data) = self.record_to_cassandra(record)?;
        
        // Check if the record exists
        let select_query = format!(
            "SELECT id FROM {} WHERE id = ?",
            collection
        );
        
        let mut select_stmt = Statement::new(select_query.as_str());
        select_stmt.bind_string(0, &id)?;
        
        let result = session
            .execute(&select_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra select error: {:?}", e)))?;
            
        if result.row_count() == 0 {
            return Ok(false);
        }
        
        // Update the record
        let update_query = format!(
            "UPDATE {} SET data = ? WHERE id = ?",
            collection
        );
        
        let mut update_stmt = Statement::new(update_query.as_str());
        update_stmt.bind_string(0, &data)?;
        update_stmt.bind_string(1, &id)?;
        
        session
            .execute(&update_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra update error: {:?}", e)))?;
            
        Ok(true)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        // Ensure the table exists
        self.ensure_table_exists(collection).await?;
        
        let mut session = self.session.lock().await;
        
        // Check if the record exists
        let select_query = format!(
            "SELECT id FROM {} WHERE id = ?",
            collection
        );
        
        let mut select_stmt = Statement::new(select_query.as_str());
        select_stmt.bind_string(0, id)?;
        
        let result = session
            .execute(&select_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra select error: {:?}", e)))?;
            
        if result.row_count() == 0 {
            return Ok(false);
        }
        
        // Delete the record
        let delete_query = format!(
            "DELETE FROM {} WHERE id = ?",
            collection
        );
        
        let mut delete_stmt = Statement::new(delete_query.as_str());
        delete_stmt.bind_string(0, id)?;
        
        session
            .execute(&delete_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra delete error: {:?}", e)))?;
            
        Ok(true)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        // Ensure the table exists
        self.ensure_table_exists(collection).await?;
        
        let mut session = self.session.lock().await;
        
        // Cassandra doesn't support complex JSON filtering natively
        // For simplicity, we'll fetch all records and filter in-memory
        let select_query = format!(
            "SELECT id, data FROM {}",
            collection
        );
        
        let select_stmt = Statement::new(select_query.as_str());
        
        let result = session
            .execute(&select_stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra select error: {:?}", e)))?;
            
        // Convert results to records
        let mut records = Vec::new();
        
        for row_index in 0..result.row_count() {
            if let Some(row) = result.get_row(row_index) {
                // Extract data
                let id = row.get_column(0)
                    .ok_or_else(|| Error::DataConversionError("Failed to get id column".to_string()))?
                    .get_string()
                    .map_err(|e| Error::DataConversionError(format!("Failed to get id: {:?}", e)))?;
                    
                let data_str = row.get_column(1)
                    .ok_or_else(|| Error::DataConversionError("Failed to get data column".to_string()))?
                    .get_string()
                    .map_err(|e| Error::DataConversionError(format!("Failed to get data: {:?}", e)))?;
                    
                // Parse the data as JSON
                let data: serde_json::Value = serde_json::from_str(&data_str)
                    .map_err(|e| Error::DataConversionError(format!("Failed to parse JSON data: {}", e)))?;
                    
                // Filter based on query
                if !params.query.is_null() && !params.query.as_object().map_or(true, |obj| obj.is_empty()) {
                    let mut include = true;
                    
                    if let (Some(query_obj), Some(data_obj)) = (params.query.as_object(), data.as_object()) {
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
                    
                    if !include {
                        continue;
                    }
                }
                
                records.push(Record { id, data });
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
        // Cassandra doesn't support true ACID transactions across multiple rows/tables
        // We'll simulate a transaction with a batch statement
        let mut session = self.session.lock().await;
        
        // Collect operations for validation first
        for operation in &operations {
            match operation {
                TransactionOperation::Create { collection, .. } |
                TransactionOperation::Update { collection, .. } |
                TransactionOperation::Delete { collection, .. } => {
                    // Ensure the table exists for each operation
                    self.ensure_table_exists(collection).await?;
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg.clone()));
                }
            }
        }
        
        // Create a batch statement
        let mut batch = Statement::new("BEGIN BATCH");
        
        // Add operations to the batch
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    let (id, data) = self.record_to_cassandra(&record)?;
                    
                    let insert_query = format!(
                        "INSERT INTO {} (id, data) VALUES ('{}', '{}')",
                        collection, id, data.replace("'", "''")
                    );
                    
                    batch.add_query(insert_query.as_str());
                },
                TransactionOperation::Update { collection, record } => {
                    let (id, data) = self.record_to_cassandra(&record)?;
                    
                    let update_query = format!(
                        "UPDATE {} SET data = '{}' WHERE id = '{}'",
                        collection, data.replace("'", "''"), id
                    );
                    
                    batch.add_query(update_query.as_str());
                },
                TransactionOperation::Delete { collection, id } => {
                    let delete_query = format!(
                        "DELETE FROM {} WHERE id = '{}'",
                        collection, id
                    );
                    
                    batch.add_query(delete_query.as_str());
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        // Finalize the batch
        batch.add_query("APPLY BATCH");
        
        // Execute the batch
        session
            .execute(&batch)
            .map_err(|e| Error::DatabaseTransactionError(format!("Cassandra batch error: {:?}", e)))?;
            
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let mut session = self.session.lock().await;
        
        // Query to get table names
        let query = format!(
            "SELECT table_name FROM system_schema.tables WHERE keyspace_name = '{}'",
            self.keyspace
        );
        
        let stmt = Statement::new(query.as_str());
        
        let result = session
            .execute(&stmt)
            .map_err(|e| Error::DatabaseQueryError(format!("Cassandra schema query error: {:?}", e)))?;
            
        // Extract table names
        let mut collections = Vec::new();
        
        for row_index in 0..result.row_count() {
            if let Some(row) = result.get_row(row_index) {
                if let Some(column) = row.get_column(0) {
                    if let Ok(table_name) = column.get_string() {
                        collections.push(table_name);
                    }
                }
            }
        }
        
        Ok(DatabaseInfo {
            db_type: "cassandra".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for CassandraConnection {
    fn get_db_type(&self) -> &str {
        "cassandra"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        let mut session = self.session.lock().await;
        
        let stmt = Statement::new("SELECT release_version FROM system.local");
        
        session
            .execute(&stmt)
            .map_err(|e| Error::DatabaseConnectionError(format!("Cassandra health check failed: {:?}", e)))?;
            
        Ok(())
    }
}
