use actix_web::{web, HttpResponse};
use serde::Deserialize;

use super::super::models::InventoryItem;
use crate::error::ApiError;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

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
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.q.as_deref().unwrap_or("");
    log::info!("Searching for: {}", q);

    let index = data.meilisearch.index("inventory");
    let result = index
        .search()
        .with_query(q)
        .execute::<InventoryItem>()
        .await?;

    log::info!("Search successful, found {} hits", result.hits.len());
    let hits: Vec<_> = result.hits.into_iter().map(|hit| hit.result).collect();

    Ok(HttpResponse::Ok().json(hits))
}
