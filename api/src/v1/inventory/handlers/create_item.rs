use actix_web::{web, HttpResponse, Responder};
use search::Client;
use sea_orm::{ActiveModelTrait, Set, DatabaseConnection};
use uuid::Uuid;
use serde_json;

use super::super::models::{CreateInventoryItem, InventoryItem};
use entity::inventory;

#[utoipa::path(
    post,
    path = "/v1/inventory/create",
    tag = "inventory",
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