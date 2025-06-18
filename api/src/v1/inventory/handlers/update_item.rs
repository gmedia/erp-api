use actix_web::{web, HttpResponse, Responder};
use search::Client;
use sea_orm::{ActiveModelTrait, Set, DatabaseConnection, EntityTrait, IntoActiveModel};
use serde_json;

use super::super::models::{UpdateInventoryItem, InventoryItem};
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
        (status = 404, description = "Item not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn update_item(
    db: web::Data<DatabaseConnection>,
    meili_client: web::Data<Client>,
    id: web::Path<String>,
    item: web::Json<UpdateInventoryItem>,
) -> impl Responder {
    let item_id = id.into_inner();
    let find_result = inventory::Entity::find_by_id(item_id.clone()).one(db.get_ref()).await;

    match find_result {
        Ok(Some(found_item)) => {
            let mut active_item = found_item.into_active_model();

            if let Some(name) = &item.name {
                active_item.name = Set(name.clone());
            }
            if let Some(quantity) = item.quantity {
                 if quantity < 0 {
                    return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Quantity cannot be negative" }));
                }
                active_item.quantity = Set(quantity);
            }
            if let Some(price) = item.price {
                if price < 0.0 {
                    return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Price cannot be negative" }));
                }
                active_item.price = Set(price);
            }

            let result = active_item.update(db.get_ref()).await;

            match result {
                Ok(updated_item) => {
                    let index = meili_client.index("inventory");
                    let item_for_meili: InventoryItem = updated_item.clone().into();
                    index
                        .add_documents(&[&item_for_meili], Some("id"))
                        .await
                        .expect("Failed to index in Meilisearch");
                    HttpResponse::Ok().json(updated_item)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}