use super::v1::{contact, foo, index, todo, todo_create, todo_store};
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index::routes::handle)
        .service(contact::routes::handle)
        .service(todo::routes::handle)
        .service(todo_create::routes::handle)
        .service(todo_store::routes::handle)
        .service(foo::routes::handle);
}
