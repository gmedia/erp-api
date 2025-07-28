use actix_web::{web};
use inertia_rust::{InertiaService};
use super::index;
use super::contact;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index::render_index)
        .service(contact::render_contact)
        .inertia_route("/todo", "Todo/Index")
        .inertia_route("/todo/create", "Todo/Create");
}