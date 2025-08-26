use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryOrder, Set};
use serde::Deserialize;

use super::models::{CreateInventoryItem, InventoryItem, UpdateInventoryItem};
use crate::error::ApiError;
use entity::inventory;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

/// Create a new inventory item
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
    // Validation
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

    let new_uuid = uuid::Uuid::new_v4();
    let new_item = inventory::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(item.name.clone()),
        quantity: Set(item.quantity),
        price: Set(item.price),
    };

    let inserted_item = new_item.insert(&data.db).await?;

    // Add to Meilisearch for indexing
    let index = data.meilisearch.index("inventory");
    let item_for_meili: InventoryItem = inserted_item.clone().into();

    index.add_documents(&[&item_for_meili], Some("id")).await?;

    Ok(HttpResponse::Ok().json(item_for_meili))
}

/// Search inventory items
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

/// Get all inventory items
#[utoipa::path(
    get,
    path = "/v1/inventory",
    tag = "inventory",
    responses(
        (status = 200, description = "List of inventory items", body = Vec<InventoryItem>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_all_items(
    data: web::Data<config::app::AppState>,
) -> Result<HttpResponse, ApiError> {
    let items = inventory::Entity::find()
        .order_by_asc(entity::inventory::Column::Name)
        .all(&data.db)
        .await?;

    let item_responses: Vec<InventoryItem> = items
        .into_iter()
        .map(|item| item.into())
        .collect();

    Ok(HttpResponse::Ok().json(item_responses))
}

/// Get inventory item by ID
#[utoipa::path(
    get,
    path = "/v1/inventory/{id}",
    tag = "inventory",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item found", body = InventoryItem),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_item_by_id(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let item = inventory::Entity::find_by_id(id.into_inner())
        .one(&data.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Item not found".to_string()))?;

    let item_response: InventoryItem = item.into();
    Ok(HttpResponse::Ok().json(item_response))
}

/// Update inventory item
#[utoipa::path(
    put,
    path = "/v1/inventory/{id}",
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
    let item_for_meili: InventoryItem = updated_item.clone().into();
    index.add_documents(&[&item_for_meili], Some("id")).await?;

    Ok(HttpResponse::Ok().json(updated_item))
}

/// Delete inventory item
#[utoipa::path(
    delete,
    path = "/v1/inventory/{id}",
    tag = "inventory",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item deleted successfully"),
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn delete_item(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let item_id = id.into_inner();

    let found_item = inventory::Entity::find_by_id(item_id.clone())
        .one(&data.db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item with id {} not found", item_id)))?;

    let item_id_for_meili = found_item.id.clone();
    let active_item: inventory::ActiveModel = found_item.into();
    active_item.delete(&data.db).await?;

    // Delete from Meilisearch
    let index = data.meilisearch.index("inventory");
    index.delete_document(&item_id_for_meili).await?;

    Ok(HttpResponse::Ok().finish())
}