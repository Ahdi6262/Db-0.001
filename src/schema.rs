//! Schema definitions for the unified database API
//!
//! This module contains schema definitions and type mappings
//! for various database types supported by the API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSchema {
    /// Name of the schema
    pub name: String,
    
    /// Collections/tables defined in the schema
    pub collections: Vec<CollectionSchema>,
}

/// Represents a collection or table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    /// Name of the collection/table
    pub name: String,
    
    /// Fields defined in the collection
    pub fields: Vec<FieldSchema>,
    
    /// Indexes defined on the collection
    pub indexes: Vec<IndexSchema>,
    
    /// Database-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Represents a field in a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Name of the field
    pub name: String,
    
    /// Type of the field
    pub field_type: FieldType,
    
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    
    /// Default value for the field
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    
    /// Database-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Represents an index on a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSchema {
    /// Name of the index
    pub name: String,
    
    /// Fields included in the index
    pub fields: Vec<String>,
    
    /// Type of the index
    pub index_type: IndexType,
    
    /// Whether the index is unique
    #[serde(default)]
    pub unique: bool,
    
    /// Database-specific options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Types of fields in a schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Boolean type
    Boolean,
    
    /// Integer type
    Integer,
    
    /// Floating-point type
    Float,
    
    /// String type
    String,
    
    /// Date/time type
    DateTime,
    
    /// Binary data type
    Binary,
    
    /// Array type with element type
    Array(Box<FieldType>),
    
    /// Object type with nested fields
    Object(HashMap<String, FieldType>),
    
    /// Reference to another collection
    Reference(String),
    
    /// Vector type (for vector databases)
    Vector(usize),
    
    /// JSON type
    Json,
    
    /// Any type (schema-less)
    Any,
}

/// Types of indexes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IndexType {
    /// B-tree index (standard)
    Btree,
    
    /// Hash index
    Hash,
    
    /// Full-text search index
    Fulltext,
    
    /// Spatial index
    Spatial,
    
    /// Vector index
    Vector,
    
    /// Time series index
    TimeSeries,
    
    /// Graph index
    Graph,
    
    /// Custom index type
    Custom(String),
}

/// Helper functions for working with schemas
pub mod helpers {
    use super::*;
    
    /// Creates a SQL CREATE TABLE statement from a collection schema
    pub fn create_sql_table(schema: &CollectionSchema) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", schema.name);
        
        let fields: Vec<String> = schema.fields.iter().map(|field| {
            let sql_type = match &field.field_type {
                FieldType::Boolean => "BOOLEAN".to_string(),
                FieldType::Integer => "INTEGER".to_string(),
                FieldType::Float => "FLOAT".to_string(),
                FieldType::String => "TEXT".to_string(),
                FieldType::DateTime => "TIMESTAMP".to_string(),
                FieldType::Binary => "BYTEA".to_string(),
                FieldType::Array(_) => "JSONB".to_string(),
                FieldType::Object(_) => "JSONB".to_string(),
                FieldType::Reference(_) => "TEXT".to_string(),
                FieldType::Vector(_) => "JSONB".to_string(),
                FieldType::Json => "JSONB".to_string(),
                FieldType::Any => "JSONB".to_string(),
            };
            
            let nullable = if field.required { "NOT NULL" } else { "" };
            
            let default = match &field.default_value {
                Some(val) => format!("DEFAULT '{}'", val.to_string().replace("'", "''")),
                None => "".to_string(),
            };
            
            format!("  {} {} {} {}", field.name, sql_type, nullable, default)
        }).collect();
        
        sql.push_str(&fields.join(",\n"));
        
        // Add primary key
        if let Some(pk_index) = schema.indexes.iter().find(|idx| idx.name == "PRIMARY" || idx.name == "PRIMARY_KEY") {
            sql.push_str(",\n  PRIMARY KEY (");
            sql.push_str(&pk_index.fields.join(", "));
            sql.push_str(")");
        }
        
        sql.push_str("\n)");
        
        sql
    }
    
    /// Converts a schema field type to a MongoDB type
    pub fn field_type_to_mongodb(field_type: &FieldType) -> String {
        match field_type {
            FieldType::Boolean => "bool".to_string(),
            FieldType::Integer => "int".to_string(),
            FieldType::Float => "double".to_string(),
            FieldType::String => "string".to_string(),
            FieldType::DateTime => "date".to_string(),
            FieldType::Binary => "binData".to_string(),
            FieldType::Array(item_type) => format!("array<{}>", field_type_to_mongodb(item_type)),
            FieldType::Object(_) => "object".to_string(),
            FieldType::Reference(_) => "objectId".to_string(),
            FieldType::Vector(_) => "array".to_string(),
            FieldType::Json => "object".to_string(),
            FieldType::Any => "mixed".to_string(),
        }
    }
    
    /// Generates a Neo4j Cypher CREATE statement for a node
    pub fn create_neo4j_node(schema: &CollectionSchema) -> String {
        let properties = schema.fields.iter()
            .map(|field| format!("{}:${}", field.name, field.name))
            .collect::<Vec<_>>()
            .join(", ");
            
        format!("CREATE (n:{} {{{}}})", schema.name, properties)
    }
}
