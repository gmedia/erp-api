use actix_web::web;
use super::v1::{index, contact, todo, todo_create};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index::routes::render)
        .service(contact::routes::render)
        .service(todo::routes::render)
        .service(todo_create::routes::render);
}
