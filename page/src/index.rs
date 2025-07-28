use actix_web::{get, Responder, HttpRequest, web};
use inertia_rust::{Inertia, InertiaFacade};

#[get("/index")]
pub async fn render_index(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Index".into()).await
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(render_index);
}