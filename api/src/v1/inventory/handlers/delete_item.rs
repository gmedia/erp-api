use actix_web::{web, HttpResponse, Responder};
use sea_orm::{EntityTrait, ModelTrait};

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
) -> impl Responder {
    let item_id = id.into_inner();
    let find_result = inventory::Entity::find_by_id(item_id.clone()).one(&data.db).await;

    match find_result {
        Ok(Some(found_item)) => {
            let item_id_for_meili = found_item.id.clone();
            let delete_result = found_item.delete(&data.db).await;

            match delete_result {
                Ok(_) => {
                    // Delete from Meilisearch
                    let index = data.meilisearch.index("inventory");
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