use actix_web::{web, HttpResponse};
use sea_orm::{EntityTrait, ModelTrait};

use crate::error::ApiError;
use entity::inventory;

#[utoipa::path(
    delete,
    path = "/v1/inventory/delete/{id}",
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
    found_item.delete(&data.db).await?;

    // Delete from Meilisearch
    let index = data.meilisearch.index("inventory");
    index.delete_document(&item_id_for_meili).await?;

    Ok(HttpResponse::Ok().finish())
}
