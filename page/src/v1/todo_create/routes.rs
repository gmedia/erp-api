use actix_web::{get, Responder, HttpRequest};
use inertia_rust::{Inertia, InertiaFacade};

#[get("/v1/todo/create")]
pub async fn render(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Todo/Create".into()).await
}