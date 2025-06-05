use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use meilisearch_sdk::client::Client;
use uuid::Uuid;
use serde_json;

use super::models::{CreateInventoryItem, InventoryItem};
use crate::db::mysql::DbPool;
use crate::db::schema::inventory::dsl::*;

pub async fn create_item(
    pool: web::Data<DbPool>,
    meili_client: web::Data<Client>,
    item: web::Json<CreateInventoryItem>,
) -> impl Responder {
    let new_uuid = Uuid::new_v4();
    let new_item = InventoryItem {
        id: new_uuid.to_string(),
        name: item.name.clone(),
        quantity: item.quantity,
        price: item.price,
    };

    let conn = &mut pool.get().expect("Gagal mendapatkan koneksi database");

    let result = diesel::insert_into(inventory)
        .values(&new_item)
        .execute(conn);

    match result {
        Ok(_) => {
            // Add to Meilisearch for indexing
            let index = meili_client.index("inventory");
            index
                .add_documents(&[&new_item], Some("id"))
                .await
                .expect("Failed to index in Meilisearch");
            HttpResponse::Ok().json(new_item)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

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