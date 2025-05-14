use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Row};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Configure logging
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Unified Database API Server");

    // Get connection details from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to PostgreSQL");
    
    // Create connection pool
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await {
            Ok(pool) => {
                tracing::info!("Database connection successful");
                pool
            },
            Err(e) => {
                tracing::error!("Failed to connect to the database: {}", e);
                panic!("Database connection failed");
            }
        };
    
    // Test the connection with a simple query
    match sqlx::query("SELECT 1 as test")
        .fetch_one(&pool)
        .await {
            Ok(_) => tracing::info!("Database query test successful"),
            Err(e) => tracing::error!("Database query test failed: {}", e)
        };
    
    // Wrap the pool in an Arc
    let db_pool = Arc::new(pool);
    
    // Start HTTP server
    tracing::info!("Starting HTTP server at 0.0.0.0:8000");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(
                        web::scope("/databases")
                            .route("", web::get().to(get_database_info))
                            .service(
                                web::scope("/postgresql/postgres")
                                    .route("/records", web::post().to(create_record))
                                    .route("/records/{id}", web::get().to(get_record))
                                    .route("/records/{id}", web::put().to(update_record))
                                    .route("/records/{id}", web::delete().to(delete_record))
                                    .route("/query", web::post().to(query_records))
                            )
                    )
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[derive(Debug, Serialize)]
struct DatabaseInfo {
    db_type: String,
    name: String,
    status: String,
    collections: Option<Vec<String>>,
}

// Get database info endpoint
async fn get_database_info(db: web::Data<Arc<sqlx::PgPool>>) -> impl Responder {
    // Get list of tables
    let query = "
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public'
    ";
    
    match sqlx::query(query)
        .fetch_all(db.as_ref())
        .await {
            Ok(rows) => {
                let collections = rows.into_iter()
                    .map(|row| row.get::<String, _>("table_name"))
                    .collect();
                
                let db_info = DatabaseInfo {
                    db_type: "postgresql".to_string(),
                    name: "postgres".to_string(),
                    status: "connected".to_string(),
                    collections: Some(collections),
                };
                
                HttpResponse::Ok().json(serde_json::json!({
                    "databases": [db_info]
                }))
            },
            Err(e) => {
                tracing::error!("Error retrieving database info: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to retrieve database information: {}", e)
                }))
            }
        }
}

#[derive(Debug, Deserialize)]
struct RecordRequest {
    collection: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct RecordResponse {
    id: String,
    data: serde_json::Value,
}

// Create record endpoint
async fn create_record(
    db: web::Data<Arc<sqlx::PgPool>>,
    record_req: web::Json<RecordRequest>,
) -> impl Responder {
    let collection = &record_req.collection;
    let data = &record_req.data;
    let id = Uuid::new_v4().to_string();
    
    // Create table if it doesn't exist
    let create_table_query = format!(
        "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data JSONB NOT NULL)",
        collection
    );
    
    match sqlx::query(&create_table_query)
        .execute(db.as_ref())
        .await {
            Ok(_) => {},
            Err(e) => {
                tracing::error!("Failed to create table: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create table: {}", e)
                }));
            }
        }
    
    // Insert record
    let insert_query = format!(
        "INSERT INTO {} (id, data) VALUES ($1, $2)",
        collection
    );
    
    match sqlx::query(&insert_query)
        .bind(&id)
        .bind(data)
        .execute(db.as_ref())
        .await {
            Ok(_) => {
                let response = RecordResponse {
                    id,
                    data: data.clone(),
                };
                HttpResponse::Created().json(response)
            },
            Err(e) => {
                tracing::error!("Failed to create record: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to create record: {}", e)
                }))
            }
        }
}

// Get record endpoint
async fn get_record(
    db: web::Data<Arc<sqlx::PgPool>>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    
    // Check if collection is specified
    let collection = match query.get("collection") {
        Some(col) => col,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Collection name is required"
        })),
    };
    
    // Check if table exists
    let check_query = "SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = $1
    )";
    
    let exists: bool = match sqlx::query_scalar(check_query)
        .bind(collection)
        .fetch_one(db.as_ref())
        .await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check if table exists: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to check if table exists: {}", e)
                }));
            }
        };
    
    if !exists {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Collection not found"
        }));
    }
    
    // Query record
    let query = format!(
        "SELECT data FROM {} WHERE id = $1",
        collection
    );
    
    match sqlx::query(&query)
        .bind(&id)
        .fetch_optional(db.as_ref())
        .await {
            Ok(Some(row)) => {
                let data: serde_json::Value = row.get("data");
                let response = RecordResponse {
                    id,
                    data,
                };
                HttpResponse::Ok().json(response)
            },
            Ok(None) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Record not found"
                }))
            },
            Err(e) => {
                tracing::error!("Failed to get record: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to get record: {}", e)
                }))
            }
        }
}

