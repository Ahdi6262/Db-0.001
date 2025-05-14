use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    database_connected: bool,
}

async fn health_check(db: web::Data<PgPool>) -> impl Responder {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(db.get_ref()).await {
        Ok(_) => true,
        Err(_) => false,
    };
    
    let response = HealthResponse {
        status: "ok".to_string(),
        database_connected: db_status,
    };
    
    HttpResponse::Ok().json(response)
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    id: String,
    data: serde_json::Value,
}

async fn get_tables(db: web::Data<PgPool>) -> impl Responder {
    let tables = match sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
        .fetch_all(db.get_ref())
        .await {
            Ok(rows) => {
                let tables: Vec<String> = rows.iter()
                    .map(|row| row.get::<String, _>("table_name"))
                    .collect();
                tables
            },
            Err(e) => {
                eprintln!("Error getting tables: {}", e);
                Vec::new()
            }
        };
    
    HttpResponse::Ok().json(serde_json::json!({
        "tables": tables
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    println!("Connecting to PostgreSQL...");
    
    // Create database connection pool
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await {
            Ok(pool) => {
                println!("Connected to the database");
                pool
            },
            Err(e) => {
                eprintln!("Failed to connect to the database: {}", e);
                panic!("Database connection failed");
            }
        };
    
    // Start HTTP server
    println!("Starting HTTP server on 0.0.0.0:8000");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health_check))
            .route("/tables", web::get().to(get_tables))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}