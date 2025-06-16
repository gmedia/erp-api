use actix_web::{web, HttpResponse, Responder};
use meilisearch_sdk::client::Client;
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};

use crate::db::entities::inventory;

#[utoipa::path(
    delete,
    path = "/inventory/delete/{id}",
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
    db: web::Data<DatabaseConnection>,
    meili_client: web::Data<Client>,
    id: web::Path<String>,
) -> impl Responder {
    let item_id = id.into_inner();
    let find_result = inventory::Entity::find_by_id(item_id.clone()).one(db.get_ref()).await;

    match find_result {
        Ok(Some(found_item)) => {
            let item_id_for_meili = found_item.id.clone();
            let delete_result = found_item.delete(db.get_ref()).await;

            match delete_result {
                Ok(_) => {
                    // Delete from Meilisearch
                    let index = meili_client.index("inventory");
                    index
                        .delete_document(&item_id_for_meili)
                        .await
                        .expect("Failed to delete from Meilisearch");
                    HttpResponse::Ok().finish()
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}