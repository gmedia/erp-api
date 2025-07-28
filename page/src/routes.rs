use actix_web::{web};
use inertia_rust::{InertiaService};
use super::index;
use super::contact;
use super::todo;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index::routes::render)
        .service(contact::routes::render)
        .service(todo::routes::render)
        .inertia_route("/todo/create", "Todo/Create");
}