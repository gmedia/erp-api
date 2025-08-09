use crate::middlewares::jwt::JwtMiddleware;
use actix_web::web;

use super::handlers::{
    create_item::create_item, delete_item::delete_item, search_items::search_items,
    update_item::update_item,
};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = JwtMiddleware::new("Bearer ".to_string());
    cfg.service(
        web::scope("/v1/inventory")
            .wrap(jwt_middleware)
            .route("/create", web::post().to(create_item))
            .route("/search", web::get().to(search_items))
            .route("/update/{id}", web::put().to(update_item))
            .route("/delete/{id}", web::delete().to(delete_item)),
    );
}
