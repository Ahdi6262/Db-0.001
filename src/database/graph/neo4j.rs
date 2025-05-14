use async_trait::async_trait;
use neo4rs::{
    Graph, Node, Relation, query, ConfigBuilder, NodeProperties, NodeProperty, Txn
};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct Neo4jConnection {
    graph: Arc<Mutex<Graph>>,
    db_name: String,
}

impl Neo4jConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = if let Some(conn_str) = &config.connection_string {
            conn_str.clone()
        } else {
            // Build connection string from components
            let host = config.host.as_deref().unwrap_or("localhost");
            let port = config.port.unwrap_or(7687);
            let username = config.username.as_deref().unwrap_or("neo4j");
            let password = config.password.as_deref().unwrap_or("neo4j");
            
            format!("bolt://{}:{}@{}:{}", username, password, host, port)
        };

        let timeout = config.timeout_seconds.unwrap_or(30);
        
        let config = ConfigBuilder::default()
            .uri(connection_string)
            .db(config.name.as_str())
            .user(config.username.as_deref().unwrap_or("neo4j"))
            .password(config.password.as_deref().unwrap_or("neo4j"))
            .fetch_size(500)
            .max_connections(config.pool_size.unwrap_or(10) as usize)
            .build()
            .map_err(|e| Error::DatabaseConnectionError(format!("Neo4j config error: {}", e)))?;
        
        let graph = Graph::connect(config).await
            .map_err(|e| Error::DatabaseConnectionError(format!("Neo4j connection error: {}", e)))?;
            
        Ok(Self {
            graph: Arc::new(Mutex::new(graph)),
            db_name: config.name.clone(),
        })
    }
    
    // Helper method to convert a record to Neo4j properties
    fn record_to_properties(&self, record: &Record) -> Result<HashMap<String, String>> {
        let mut properties = HashMap::new();
        
        if let Some(obj) = record.data.as_object() {
            for (key, value) in obj {
                let string_value = match value {
                    serde_json::Value::Null => "null".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => serde_json::to_string(value)
                        .map_err(|e| Error::DataConversionError(format!("Failed to serialize JSON value: {}", e)))?,
                };
                
                properties.insert(key.clone(), string_value);
            }
        }
        
        // Add id property
        properties.insert("id".to_string(), record.id.clone());
        
        Ok(properties)
    }
    
    // Helper method to convert Neo4j node to a record
    fn node_to_record(&self, node: Node) -> Result<Record> {
        let id = match node.get::<String>("id") {
            Ok(id) => id,
            Err(_) => node.id().to_string(),
        };
        
        let mut data_map = serde_json::Map::new();
        
        // Convert node properties to a JSON object
        for key in node.keys() {
            if key == "id" {
                continue; // Skip id property as it's handled separately
            }
            
            let value = match node.get::<String>(&key) {
                Ok(v) => {
                    // Try to parse as a more specific type if possible
                    if let Ok(num) = v.parse::<i64>() {
                        serde_json::Value::Number(serde_json::Number::from(num))
                    } else if let Ok(num) = v.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(num) {
                            serde_json::Value::Number(n)
                        } else {
                            serde_json::Value::String(v)
                        }
                    } else if v == "true" {
                        serde_json::Value::Bool(true)
                    } else if v == "false" {
                        serde_json::Value::Bool(false)
                    } else if v == "null" {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(v)
                    }
                },
                Err(_) => serde_json::Value::Null,
            };
            
            data_map.insert(key, value);
        }
        
        Ok(Record {
            id,
            data: serde_json::Value::Object(data_map),
        })
    }
}

