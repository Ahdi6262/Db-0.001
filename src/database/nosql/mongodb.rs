use async_trait::async_trait;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Document, to_document, from_document},
    Client, Collection, Database, options::{ClientOptions, FindOptions}
};
use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct MongoDbConnection {
    client: Client,
    database: Database,
    db_name: String,
}

impl MongoDbConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = config.connection_string.clone()
            .unwrap_or_else(|| {
                let host = config.host.as_deref().unwrap_or("localhost");
                let port = config.port.unwrap_or(27017);
                
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    format!("mongodb://{}:{}@{}:{}", username, password, host, port)
                } else {
                    format!("mongodb://{}:{}", host, port)
                }
            });

        let timeout = config.timeout_seconds.unwrap_or(30);
        
        let mut client_options = ClientOptions::parse(&connection_string).await
            .map_err(|e| Error::DatabaseConnectionError(format!("MongoDB connection error: {}", e)))?;
            
        client_options.connect_timeout = Some(Duration::from_secs(timeout));
        
        if let Some(pool_size) = config.pool_size {
            client_options.max_pool_size = Some(pool_size as u32);
        }
        
        let client = Client::with_options(client_options)
            .map_err(|e| Error::DatabaseConnectionError(format!("MongoDB client creation error: {}", e)))?;
            
        let database = client.database(&config.name);
        
        Ok(Self {
            client,
            database,
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to get a collection
    fn get_collection(&self, collection_name: &str) -> Collection<Document> {
        self.database.collection(collection_name)
    }
    
    // Helper method to convert MongoDB document to a record
    fn document_to_record(&self, doc: Document) -> Result<Record> {
        // Extract the _id field
        let id = match doc.get("_id") {
            Some(id_bson) => id_bson.to_string(),
            None => return Err(Error::DataConversionError("MongoDB document missing _id field".to_string())),
        };
        
        // Remove _id from the document to avoid duplicates
        let mut doc_without_id = doc.clone();
        doc_without_id.remove("_id");
        
        // Convert BSON document to JSON value
        let data = match serde_json::to_value(&doc_without_id) {
            Ok(v) => v,
            Err(e) => return Err(Error::DataConversionError(format!("Failed to convert MongoDB document to JSON: {}", e))),
        };
        
        Ok(Record { id, data })
    }
    
    // Helper method to convert a record to MongoDB document
    fn record_to_document(&self, record: &Record) -> Result<Document> {
        // Convert JSON data to BSON document
        let mut doc = match to_document(&record.data) {
            Ok(d) => d,
            Err(e) => return Err(Error::DataConversionError(format!("Failed to convert JSON to MongoDB document: {}", e))),
        };
        
        // Add _id field
        doc.insert("_id", record.id.clone());
        
        Ok(doc)
    }
    
    // Helper method to build a MongoDB filter from query params
    fn build_filter(&self, params: &QueryParams) -> Result<Document> {
        if params.query.is_null() {
            return Ok(Document::new());
        }
        
        // Convert the JSON query to a MongoDB filter
        match to_document(&params.query) {
            Ok(doc) => Ok(doc),
            Err(e) => Err(Error::InvalidQueryError(format!("Failed to convert query to MongoDB filter: {}", e))),
        }
    }
}

#[async_trait]
impl DatabaseOperation for MongoDbConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        let coll = self.get_collection(collection);
        
        let doc = self.record_to_document(record)?;
        
        coll.insert_one(doc, None).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB insert error: {}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        let coll = self.get_collection(collection);
        
        let filter = doc! { "_id": id };
        
        let result = coll.find_one(filter, None).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB find error: {}", e)))?;
            
        match result {
            Some(doc) => {
                let record = self.document_to_record(doc)?;
                Ok(Some(record))
            },
            None => Ok(None),
        }
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        let coll = self.get_collection(collection);
        
        let filter = doc! { "_id": &record.id };
        
        // Convert JSON data to BSON document
        let doc_data = match to_document(&record.data) {
            Ok(d) => d,
            Err(e) => return Err(Error::DataConversionError(format!("Failed to convert JSON to MongoDB document: {}", e))),
        };
        
        // Use $set to update fields without replacing the entire document
        let update = doc! { "$set": doc_data };
        
        let result = coll.update_one(filter, update, None).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB update error: {}", e)))?;
            
        Ok(result.modified_count > 0)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        let coll = self.get_collection(collection);
        
        let filter = doc! { "_id": id };
        
        let result = coll.delete_one(filter, None).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB delete error: {}", e)))?;
            
        Ok(result.deleted_count > 0)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        let coll = self.get_collection(collection);
        
        let filter = self.build_filter(params)?;
        
        let mut options = FindOptions::default();
        
        // Handle pagination
        if let Some(limit) = params.limit {
            options.limit = Some(limit as i64);
        }
        
        if let Some(offset) = params.offset {
            options.skip = Some(offset as u64);
        }
        
        // Handle sorting
        if let Some(sort_by) = &params.sort_by {
            let direction = match params.sort_order.as_deref() {
                Some("desc") => -1,
                _ => 1,
            };
            
            options.sort = Some(doc! { sort_by: direction });
        }
        
        let cursor = coll.find(filter, options).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB find error: {}", e)))?;
            
        let docs: Vec<Document> = cursor.try_collect().await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB cursor error: {}", e)))?;
            
        let mut records = Vec::with_capacity(docs.len());
        for doc in docs {
            let record = self.document_to_record(doc)?;
            records.push(record);
        }
        
        Ok(records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        // Start a session for the transaction
        let mut session = self.client.start_session(None).await
            .map_err(|e| Error::DatabaseTransactionError(format!("MongoDB session error: {}", e)))?;
            
        // Start a transaction
        session.start_transaction(None).await
            .map_err(|e| Error::DatabaseTransactionError(format!("MongoDB transaction start error: {}", e)))?;
            
        // Execute operations in the transaction
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    let coll = self.get_collection(&collection);
                    let doc = self.record_to_document(&record)?;
                    
                    coll.insert_one_with_session(doc, None, &mut session).await
                        .map_err(|e| Error::DatabaseQueryError(format!("MongoDB insert error: {}", e)))?;
                },
                TransactionOperation::Update { collection, record } => {
                    let coll = self.get_collection(&collection);
                    let filter = doc! { "_id": &record.id };
                    
                    // Convert JSON data to BSON document
                    let doc_data = match to_document(&record.data) {
                        Ok(d) => d,
                        Err(e) => return Err(Error::DataConversionError(format!("Failed to convert JSON to MongoDB document: {}", e))),
                    };
                    
                    let update = doc! { "$set": doc_data };
                    
                    coll.update_one_with_session(filter, update, None, &mut session).await
                        .map_err(|e| Error::DatabaseQueryError(format!("MongoDB update error: {}", e)))?;
                },
                TransactionOperation::Delete { collection, id } => {
                    let coll = self.get_collection(&collection);
                    let filter = doc! { "_id": id };
                    
                    coll.delete_one_with_session(filter, None, &mut session).await
                        .map_err(|e| Error::DatabaseQueryError(format!("MongoDB delete error: {}", e)))?;
                },
                TransactionOperation::Invalid(msg) => {
                    // Abort the transaction on invalid operation
                    session.abort_transaction().await
                        .map_err(|e| Error::DatabaseTransactionError(format!("MongoDB transaction abort error: {}", e)))?;
                        
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        // Commit the transaction
        session.commit_transaction().await
            .map_err(|e| Error::DatabaseTransactionError(format!("MongoDB transaction commit error: {}", e)))?;
            
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        // Get list of collections
        let collections = self.database.list_collection_names(None).await
            .map_err(|e| Error::DatabaseQueryError(format!("MongoDB list collections error: {}", e)))?;
            
        Ok(DatabaseInfo {
            db_type: "mongodb".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for MongoDbConnection {
    fn get_db_type(&self) -> &str {
        "mongodb"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        self.client.list_database_names(None, None).await
            .map_err(|e| Error::DatabaseConnectionError(format!("MongoDB health check failed: {}", e)))?;
            
        Ok(())
    }
}
