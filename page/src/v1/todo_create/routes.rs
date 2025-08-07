use actix_web::{get, HttpRequest, Responder};
use inertia_rust::{Inertia, InertiaFacade};

#[get("/v1/todo/create")]
pub async fn handle(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Todo/Create".into()).await
}
