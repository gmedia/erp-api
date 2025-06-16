use actix_web::web;

use super::handlers::{create_item, search_items, update_item};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inventory")
            .route("/create", web::post().to(create_item))
            .route("/search", web::get().to(search_items))
            .route("/update/{id}", web::put().to(update_item)),
    );
}