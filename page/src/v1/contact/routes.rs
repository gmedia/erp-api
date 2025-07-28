use actix_web::{get, Responder, HttpRequest};
use inertia_rust::{
    hashmap, Inertia, InertiaFacade, InertiaProp,
};
use serde_json::{json};

#[get("/v1/contact")]
pub async fn render(req: HttpRequest) -> impl Responder {
    let props = hashmap![
        "user" => InertiaProp::always(json!({
            "name": "John Doe",
            "email": "johndoe@example.com"
        }))
    ];

    Inertia::render_with_props(&req, "Contact".into(), props).await
}
