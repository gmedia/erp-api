use actix_web::{HttpResponse, Responder};
use serde_json::json;

pub async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "active" }))
}
