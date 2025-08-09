use super::handlers;
use crate::middlewares::jwt::JwtMiddleware;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    let jwt_middleware = JwtMiddleware::new("Bearer ".to_string());
    cfg.service(
        web::scope("/v1/order")
            .wrap(jwt_middleware)
            .route("/create", web::post().to(handlers::create_order)),
    );
}
