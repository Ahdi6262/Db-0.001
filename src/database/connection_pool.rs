use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{AppConfig, DatabaseConfig};
use crate::database::interface::{DatabaseConnection, DatabaseInfo};
use crate::database::sql::postgres::PostgresConnection;
use crate::database::sql::mysql::MySqlConnection;
use crate::database::nosql::mongodb::MongoDbConnection;
use crate::database::nosql::redis::RedisConnection;
// use crate::database::nosql::cassandra::CassandraConnection; // Temporarily commented out due to package not available
use crate::database::graph::neo4j::Neo4jConnection;
use crate::database::time_series::influxdb::InfluxDbConnection;
use crate::database::vector::qdrant::QdrantConnection;
use crate::error::{Error, Result};

/// Manages database connections for different types of databases
pub struct ConnectionManager {
    connections: RwLock<HashMap<String, Arc<dyn DatabaseConnection>>>,
    config: AppConfig,
}

impl ConnectionManager {
    /// Creates a new connection manager with the given configuration
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let mut connections = HashMap::new();
        
        // Initialize connections for each database in the configuration
        for db_config in &config.databases {
            match Self::create_connection(db_config).await {
                Ok(connection) => {
                    let key = format!("{}:{}", db_config.db_type, db_config.name);
                    connections.insert(key, connection);
                    tracing::info!("Initialized connection for {}/{}", db_config.db_type, db_config.name);
                },
                Err(e) => {
                    tracing::error!("Failed to initialize connection for {}/{}: {:?}", 
                        db_config.db_type, db_config.name, e);
                    // Continue with other connections even if one fails
                }
            }
        }

        Ok(Self {
            connections: RwLock::new(connections),
            config: config.clone(),
        })
    }

    /// Creates a database connection based on the database type
    async fn create_connection(db_config: &DatabaseConfig) -> Result<Arc<dyn DatabaseConnection>> {
        match db_config.db_type.as_str() {
            "postgresql" => {
                let conn = PostgresConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "mysql" => {
                let conn = MySqlConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "mongodb" => {
                let conn = MongoDbConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "redis" => {
                let conn = RedisConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "cassandra" => {
                // Temporarily disabled due to package not available
                Err(Error::UnsupportedDatabaseType("cassandra".to_string()))
            },
            "neo4j" => {
                let conn = Neo4jConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "influxdb" => {
                let conn = InfluxDbConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            "qdrant" => {
                let conn = QdrantConnection::new(db_config).await?;
                Ok(Arc::new(conn))
            },
            _ => Err(Error::UnsupportedDatabaseType(db_config.db_type.clone())),
        }
    }

    /// Gets a connection for the specified database type and name
    pub async fn get_connection(&self, db_type: &str, db_name: &str) -> Result<Arc<dyn DatabaseConnection>> {
        let key = format!("{}:{}", db_type, db_name);
        
        // First, try to get an existing connection
        {
            let connections = self.connections.read().await;
            if let Some(connection) = connections.get(&key) {
                return Ok(connection.clone());
            }
        }
        
        // If connection doesn't exist, try to create one
        let db_config = self.config.databases.iter()
            .find(|c| c.db_type == db_type && c.name == db_name)
            .ok_or_else(|| Error::DatabaseNotFound(db_type.to_string(), db_name.to_string()))?;
        
        let connection = Self::create_connection(db_config).await?;
        
        // Add the new connection to the pool
        {
            let mut connections = self.connections.write().await;
            connections.insert(key, connection.clone());
        }
        
        Ok(connection)
    }

    /// Gets information about all databases
    pub async fn get_database_info(&self) -> Result<Vec<DatabaseInfo>> {
        let mut db_infos = Vec::new();
        
        let connections = self.connections.read().await;
        for ((db_type, db_name), connection) in connections.iter()
            .map(|(k, v)| {
                let parts: Vec<&str> = k.split(':').collect();
                ((parts[0].to_string(), parts[1].to_string()), v)
            }) 
        {
            match connection.get_database_info().await {
                Ok(mut info) => {
                    info.db_type = db_type;
                    info.name = db_name;
                    db_infos.push(info);
                },
                Err(e) => {
                    // Add database with error status
                    db_infos.push(DatabaseInfo {
                        db_type,
                        name: db_name,
                        status: format!("Error: {}", e),
                        collections: None,
                    });
                }
            }
        }
        
        Ok(db_infos)
    }
}
