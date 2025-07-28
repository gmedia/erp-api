use actix_web::{get, Responder, HttpRequest};
use inertia_rust::{Inertia, InertiaFacade};

#[get("/")]
pub async fn render(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Index".into()).await
}