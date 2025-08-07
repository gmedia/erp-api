use actix_web::{get, HttpRequest, Responder};
use inertia_rust::{hashmap, Inertia, InertiaFacade, InertiaProp};
use serde_json::json;

#[get("/v1/contact")]
pub async fn handle(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "user" => InertiaProp::always(json!({
            "name": "John Doe",
            "email": "johndoe@example.com"
        }))
    ];

    Inertia::render_with_props(&req, "Contact".into(), props).await
}
