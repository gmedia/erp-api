use actix_web::{web};
use inertia_rust::{InertiaService};
use super::v1::{index, contact, todo};
use super::middlewares::reflash_temporary_session::ReflashTemporarySessionMiddleware;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.wrap(ReflashTemporarySessionMiddleware::new())
        .service(index::routes::render)
        .service(contact::routes::render)
        .service(todo::routes::render)
        .inertia_route("/todo/create", "Todo/Create");
}