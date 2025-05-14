use actix_web::web;

use crate::api::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/databases")
                    .route("", web::get().to(handlers::get_database_info))
            )
            .service(
                web::scope("/databases/{db_type}/{db_name}")
                    .route("/records/{id}", web::get().to(handlers::get_record))
                    .route("/records", web::post().to(handlers::create_record))
                    .route("/records/{id}", web::put().to(handlers::update_record))
                    .route("/records/{id}", web::delete().to(handlers::delete_record))
                    .route("/query", web::post().to(handlers::query_records))
                    .route("/transactions", web::post().to(handlers::execute_transaction))
            )
    );
}
