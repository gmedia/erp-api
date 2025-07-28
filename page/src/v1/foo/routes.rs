use actix_web::{get, Responder, HttpRequest};
use inertia_rust::{Inertia, InertiaFacade};

#[get("/v1/foo")]
pub async fn handle(req: HttpRequest) -> impl Responder {
    Inertia::render(&req, "Foo/Index".into()).await
}