// Update record endpoint
async fn update_record(
    db: web::Data<Arc<sqlx::PgPool>>,
    path: web::Path<String>,
    record_req: web::Json<RecordRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let collection = &record_req.collection;
    let data = &record_req.data;
    
    // Check if table exists
    let check_query = "SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = $1
    )";
    
    let exists: bool = match sqlx::query_scalar(check_query)
        .bind(collection)
        .fetch_one(db.as_ref())
        .await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check if table exists: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to check if table exists: {}", e)
                }));
            }
        };
    
    if !exists {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Collection not found"
        }));
    }
    
    // Update record
    let update_query = format!(
        "UPDATE {} SET data = $1 WHERE id = $2",
        collection
    );
    
    match sqlx::query(&update_query)
        .bind(data)
        .bind(&id)
        .execute(db.as_ref())
        .await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    let response = RecordResponse {
                        id,
                        data: data.clone(),
                    };
                    HttpResponse::Ok().json(response)
                } else {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Record not found"
                    }))
                }
            },
            Err(e) => {
                tracing::error!("Failed to update record: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to update record: {}", e)
                }))
            }
        }
}

// Delete record endpoint
async fn delete_record(
    db: web::Data<Arc<sqlx::PgPool>>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let id = path.into_inner();
    
    // Check if collection is specified
    let collection = match query.get("collection") {
        Some(col) => col,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Collection name is required"
        })),
    };
    
    // Check if table exists
    let check_query = "SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = $1
    )";
    
    let exists: bool = match sqlx::query_scalar(check_query)
        .bind(collection)
        .fetch_one(db.as_ref())
        .await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check if table exists: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to check if table exists: {}", e)
                }));
            }
        };
    
    if !exists {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Collection not found"
        }));
    }
    
    // Delete record
    let delete_query = format!(
        "DELETE FROM {} WHERE id = $1",
        collection
    );
    
    match sqlx::query(&delete_query)
        .bind(&id)
        .execute(db.as_ref())
        .await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    HttpResponse::NoContent().finish()
                } else {
                    HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Record not found"
                    }))
                }
            },
            Err(e) => {
                tracing::error!("Failed to delete record: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to delete record: {}", e)
                }))
            }
        }
}

#[derive(Debug, Deserialize)]
struct QueryRequest {
    collection: String,
    query: serde_json::Value,
    limit: Option<u64>,
    offset: Option<u64>,
    sort_by: Option<String>,
    sort_order: Option<String>,
}

// Query records endpoint
async fn query_records(
    db: web::Data<Arc<sqlx::PgPool>>,
    query_req: web::Json<QueryRequest>,
) -> impl Responder {
    let collection = &query_req.collection;
    
    // Check if table exists
    let check_query = "SELECT EXISTS (
        SELECT FROM information_schema.tables 
        WHERE table_schema = 'public' 
        AND table_name = $1
    )";
    
    let exists: bool = match sqlx::query_scalar(check_query)
        .bind(collection)
        .fetch_one(db.as_ref())
        .await {
            Ok(exists) => exists,
            Err(e) => {
                tracing::error!("Failed to check if table exists: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to check if table exists: {}", e)
                }));
            }
        };
    
    if !exists {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Collection not found"
        }));
    }
    
    // Build the query
    let mut sql_query = format!(
        "SELECT id, data FROM {}",
        collection
    );
    
    // Add filtering
    if !query_req.query.is_null() && !query_req.query.as_object().map_or(true, |obj| obj.is_empty()) {
        sql_query.push_str(" WHERE ");
            
        if let Some(obj) = query_req.query.as_object() {
            let mut first = true;
            for (key, value) in obj {
                if !first {
                    sql_query.push_str(" AND ");
                }
                    
                // Convert to proper SQL condition
                match value {
                    serde_json::Value::Null => {
                        sql_query.push_str(&format!("data->>'{}' IS NULL", key));
                    },
                    serde_json::Value::String(s) => {
                        sql_query.push_str(&format!("data->>'{}' = '{}'", key, s.replace('\'', "''")));
                    },
                    serde_json::Value::Number(n) => {
                        sql_query.push_str(&format!("(data->'{}')::numeric = {}", key, n));
                    },
                    serde_json::Value::Bool(b) => {
                        sql_query.push_str(&format!("(data->'{}')::boolean = {}", key, b));
                    },
                    _ => {
                        sql_query.push_str(&format!("data->>'{}' = '{}'", key, value.to_string().replace('\'', "''")));
                    },
                }
                    
                first = false;
            }
        }
    }
    
    // Add sorting
    if let Some(sort_by) = &query_req.sort_by {
        let direction = query_req.sort_order.as_deref().unwrap_or("asc");
        sql_query.push_str(&format!(" ORDER BY data->>'{0}' {1}", sort_by, direction));
    }
    
    // Add pagination
    if let Some(limit) = query_req.limit {
        sql_query.push_str(&format!(" LIMIT {}", limit));
            
        if let Some(offset) = query_req.offset {
            sql_query.push_str(&format!(" OFFSET {}", offset));
        }
    }
    
    // Execute the query
    match sqlx::query(&sql_query)
        .fetch_all(db.as_ref())
        .await {
            Ok(rows) => {
                let records: Vec<RecordResponse> = rows.iter().map(|row| {
                    RecordResponse {
                        id: row.get("id"),
                        data: row.get("data"),
                    }
                }).collect();
                
                HttpResponse::Ok().json(records)
            },
            Err(e) => {
                tracing::error!("Failed to execute query: {:?}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to execute query: {}", e)
                }))
            }
        }
}