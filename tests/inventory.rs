use fake::{
    faker::{
        lorem::en::{Sentence, Word},
    },
    Fake,
};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use api::v1::inventory::models::InventoryItem;
mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_create_and_search_inventory() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();
    let search_query: String = Word().fake();

    // Tes endpoint POST /inventory/create
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    assert_eq!(created_item.name, name);
    assert_eq!(created_item.quantity, quantity);
    assert!((created_item.price - price).abs() < 1e-9);

    // Tunggu Meilisearch untuk mengindeks
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Tes endpoint GET /inventory/search
    let response = client
        .get(&format!(
            "{}/inventory/search?q={}",
            server_url, search_query
        ))
        .send()
        .await
        .expect("Gagal mengirim request GET");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let _search_results: Vec<InventoryItem> =
        response.json().await.expect("Gagal parse response JSON");

}

#[tokio::test]
#[serial]
async fn test_create_inventory_negative_quantity() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let price: f64 = (1.0..1000.0).fake();

    let new_item = json!({
        "name": name,
        "quantity": -5,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_create_inventory_negative_price() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();

    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": -10.0
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}
#[tokio::test]
#[serial]
async fn test_update_inventory() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Buat item baru untuk diupdate
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    // Update item
    let updated_name: String = Sentence(1..3).fake();
    let updated_quantity: i32 = (1..100).fake();
    let updated_price: f64 = (1.0..1000.0).fake();
    let updated_data = json!({
        "name": updated_name,
        "quantity": updated_quantity,
        "price": updated_price
    });

    let response = client
        .put(&format!(
            "{}/inventory/update/{}",
            server_url, created_item.id
        ))
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_item: InventoryItem = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    assert_eq!(updated_item.name, updated_name);
    assert_eq!(updated_item.quantity, updated_quantity);
    assert!((updated_item.price - updated_price).abs() < 1e-9);
}

#[tokio::test]
#[serial]
async fn test_update_inventory_negative_quantity() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Buat item baru
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");
    let created_item: InventoryItem = response.json().await.unwrap();

    // Coba update dengan kuantitas negatif
    let updated_data = json!({
        "quantity": -5
    });

    let response = client
        .put(&format!(
            "{}/inventory/update/{}",
            server_url, created_item.id
        ))
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_update_inventory_negative_price() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Buat item baru
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");
    let created_item: InventoryItem = response.json().await.unwrap();

    // Coba update dengan harga negatif
    let updated_data = json!({
        "price": -10.0
    });

    let response = client
        .put(&format!(
            "{}/inventory/update/{}",
            server_url, created_item.id
        ))
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_delete_inventory() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Buat item baru untuk dihapus
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    // Hapus item
    let response = client
        .delete(&format!(
            "{}/inventory/delete/{}",
            server_url, created_item.id
        ))
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Coba hapus lagi, harusnya 404 Not Found
    let response = client
        .delete(&format!(
            "{}/inventory/delete/{}",
            server_url, created_item.id
        ))
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}
#[tokio::test]
#[serial]
async fn test_create_item_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(&format!("{}/inventory/create", server_url))
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}
#[tokio::test]
#[serial]
async fn test_update_item_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let item_id = "some-random-id";
    let updated_data = json!({
        "name": "some new name"
    });

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let response = client
        .put(&format!("{}/inventory/update/{}", server_url, item_id))
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
#[serial]
async fn test_delete_item_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let item_id = "some-random-id";

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let response = client
        .delete(&format!("{}/inventory/delete/{}", server_url, item_id))
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}
#[tokio::test]
#[serial]
async fn test_search_items_internal_server_error() {
    // The search_items handler only interacts with Meilisearch.
    // A real failure would be Meilisearch being down.
    // We can't easily simulate that here.
    // However, the test server setup might be affected by the db connection
    // being closed, which could lead to a server error.
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let _ = db_pool.close().await;
    let client = HttpClient::new();
    let response = client
        .get(&format!("{}/inventory/search?q=test", server_url))
        .send()
        .await
        .expect("Gagal mengirim request GET");

    // This is an indirect way to test this, but it's the best we can do
    // without more complex mocking. We expect the server to fail to respond
    // correctly if its database dependency is unavailable, even if this
    // specific endpoint doesn't use the DB directly.
    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}