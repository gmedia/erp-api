use super::handlers;
use crate::middlewares::jwt::JwtMiddleware;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = JwtMiddleware::new("Bearer ".to_string());
    cfg.service(
        web::scope("/v1/order")
            .wrap(jwt_middleware)
            .route("", web::get().to(handlers::get_all_orders))
            .route("", web::post().to(handlers::create_order))
            .route("/{id}", web::get().to(handlers::get_order_by_id))
            .route("/{id}", web::put().to(handlers::update_order))
            .route("/{id}", web::delete().to(handlers::delete_order)),
    );
}
