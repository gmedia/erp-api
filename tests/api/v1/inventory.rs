use fake::{
    Fake,
    faker::lorem::en::{Sentence, Word},
};
use reqwest::Client as HttpClient;
use serde_json::json;

use api::v1::inventory::models::InventoryItem;

use crate::helper::{get_auth_token, TestAppBuilder};
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_search() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();
    let search_query: String = Word().fake();

    // Tes endpoint POST /v1/inventory/create
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response.json().await.expect("Gagal parse response JSON");

    assert_eq!(created_item.name, name);
    assert_eq!(created_item.quantity, quantity);
    assert!((created_item.price - price).abs() < 1e-9);

    // Tunggu Meilisearch untuk indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Tes endpoint GET /v1/inventory/search
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q={search_query}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Gagal mengirim request GET");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let _search_results: Vec<InventoryItem> =
        response.json().await.expect("Gagal parse response JSON");

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_create_negative_quantity() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let price: f64 = (1.0..1000.0).fake();

    let new_item = json!({
        "name": name,
        "quantity": -5,
        "price": price
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token)
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_create_negative_price() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();

    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": -10.0
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token)
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Buat item baru untuk diubah
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response.json().await.expect("Gagal parse response JSON");

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
        .put(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_item: InventoryItem = response.json().await.expect("Gagal parse response JSON");

    assert_eq!(updated_item.name, updated_name);
    assert_eq!(updated_item.quantity, updated_quantity);
    assert!((updated_item.price - updated_price).abs() < 1e-9);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_negative_quantity() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
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
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
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
        .put(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_negative_price() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
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
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
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
        .put(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
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
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_item: InventoryItem = response.json().await.expect("Gagal parse response JSON");

    // Hapus item
    let response = client
        .delete(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token.clone())
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Coba hapus lagi, harusnya 404 Not Found
    let response = client
        .delete(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_create_internal_server_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Simulate database connection error by closing the pool
    let _ = app.db.close().await;

    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token)
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_internal_server_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let item_id = "some-random-id";
    let updated_data = json!({
        "name": "some new name"
    });

    // Simulate database connection error by closing the pool
    let _ = app.db.close().await;

    let response = client
        .put(format!("{server_url}/v1/inventory/{item_id}"))
        .bearer_auth(token)
        .json(&updated_data)
        .send()
        .await
        .expect("Gagal mengirim request PUT");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete_internal_server_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Sentence(1..3).fake();
    let quantity: i32 = (1..100).fake();
    let price: f64 = (1.0..1000.0).fake();

    // Create a new item to delete
    let new_item = json!({
        "name": name,
        "quantity": quantity,
        "price": price
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Gagal mengirim request POST");
    let created_item: InventoryItem = response.json().await.unwrap();

    // Simulate database connection error by closing the pool
    let _ = app.db.close().await;

    // Try to delete the item, this should fail on the delete operation
    let response = client
        .delete(format!(
            "{}/v1/inventory/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .send()
        .await
        .expect("Gagal mengirim request DELETE");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_search_internal_server_error() {
    let app = TestAppBuilder::new()
        .meili_host("http://localhost:9999".to_string())
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;
    
    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;

    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_not_found() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let non_existent_id = Uuid::new_v4().to_string();
    let updated_data = json!({ "name": "this should fail" });

    let response = client
        .put(format!(
            "{server_url}/v1/inventory/{non_existent_id}"
        ))
        .bearer_auth(token)
        .json(&updated_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
