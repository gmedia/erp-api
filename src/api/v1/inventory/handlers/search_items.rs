use actix_web::{web, HttpResponse, Responder};
use meilisearch_sdk::client::Client;
use serde_json;

use super::super::models::InventoryItem;

#[utoipa::path(
    get,
    path = "/inventory/search",
    tag = "inventory",
    params(
        ("q" = String, Query, description = "Search query for inventory items")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<InventoryItem>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn search_items(
    meili_client: web::Data<Client>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let q = query.get("q").and_then(|v| v.as_str()).unwrap_or("");
    let index = meili_client.index("inventory");
    let search_result = index.search().with_query(q).execute::<InventoryItem>().await;

    match search_result {
        Ok(result) => {
            let hits: Vec<_> = result.hits.into_iter().map(|hit| hit.result).collect();
            HttpResponse::Ok().json(hits)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}