use actix_web::web;
use super::handlers;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/v1/employee/create")
            .route(web::post().to(handlers::create_employee))
    );
}