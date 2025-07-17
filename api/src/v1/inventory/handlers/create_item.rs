use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use super::super::models::{CreateInventoryItem, InventoryItem};
use crate::error::ApiError;
use entity::inventory;

#[utoipa::path(
    post,
    path = "/v1/inventory/create",
    tag = "inventory",
    request_body = CreateInventoryItem,
    responses(
        (status = 200, description = "Item created successfully", body = InventoryItem),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_item(
    data: web::Data<config::app::AppState>,
    item: web::Json<CreateInventoryItem>,
) -> Result<HttpResponse, ApiError> {
    if item.quantity < 0 {
        return Err(ApiError::ValidationError(
            "Quantity cannot be negative".to_string(),
        ));
    }
    if item.price < 0.0 {
        return Err(ApiError::ValidationError(
            "Price cannot be negative".to_string(),
        ));
    }

    let new_uuid = Uuid::new_v4();
    let new_item = inventory::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(item.name.clone()),
        quantity: Set(item.quantity),
        price: Set(item.price),
    };

    let inserted_item = new_item.insert(&data.db).await?;

    // Add to Meilisearch for indexing
    let index = data.meilisearch.index("inventory");
    let item_for_meili: InventoryItem = inserted_item;

    index.add_documents(&[&item_for_meili], Some("id")).await?;

    Ok(HttpResponse::Ok().json(item_for_meili))
}
