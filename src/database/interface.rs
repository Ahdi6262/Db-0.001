use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::database::types::Value;
use crate::error::Result;

/// Represents a database record
#[derive(Debug, Clone)]
pub struct Record {
    pub id: String,
    pub data: JsonValue,
}

/// Parameters for querying database records
#[derive(Debug, Clone)]
pub struct QueryParams {
    pub query: JsonValue,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Information about a database
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub db_type: String,
    pub name: String,
    pub status: String,
    pub collections: Option<Vec<String>>,
}

/// Represents a transaction operation
#[derive(Debug, Clone)]
pub enum TransactionOperation {
    Create {
        collection: String,
        record: Record,
    },
    Update {
        collection: String,
        record: Record,
    },
    Delete {
        collection: String,
        id: String,
    },
    Invalid(String), // For validation errors
}

/// Trait that defines database operations
#[async_trait]
pub trait DatabaseOperation {
    /// Creates a new record in the specified collection
    async fn create(&self, collection: &str, record: &Record) -> Result<()>;
    
    /// Reads a record with the specified ID from the collection
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>>;
    
    /// Updates the record with the specified ID
    /// Returns true if the record was updated, false if not found
    async fn update(&self, collection: &str, record: &Record) -> Result<bool>;
    
    /// Deletes the record with the specified ID
    /// Returns true if the record was deleted, false if not found
    async fn delete(&self, collection: &str, id: &str) -> Result<bool>;
    
    /// Queries records based on the provided query parameters
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>>;
    
    /// Executes a transaction with multiple operations
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()>;
    
    /// Gets information about the database
    async fn get_database_info(&self) -> Result<DatabaseInfo>;
}

/// Trait that defines a database connection
#[async_trait]
pub trait DatabaseConnection: DatabaseOperation + Send + Sync {
    /// Returns the type of database this connection is for
    fn get_db_type(&self) -> &str;
    
    /// Returns the name of the database
    fn get_db_name(&self) -> &str;
    
    /// Checks if the connection is healthy
    async fn health_check(&self) -> Result<()>;
}
