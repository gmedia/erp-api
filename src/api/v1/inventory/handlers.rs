use actix_web::{web, HttpResponse, Responder};
use meilisearch_sdk::client::Client;
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;
use serde_json;

use super::models::{CreateInventoryItem, InventoryItem};
use crate::db::entities::inventory;
use sea_orm::DatabaseConnection;

#[utoipa::path(
    post,
    path = "/inventory/create",
    request_body = CreateInventoryItem,
    responses(
        (status = 200, description = "Item created successfully", body = InventoryItem),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_item(
    db: web::Data<DatabaseConnection>,
    meili_client: web::Data<Client>,
    item: web::Json<CreateInventoryItem>,
) -> impl Responder {
    if item.quantity < 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quantity cannot be negative" }));
    }
    if item.price < 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Price cannot be negative" }));
    }

    let new_uuid = Uuid::new_v4();
    let new_item = inventory::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(item.name.clone()),
        quantity: Set(item.quantity),
        price: Set(item.price),
    };

    let result = new_item.insert(db.get_ref()).await;

    match result {
        Ok(inserted_item) => {
            // Add to Meilisearch for indexing
            let index = meili_client.index("inventory");
            let item_for_meili: InventoryItem = inserted_item.into();
            index
                .add_documents(&[&item_for_meili], Some("id"))
                .await
                .expect("Failed to index in Meilisearch");
            HttpResponse::Ok().json(item_for_meili)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[utoipa::path(
    get,
    path = "/inventory/search",
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