#[async_trait]
impl DatabaseOperation for Neo4jConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        let graph = self.graph.lock().await;
        
        // Convert record to properties
        let properties = self.record_to_properties(record)?;
        
        // Build query for creating a node
        let properties_str = properties.iter()
            .map(|(k, v)| format!("{}: ${}Property", k, k))
            .collect::<Vec<_>>()
            .join(", ");
            
        let query_str = format!(
            "CREATE (n:{} {{{}}}) RETURN n",
            collection, properties_str
        );
        
        let mut query = query(&query_str);
        
        // Add properties as parameters
        for (key, value) in properties {
            query = query.param(format!("{}Property", key), value);
        }
        
        // Execute query
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j create error: {}", e)))?;
            
        // Consume result
        while let Some(_) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {}
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        let graph = self.graph.lock().await;
        
        // Query for a node with the given id
        let query_str = format!(
            "MATCH (n:{} {{id: $id}}) RETURN n",
            collection
        );
        
        let query = query(&query_str).param("id", id);
        
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j read error: {}", e)))?;
            
        // Get the first result
        if let Some(row) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {
                
            let node: Node = row.get("n")
                .map_err(|e| Error::DataConversionError(format!("Failed to get node from result: {}", e)))?;
                
            let record = self.node_to_record(node)?;
            
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        let graph = self.graph.lock().await;
        
        // Convert record to properties
        let properties = self.record_to_properties(record)?;
        
        // Build query for updating a node
        let properties_str = properties.iter()
            .filter(|(k, _)| *k != "id") // Skip id property for SET clause
            .map(|(k, _)| format!("n.{} = ${}Property", k, k))
            .collect::<Vec<_>>()
            .join(", ");
            
        let query_str = format!(
            "MATCH (n:{} {{id: $id}}) SET {} RETURN n",
            collection, properties_str
        );
        
        let mut query = query(&query_str).param("id", &record.id);
        
        // Add properties as parameters
        for (key, value) in properties {
            if key != "id" { // Skip id property as it's already added
                query = query.param(format!("{}Property", key), value);
            }
        }
        
        // Execute query
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j update error: {}", e)))?;
            
        // Check if a node was updated
        if let Some(_) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        let graph = self.graph.lock().await;
        
        // Query for deleting a node
        let query_str = format!(
            "MATCH (n:{} {{id: $id}}) DELETE n RETURN count(n) as count",
            collection
        );
        
        let query = query(&query_str).param("id", id);
        
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j delete error: {}", e)))?;
            
        // Check if a node was deleted
        if let Some(row) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {
                
            let count: i64 = row.get("count")
                .map_err(|e| Error::DataConversionError(format!("Failed to get delete count: {}", e)))?;
                
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        let graph = self.graph.lock().await;
        
        // Build Cypher query from query parameters
        let mut conditions = Vec::new();
        
        if !params.query.is_null() && !params.query.as_object().map_or(true, |obj| obj.is_empty()) {
            if let Some(obj) = params.query.as_object() {
                for (key, value) in obj {
                    match value {
                        serde_json::Value::Null => {
                            conditions.push(format!("n.{} IS NULL", key));
                        },
                        serde_json::Value::Bool(b) => {
                            conditions.push(format!("n.{} = {}", key, b));
                        },
                        serde_json::Value::Number(n) => {
                            conditions.push(format!("n.{} = {}", key, n));
                        },
                        serde_json::Value::String(s) => {
                            conditions.push(format!("n.{} = '{}'", key, s.replace("'", "\\'")));
                        },
                        _ => {
                            conditions.push(format!("n.{} = '{}'", key, value.to_string().replace("'", "\\'")));
                        },
                    }
                }
            }
        }
        
        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        // Build ordering clause
        let order_clause = if let Some(sort_by) = &params.sort_by {
            let direction = params.sort_order.as_deref().unwrap_or("asc");
            format!("ORDER BY n.{} {}", sort_by, direction)
        } else {
            "".to_string()
        };
        
        // Build pagination
        let skip_clause = if let Some(offset) = params.offset {
            format!("SKIP {}", offset)
        } else {
            "".to_string()
        };
        
        let limit_clause = if let Some(limit) = params.limit {
            format!("LIMIT {}", limit)
        } else {
            "".to_string()
        };
        
        // Build final query
        let query_str = format!(
            "MATCH (n:{}) {} {} {} {} RETURN n",
            collection, where_clause, order_clause, skip_clause, limit_clause
        );
        
        let query = query(&query_str);
        
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j query error: {}", e)))?;
            
        // Collect results
        let mut records = Vec::new();
        
        while let Some(row) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {
                
            let node: Node = row.get("n")
                .map_err(|e| Error::DataConversionError(format!("Failed to get node from result: {}", e)))?;
                
            let record = self.node_to_record(node)?;
            
            records.push(record);
        }
        
        Ok(records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        let graph = self.graph.lock().await;
        
        // Begin transaction
        let mut txn = graph.start_txn().await
            .map_err(|e| Error::DatabaseTransactionError(format!("Neo4j transaction start error: {}", e)))?;
            
        // Execute operations
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    // Convert record to properties
                    let properties = self.record_to_properties(&record)?;
                    
                    // Build query for creating a node
                    let properties_str = properties.iter()
                        .map(|(k, v)| format!("{}: ${}Property", k, k))
                        .collect::<Vec<_>>()
                        .join(", ");
                        
                    let query_str = format!(
                        "CREATE (n:{} {{{}}}) RETURN n",
                        collection, properties_str
                    );
                    
                    let mut query = query(&query_str);
                    
                    // Add properties as parameters
                    for (key, value) in properties {
                        query = query.param(format!("{}Property", key), value);
                    }
                    
                    // Execute query in transaction
                    let mut result = txn.execute(query).await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j create error in transaction: {}", e)))?;
                        
                    // Consume result
                    while let Some(_) = result.next().await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error in transaction: {}", e)))? {}
                },
                TransactionOperation::Update { collection, record } => {
                    // Convert record to properties
                    let properties = self.record_to_properties(&record)?;
                    
                    // Build query for updating a node
                    let properties_str = properties.iter()
                        .filter(|(k, _)| *k != "id") // Skip id property for SET clause
                        .map(|(k, _)| format!("n.{} = ${}Property", k, k))
                        .collect::<Vec<_>>()
                        .join(", ");
                        
                    let query_str = format!(
                        "MATCH (n:{} {{id: $id}}) SET {} RETURN n",
                        collection, properties_str
                    );
                    
                    let mut query = query(&query_str).param("id", &record.id);
                    
                    // Add properties as parameters
                    for (key, value) in properties {
                        if key != "id" { // Skip id property as it's already added
                            query = query.param(format!("{}Property", key), value);
                        }
                    }
                    
                    // Execute query in transaction
                    let mut result = txn.execute(query).await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j update error in transaction: {}", e)))?;
                        
                    // Consume result
                    while let Some(_) = result.next().await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error in transaction: {}", e)))? {}
                },
                TransactionOperation::Delete { collection, id } => {
                    // Query for deleting a node
                    let query_str = format!(
                        "MATCH (n:{} {{id: $id}}) DELETE n",
                        collection
                    );
                    
                    let query = query(&query_str).param("id", id);
                    
                    // Execute query in transaction
                    let mut result = txn.execute(query).await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j delete error in transaction: {}", e)))?;
                        
                    // Consume result
                    while let Some(_) = result.next().await
                        .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error in transaction: {}", e)))? {}
                },
                TransactionOperation::Invalid(msg) => {
                    // Rollback transaction on invalid operation
                    txn.rollback().await
                        .map_err(|e| Error::DatabaseTransactionError(format!("Neo4j transaction rollback error: {}", e)))?;
                        
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        // Commit transaction
        txn.commit().await
            .map_err(|e| Error::DatabaseTransactionError(format!("Neo4j transaction commit error: {}", e)))?;
            
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let graph = self.graph.lock().await;
        
        // Query to get all node labels (collection names)
        let query_str = "CALL db.labels() YIELD label RETURN label";
        
        let query = query(query_str);
        
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j labels query error: {}", e)))?;
            
        // Collect labels
        let mut collections = Vec::new();
        
        while let Some(row) = result.next().await
            .map_err(|e| Error::DatabaseQueryError(format!("Neo4j result error: {}", e)))? {
                
            let label: String = row.get("label")
                .map_err(|e| Error::DataConversionError(format!("Failed to get label from result: {}", e)))?;
                
            collections.push(label);
        }
        
        Ok(DatabaseInfo {
            db_type: "neo4j".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for Neo4jConnection {
    fn get_db_type(&self) -> &str {
        "neo4j"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        let graph = self.graph.lock().await;
        
        let query = query("RETURN 1 as n");
        
        let mut result = graph.execute(query).await
            .map_err(|e| Error::DatabaseConnectionError(format!("Neo4j health check failed: {}", e)))?;
            
        if let Some(_) = result.next().await
            .map_err(|e| Error::DatabaseConnectionError(format!("Neo4j health check failed: {}", e)))? {
            Ok(())
        } else {
            Err(Error::DatabaseConnectionError("Neo4j health check failed: no result".to_string()))
        }
    }
}
