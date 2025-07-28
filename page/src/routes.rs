use actix_web::{web};
use inertia_rust::{InertiaService};
use super::index::render_index;
use super::contact::render_contact;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(render_index)
        .service(render_contact)
        .inertia_route("/todo", "Todo/Index")
        .inertia_route("/todo/create", "Todo/Create");
}