use sqlx::{postgres::PgPoolOptions, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting minimal PostgreSQL test");

    // Get connection details from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to PostgreSQL at {}", database_url);
    
    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    tracing::info!("Connection successful!");
    
    // Test the connection with a simple query
    let row = sqlx::query("SELECT 1 as test")
        .fetch_one(&pool)
        .await?;
    
    let value: i32 = row.get("test");
    tracing::info!("Executed test query, result: {}", value);
    
    // Close the pool
    pool.close().await;
    tracing::info!("Test completed successfully");
    
    Ok(())
}