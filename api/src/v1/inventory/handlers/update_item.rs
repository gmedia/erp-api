use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};

use super::super::models::{InventoryItem, UpdateInventoryItem};
use crate::error::ApiError;
use entity::inventory;

#[utoipa::path(
    put,
    path = "/v1/inventory/update/{id}",
    tag = "inventory",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    request_body = UpdateInventoryItem,
    responses(
        (status = 200, description = "Item updated successfully", body = InventoryItem),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn update_item(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
    item: web::Json<UpdateInventoryItem>,
) -> Result<HttpResponse, ApiError> {
    let item_id = id.into_inner();

    let found_item = inventory::Entity::find_by_id(item_id.clone())
        .one(&data.db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item with id {} not found", item_id)))?;

    let mut active_item = found_item.into_active_model();

    if let Some(name) = &item.name {
        active_item.name = Set(name.clone());
    }
    if let Some(quantity) = item.quantity {
        if quantity < 0 {
            return Err(ApiError::ValidationError(
                "Quantity cannot be negative".to_string(),
            ));
        }
        active_item.quantity = Set(quantity);
    }
    if let Some(price) = item.price {
        if price < 0.0 {
            return Err(ApiError::ValidationError(
                "Price cannot be negative".to_string(),
            ));
        }
        active_item.price = Set(price);
    }

    let updated_item = active_item.update(&data.db).await?;

    let index = data.meilisearch.index("inventory");
    let item_for_meili: InventoryItem = updated_item.clone();
    index.add_documents(&[&item_for_meili], Some("id")).await?;

    Ok(HttpResponse::Ok().json(updated_item))
}
