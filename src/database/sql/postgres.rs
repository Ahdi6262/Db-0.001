use async_trait::async_trait;
use sqlx::{postgres::{PgPool, PgPoolOptions}, Row};
use std::time::Duration;

use crate::config::DatabaseConfig;
use crate::database::interface::{DatabaseConnection, DatabaseInfo, DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::error::{Error, Result};

pub struct PostgresConnection {
    pool: PgPool,
    db_name: String,
}

impl PostgresConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let connection_string = if let Some(conn_str) = &config.connection_string {
            conn_str.clone()
        } else {
            // Build connection string from components
            let host = config.host.as_deref().unwrap_or("localhost");
            let port = config.port.unwrap_or(5432);
            let username = config.username.as_deref().unwrap_or("postgres");
            let password = config.password.as_deref().unwrap_or("postgres");
            let database = &config.name;
            
            format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database)
        };

        let pool_size = config.pool_size.unwrap_or(10) as u32;
        let timeout = config.timeout_seconds.unwrap_or(30);

        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .acquire_timeout(Duration::from_secs(timeout))
            .connect(&connection_string)
            .await
            .map_err(|e| Error::DatabaseConnectionError(format!("PostgreSQL connection error: {}", e)))?;

        Ok(Self {
            pool,
            db_name: config.name.clone(),
        })
    }

    // Helper method to convert JSON to a record format
    async fn json_to_record(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        let query = format!(
            "SELECT data FROM {} WHERE id = $1",
            sqlx::Postgres::identifier(collection)
        );
        
        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL query error: {}", e)))?;
            
        if let Some(row) = row {
            let data: serde_json::Value = row.try_get("data")
                .map_err(|e| Error::DataConversionError(format!("Failed to parse JSON data: {}", e)))?;
                
            Ok(Some(Record {
                id: id.to_string(),
                data,
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl DatabaseOperation for PostgresConnection {
    async fn create(&self, collection: &str, record: &Record) -> Result<()> {
        // Ensure the table exists
        let create_table_query = format!(
            "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data JSONB NOT NULL)",
            sqlx::Postgres::identifier(collection)
        );
        
        sqlx::query(&create_table_query)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table creation error: {}", e)))?;
            
        // Insert the record
        let query = format!(
            "INSERT INTO {} (id, data) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET data = $2",
            sqlx::Postgres::identifier(collection)
        );
        
        sqlx::query(&query)
            .bind(&record.id)
            .bind(&record.data)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL insert error: {}", e)))?;
            
        Ok(())
    }
    
    async fn read(&self, collection: &str, id: &str) -> Result<Option<Record>> {
        // Check if the table exists
        let check_query = "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        )";
        
        let exists: bool = sqlx::query_scalar(check_query)
            .bind(collection)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table check error: {}", e)))?;
            
        if !exists {
            return Ok(None);
        }
        
        self.json_to_record(collection, id).await
    }
    
    async fn update(&self, collection: &str, record: &Record) -> Result<bool> {
        // Check if the table exists
        let check_query = "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        )";
        
        let exists: bool = sqlx::query_scalar(check_query)
            .bind(collection)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table check error: {}", e)))?;
            
        if !exists {
            return Ok(false);
        }
        
        // Update the record
        let query = format!(
            "UPDATE {} SET data = $1 WHERE id = $2",
            sqlx::Postgres::identifier(collection)
        );
        
        let result = sqlx::query(&query)
            .bind(&record.data)
            .bind(&record.id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL update error: {}", e)))?;
            
        Ok(result.rows_affected() > 0)
    }
    
    async fn delete(&self, collection: &str, id: &str) -> Result<bool> {
        // Check if the table exists
        let check_query = "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        )";
        
        let exists: bool = sqlx::query_scalar(check_query)
            .bind(collection)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table check error: {}", e)))?;
            
        if !exists {
            return Ok(false);
        }
        
        // Delete the record
        let query = format!(
            "DELETE FROM {} WHERE id = $1",
            sqlx::Postgres::identifier(collection)
        );
        
        let result = sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL delete error: {}", e)))?;
            
        Ok(result.rows_affected() > 0)
    }
    
    async fn query(&self, collection: &str, params: &QueryParams) -> Result<Vec<Record>> {
        // Check if the table exists
        let check_query = "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = $1
        )";
        
        let exists: bool = sqlx::query_scalar(check_query)
            .bind(collection)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table check error: {}", e)))?;
            
        if !exists {
            return Ok(Vec::new());
        }
        
        // Build the query based on parameters
        let mut query = format!(
            "SELECT id, data FROM {}",
            sqlx::Postgres::identifier(collection)
        );
        
        // Handle filtering
        if !params.query.is_null() && !params.query.as_object().map_or(true, |obj| obj.is_empty()) {
            query.push_str(" WHERE ");
            
            if let Some(obj) = params.query.as_object() {
                let mut first = true;
                for (key, value) in obj {
                    if !first {
                        query.push_str(" AND ");
                    }
                    
                    // Convert to proper SQL condition
                    match value {
                        serde_json::Value::Null => {
                            query.push_str(&format!("data->>'{}' IS NULL", key));
                        },
                        serde_json::Value::String(s) => {
                            query.push_str(&format!("data->>'{}' = '{}'", key, s.replace('\'', "''")));
                        },
                        serde_json::Value::Number(n) => {
                            query.push_str(&format!("(data->'{}')::numeric = {}", key, n));
                        },
                        serde_json::Value::Bool(b) => {
                            query.push_str(&format!("(data->'{}')::boolean = {}", key, b));
                        },
                        _ => {
                            query.push_str(&format!("data->>'{}' = '{}'", key, value.to_string().replace('\'', "''")));
                        },
                    }
                    
                    first = false;
                }
            }
        }
        
        // Handle sorting
        if let Some(sort_by) = &params.sort_by {
            let direction = params.sort_order.as_deref().unwrap_or("asc");
            query.push_str(&format!(" ORDER BY data->>'{0}' {1}", sort_by, direction));
        }
        
        // Handle pagination
        if let Some(limit) = params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
            
            if let Some(offset) = params.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        }
        
        // Execute the query
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL query error: {}", e)))?;
            
        // Convert rows to records
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.try_get("id")
                .map_err(|e| Error::DataConversionError(format!("Failed to get ID: {}", e)))?;
                
            let data: serde_json::Value = row.try_get("data")
                .map_err(|e| Error::DataConversionError(format!("Failed to parse JSON data: {}", e)))?;
                
            records.push(Record {
                id,
                data,
            });
        }
        
        Ok(records)
    }
    
    async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| Error::DatabaseTransactionError(format!("PostgreSQL transaction start error: {}", e)))?;
            
        for operation in operations {
            match operation {
                TransactionOperation::Create { collection, record } => {
                    // Ensure the table exists
                    let create_table_query = format!(
                        "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data JSONB NOT NULL)",
                        sqlx::Postgres::identifier(&collection)
                    );
                    
                    sqlx::query(&create_table_query)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table creation error: {}", e)))?;
                    
                    // Insert the record
                    let query = format!(
                        "INSERT INTO {} (id, data) VALUES ($1, $2)",
                        sqlx::Postgres::identifier(&collection)
                    );
                    
                    sqlx::query(&query)
                        .bind(&record.id)
                        .bind(&record.data)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL insert error: {}", e)))?;
                },
                TransactionOperation::Update { collection, record } => {
                    // Ensure the table exists
                    let create_table_query = format!(
                        "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data JSONB NOT NULL)",
                        sqlx::Postgres::identifier(&collection)
                    );
                    
                    sqlx::query(&create_table_query)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table creation error: {}", e)))?;
                    
                    // Update the record
                    let query = format!(
                        "UPDATE {} SET data = $1 WHERE id = $2",
                        sqlx::Postgres::identifier(&collection)
                    );
                    
                    sqlx::query(&query)
                        .bind(&record.data)
                        .bind(&record.id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL update error: {}", e)))?;
                },
                TransactionOperation::Delete { collection, id } => {
                    // Check if the table exists
                    let check_query = "SELECT EXISTS (
                        SELECT FROM information_schema.tables 
                        WHERE table_schema = 'public' 
                        AND table_name = $1
                    )";
                    
                    let exists: bool = sqlx::query_scalar(check_query)
                        .bind(&collection)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL table check error: {}", e)))?;
                        
                    if exists {
                        // Delete the record
                        let query = format!(
                            "DELETE FROM {} WHERE id = $1",
                            sqlx::Postgres::identifier(&collection)
                        );
                        
                        sqlx::query(&query)
                            .bind(&id)
                            .execute(&mut *tx)
                            .await
                            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL delete error: {}", e)))?;
                    }
                },
                TransactionOperation::Invalid(msg) => {
                    return Err(Error::InvalidOperation(msg));
                },
            }
        }
        
        tx.commit().await
            .map_err(|e| Error::DatabaseTransactionError(format!("PostgreSQL transaction commit error: {}", e)))?;
            
        Ok(())
    }
    
    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        // Get list of tables
        let query = "
            SELECT table_name 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
        ";
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseQueryError(format!("PostgreSQL query error: {}", e)))?;
            
        let collections = rows.into_iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();
            
        Ok(DatabaseInfo {
            db_type: "postgresql".to_string(),
            name: self.db_name.clone(),
            status: "connected".to_string(),
            collections: Some(collections),
        })
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    fn get_db_type(&self) -> &str {
        "postgresql"
    }
    
    fn get_db_name(&self) -> &str {
        &self.db_name
    }
    
    async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::DatabaseConnectionError(format!("PostgreSQL health check failed: {}", e)))?;
            
        Ok(())
    }
}
