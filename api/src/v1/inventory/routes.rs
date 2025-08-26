use crate::middlewares::jwt::JwtMiddleware;
use actix_web::web;

use super::handlers::{
    create_item, delete_item, get_all_items, get_item_by_id, search_items, update_item,
};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = JwtMiddleware::new("Bearer ".to_string());
    cfg.service(
        web::scope("/v1/inventory")
            .wrap(jwt_middleware)
            .route("", web::get().to(get_all_items))
            .route("/create", web::post().to(create_item))
            .route("/search", web::get().to(search_items))
            .route("/{id}", web::get().to(get_item_by_id))
            .route("/{id}", web::put().to(update_item))
            .route("/{id}", web::delete().to(delete_item)),
    );
}
