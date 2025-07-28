use actix_web::web;
use super::v1::{index, contact, todo, todo_create, todo_store, foo};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index::routes::handle)
        .service(contact::routes::handle)
        .service(todo::routes::handle)
        .service(todo_create::routes::handle)
        .service(todo_store::routes::handle)
        .service(foo::routes::handle);
}
