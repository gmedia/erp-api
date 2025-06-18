use actix_web::web;
use crate::v1::auth::middleware::JwtMiddleware;

use super::handlers::{
    create_item::create_item,
    delete_item::delete_item,
    search_items::search_items,
    update_item::update_item,
};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/inventory")
            .wrap(JwtMiddleware)
            .route("/create", web::post().to(create_item))
            .route("/search", web::get().to(search_items))
            .route("/update/{id}", web::put().to(update_item))
            .route("/delete/{id}", web::delete().to(delete_item)),
    );
}