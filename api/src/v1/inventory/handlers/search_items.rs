use actix_web::{web, HttpResponse, Responder};
use serde_json;

use super::super::models::InventoryItem;

#[utoipa::path(
    get,
    path = "/v1/inventory/search",
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
    data: web::Data<config::app::AppState>,
    query: web::Query<serde_json::Value>,
) -> impl Responder {
    let q = query.get("q").and_then(|v| v.as_str()).unwrap_or("");
    log::info!("Searching for: {}", q);
    let index = data.meilisearch.index("inventory");
    let search_result = index.search().with_query(q).execute::<InventoryItem>().await;

    match search_result {
        Ok(result) => {
            log::info!("Search successful, found {} hits", result.hits.len());
            let hits: Vec<_> = result.hits.into_iter().map(|hit| hit.result).collect();
            HttpResponse::Ok().json(hits)
        },
        Err(e) => {
            log::error!("Meilisearch error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}