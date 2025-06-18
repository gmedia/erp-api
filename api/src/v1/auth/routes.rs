use actix_web::web;
use crate::v1::auth::handlers;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(handlers::register))
            .route("/login", web::post().to(handlers::login)),
    );
}