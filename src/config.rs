use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Read;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub name: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_string: Option<String>,
    pub pool_size: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub databases: Vec<DatabaseConfig>,
    pub server: ServerConfig,
    pub log_level: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // Try to load from config file first
        let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| "config/default.toml".to_string());
        
        let mut config = match File::open(&config_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                toml::from_str::<AppConfig>(&contents)?
            },
            Err(_) => {
                // If config file doesn't exist, use default configuration
                tracing::warn!("Configuration file not found, using default configuration");
                Self::default()
            }
        };

        // Override with environment variables if they exist
        Self::override_from_env(&mut config)?;

        Ok(config)
    }

    fn override_from_env(config: &mut AppConfig) -> Result<()> {
        // Check for database URLs from environment
        if let Ok(url) = env::var("DATABASE_URL") {
            // Add or update PostgreSQL configuration
            let postgres_config = DatabaseConfig {
                db_type: "postgresql".to_string(),
                name: "postgres".to_string(),
                host: env::var("PGHOST").ok(),
                port: env::var("PGPORT").ok().and_then(|p| p.parse().ok()),
                username: env::var("PGUSER").ok(),
                password: env::var("PGPASSWORD").ok(),
                connection_string: Some(url),
                pool_size: Some(10),
                timeout_seconds: Some(30),
            };

            // Update or add the PostgreSQL config
            if let Some(idx) = config.databases.iter().position(|db| db.db_type == "postgresql") {
                config.databases[idx] = postgres_config;
            } else {
                config.databases.push(postgres_config);
            }
        }

        // Add other environment variable overrides here for other database types
        // Example for MongoDB:
        if let Ok(mongo_url) = env::var("MONGODB_URI") {
            let mongo_config = DatabaseConfig {
                db_type: "mongodb".to_string(),
                name: "mongodb".to_string(),
                connection_string: Some(mongo_url),
                host: env::var("MONGODB_HOST").ok(),
                port: env::var("MONGODB_PORT").ok().and_then(|p| p.parse().ok()),
                username: env::var("MONGODB_USER").ok(),
                password: env::var("MONGODB_PASSWORD").ok(),
                pool_size: Some(10),
                timeout_seconds: Some(30),
            };

            if let Some(idx) = config.databases.iter().position(|db| db.db_type == "mongodb") {
                config.databases[idx] = mongo_config;
            } else {
                config.databases.push(mongo_config);
            }
        }

        // Could continue with other database types...

        Ok(())
    }

    pub fn default() -> Self {
        AppConfig {
            databases: vec![
                DatabaseConfig {
                    db_type: "postgresql".to_string(),
                    name: "postgres".to_string(),
                    host: Some("localhost".to_string()),
                    port: Some(5432),
                    username: Some("postgres".to_string()),
                    password: Some("postgres".to_string()),
                    connection_string: None,
                    pool_size: Some(10),
                    timeout_seconds: Some(30),
                },
                DatabaseConfig {
                    db_type: "mongodb".to_string(),
                    name: "mongodb".to_string(),
                    host: Some("localhost".to_string()),
                    port: Some(27017),
                    username: None,
                    password: None,
                    connection_string: Some("mongodb://localhost:27017".to_string()),
                    pool_size: Some(10),
                    timeout_seconds: Some(30),
                },
                // Add default configs for other database types as needed
            ],
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
                workers: Some(num_cpus::get()),
            },
            log_level: Some("info".to_string()),
        }
    }
}
