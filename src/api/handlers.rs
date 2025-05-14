use std::sync::Arc;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::connection_pool::ConnectionManager;
use crate::database::interface::{DatabaseOperation, QueryParams, Record, TransactionOperation};
use crate::database::types::Value;
use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecordRequest {
    pub collection: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecordResponse {
    pub id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QueryRequest {
    pub collection: String,
    pub query: serde_json::Value,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseInfoResponse {
    pub databases: Vec<DatabaseInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseInfo {
    pub db_type: String,
    pub name: String,
    pub status: String,
    pub collections: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionRequest {
    pub operations: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Operation {
    pub operation_type: String, // "create", "update", "delete"
    pub collection: String,
    pub id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[utoipa::path(
    get,
    path = "/api/v1/databases/{db_type}/{db_name}/records/{id}",
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name"),
        ("id" = String, Path, description = "Record ID"),
        ("collection" = String, Query, description = "Collection or table name")
    ),
    responses(
        (status = 200, description = "Record found", body = RecordResponse),
        (status = 404, description = "Record not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_record(
    path: web::Path<(String, String, String)>,
    query: web::Query<std::collections::HashMap<String, String>>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name, id) = path.into_inner();
    
    // Check if collection is specified
    let collection = match query.get("collection") {
        Some(col) => col,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Collection name is required"
        })),
    };

    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            match connection.read(collection, &id).await {
                Ok(Some(record)) => {
                    let response = RecordResponse {
                        id: record.id,
                        data: record.data,
                    };
                    HttpResponse::Ok().json(response)
                },
                Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Record not found"
                })),
                Err(e) => {
                    tracing::error!("Error retrieving record: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to retrieve record: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/databases/{db_type}/{db_name}/records",
    request_body = RecordRequest,
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name")
    ),
    responses(
        (status = 201, description = "Record created", body = RecordResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_record(
    path: web::Path<(String, String)>,
    record_req: web::Json<RecordRequest>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name) = path.into_inner();
    
    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            let id = Uuid::new_v4().to_string();
            let record = Record {
                id: id.clone(),
                data: record_req.data.clone(),
            };

            match connection.create(&record_req.collection, &record).await {
                Ok(_) => {
                    let response = RecordResponse {
                        id,
                        data: record_req.data.clone(),
                    };
                    HttpResponse::Created().json(response)
                },
                Err(e) => {
                    tracing::error!("Error creating record: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to create record: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/databases/{db_type}/{db_name}/records/{id}",
    request_body = RecordRequest,
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name"),
        ("id" = String, Path, description = "Record ID")
    ),
    responses(
        (status = 200, description = "Record updated", body = RecordResponse),
        (status = 404, description = "Record not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_record(
    path: web::Path<(String, String, String)>,
    record_req: web::Json<RecordRequest>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name, id) = path.into_inner();
    
    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            let record = Record {
                id: id.clone(),
                data: record_req.data.clone(),
            };

            match connection.update(&record_req.collection, &record).await {
                Ok(updated) => {
                    if updated {
                        let response = RecordResponse {
                            id,
                            data: record_req.data.clone(),
                        };
                        HttpResponse::Ok().json(response)
                    } else {
                        HttpResponse::NotFound().json(serde_json::json!({
                            "error": "Record not found"
                        }))
                    }
                },
                Err(e) => {
                    tracing::error!("Error updating record: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to update record: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/databases/{db_type}/{db_name}/records/{id}",
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name"),
        ("id" = String, Path, description = "Record ID"),
        ("collection" = String, Query, description = "Collection or table name")
    ),
    responses(
        (status = 204, description = "Record deleted"),
        (status = 404, description = "Record not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_record(
    path: web::Path<(String, String, String)>,
    query: web::Query<std::collections::HashMap<String, String>>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name, id) = path.into_inner();
    
    // Check if collection is specified
    let collection = match query.get("collection") {
        Some(col) => col,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Collection name is required"
        })),
    };

    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            match connection.delete(collection, &id).await {
                Ok(deleted) => {
                    if deleted {
                        HttpResponse::NoContent().finish()
                    } else {
                        HttpResponse::NotFound().json(serde_json::json!({
                            "error": "Record not found"
                        }))
                    }
                },
                Err(e) => {
                    tracing::error!("Error deleting record: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to delete record: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/databases/{db_type}/{db_name}/query",
    request_body = QueryRequest,
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name")
    ),
    responses(
        (status = 200, description = "Query executed successfully", body = Vec<RecordResponse>),
        (status = 400, description = "Invalid query"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn query_records(
    path: web::Path<(String, String)>,
    query_req: web::Json<QueryRequest>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name) = path.into_inner();
    
    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            let params = QueryParams {
                query: query_req.query.clone(),
                limit: query_req.limit,
                offset: query_req.offset,
                sort_by: query_req.sort_by.clone(),
                sort_order: query_req.sort_order.clone(),
            };

            match connection.query(&query_req.collection, &params).await {
                Ok(records) => {
                    let response: Vec<RecordResponse> = records
                        .into_iter()
                        .map(|record| RecordResponse {
                            id: record.id,
                            data: record.data,
                        })
                        .collect();
                    HttpResponse::Ok().json(response)
                },
                Err(e) => {
                    tracing::error!("Error querying records: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to query records: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/databases",
    responses(
        (status = 200, description = "Database information retrieved", body = DatabaseInfoResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_database_info(
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    match connection_manager.get_database_info().await {
        Ok(db_infos) => {
            let response = DatabaseInfoResponse {
                databases: db_infos.into_iter().map(|info| DatabaseInfo {
                    db_type: info.db_type,
                    name: info.name,
                    status: info.status,
                    collections: info.collections,
                }).collect(),
            };
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            tracing::error!("Error retrieving database info: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve database information: {}", e)
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/databases/{db_type}/{db_name}/transactions",
    request_body = TransactionRequest,
    params(
        ("db_type" = String, Path, description = "Database type"),
        ("db_name" = String, Path, description = "Database name")
    ),
    responses(
        (status = 200, description = "Transaction executed successfully"),
        (status = 400, description = "Invalid transaction"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn execute_transaction(
    path: web::Path<(String, String)>,
    transaction_req: web::Json<TransactionRequest>,
    connection_manager: web::Data<Arc<ConnectionManager>>,
) -> impl Responder {
    let (db_type, db_name) = path.into_inner();
    
    match connection_manager.get_connection(&db_type, &db_name).await {
        Ok(connection) => {
            // Convert Operations to TransactionOperations
            let operations: Vec<TransactionOperation> = transaction_req.operations
                .iter()
                .map(|op| {
                    match op.operation_type.as_str() {
                        "create" => {
                            if let (Some(data), Some(id)) = (&op.data, &op.id) {
                                TransactionOperation::Create {
                                    collection: op.collection.clone(),
                                    record: Record {
                                        id: id.clone(),
                                        data: data.clone(),
                                    },
                                }
                            } else if let Some(data) = &op.data {
                                // Generate ID if not provided
                                TransactionOperation::Create {
                                    collection: op.collection.clone(),
                                    record: Record {
                                        id: Uuid::new_v4().to_string(),
                                        data: data.clone(),
                                    },
                                }
                            } else {
                                TransactionOperation::Invalid(
                                    "Create operation requires data".to_string()
                                )
                            }
                        },
                        "update" => {
                            if let (Some(data), Some(id)) = (&op.data, &op.id) {
                                TransactionOperation::Update {
                                    collection: op.collection.clone(),
                                    record: Record {
                                        id: id.clone(),
                                        data: data.clone(),
                                    },
                                }
                            } else {
                                TransactionOperation::Invalid(
                                    "Update operation requires id and data".to_string()
                                )
                            }
                        },
                        "delete" => {
                            if let Some(id) = &op.id {
                                TransactionOperation::Delete {
                                    collection: op.collection.clone(),
                                    id: id.clone(),
                                }
                            } else {
                                TransactionOperation::Invalid(
                                    "Delete operation requires id".to_string()
                                )
                            }
                        },
                        _ => TransactionOperation::Invalid(
                            format!("Unknown operation type: {}", op.operation_type)
                        ),
                    }
                })
                .collect();

            // Check if there are any invalid operations
            let invalid_ops: Vec<String> = operations.iter()
                .filter_map(|op| {
                    if let TransactionOperation::Invalid(msg) = op {
                        Some(msg.clone())
                    } else {
                        None
                    }
                })
                .collect();

            if !invalid_ops.is_empty() {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid operations in transaction",
                    "details": invalid_ops
                }));
            }

            // Execute transaction
            match connection.execute_transaction(operations).await {
                Ok(_) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "message": "Transaction executed successfully"
                    }))
                },
                Err(e) => {
                    tracing::error!("Error executing transaction: {:?}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to execute transaction: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error connecting to database: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database connection error: {}", e)
            }))
        }
    }
}
