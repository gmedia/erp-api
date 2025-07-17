use super::handlers;
use crate::v1::auth::middleware::JwtMiddleware;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/employee")
            .wrap(JwtMiddleware)
            .route("/create", web::post().to(handlers::create_employee)),
    );
}
