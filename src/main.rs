mod api;
mod config;
mod database;
mod error;
mod schema;

use actix_web::{App, HttpServer, middleware, web};
use dotenv::dotenv;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::routes;
use crate::config::AppConfig;
use crate::database::connection_pool::ConnectionManager;
use crate::error::Result;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::handlers::get_record,
        api::handlers::create_record,
        api::handlers::update_record,
        api::handlers::delete_record,
        api::handlers::query_records,
        api::handlers::get_database_info,
        api::handlers::execute_transaction,
    ),
    components(
        schemas(
            api::handlers::RecordRequest,
            api::handlers::RecordResponse,
            api::handlers::QueryRequest,
            api::handlers::DatabaseInfoResponse,
            api::handlers::TransactionRequest,
        )
    ),
    tags(
        (name = "unified_database_api", description = "Unified Database API Operations")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!("Application configuration loaded");

    // Initialize database connection pools
    let connection_manager = ConnectionManager::new(&config).await?;
    let connection_manager = Arc::new(connection_manager);
    tracing::info!("Database connection pools initialized");

    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Start HTTP server
    tracing::info!("Starting HTTP server at 0.0.0.0:8000");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(connection_manager.clone()))
            .app_data(web::JsonConfig::default().limit(4096 * 1024)) // 4MB
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await?;

    Ok(())
}
