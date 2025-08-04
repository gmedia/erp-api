use fake::{
    Fake,
    faker::{
        lorem::en::{Sentence, Word},
    },
};
use reqwest::Client as HttpClient;
use serde_json::json;

use api::v1::inventory::models::InventoryItem;
mod common;
use common::{setup_test_app, setup_test_app_with_meili_error, get_auth_token};
use sea_orm::{ConnectionTrait, Statement};
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn test_create_and_search_inventory() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

#[tokio::test(flavor = "multi_thread")]
async fn test_create_inventory_negative_quantity() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

#[tokio::test(flavor = "multi_thread")]
async fn test_create_inventory_negative_price() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

#[tokio::test(flavor = "multi_thread")]
async fn test_update_inventory() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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
            "{}/v1/inventory/update/{}",
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

#[tokio::test(flavor = "multi_thread")]
async fn test_update_inventory_negative_quantity() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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
            "{}/v1/inventory/update/{}",
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

#[tokio::test(flavor = "multi_thread")]
async fn test_update_inventory_negative_price() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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
            "{}/v1/inventory/update/{}",
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

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_inventory() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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
            "{}/v1/inventory/delete/{}",
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
            "{}/v1/inventory/delete/{}",
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

#[tokio::test(flavor = "multi_thread")]
async fn test_create_item_internal_server_error() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

#[tokio::test(flavor = "multi_thread")]
async fn test_update_item_internal_server_error() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    let item_id = "some-random-id";
    let updated_data = json!({
        "name": "some new name"
    });

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let response = client
        .put(format!("{server_url}/v1/inventory/update/{item_id}"))
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

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_item_internal_server_error() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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
    let _ = db_pool.close().await;

    // Try to delete the item, this should fail on the delete operation
    let response = client
        .delete(format!(
            "{}/v1/inventory/delete/{}",
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

#[tokio::test(flavor = "multi_thread")]
async fn test_search_items_internal_server_error() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app_with_meili_error().await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

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

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_item_with_fk_constraint_fails() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create a temporary table with a foreign key to the inventory table
    // to simulate a constraint violation.
    let backend: sea_orm::DatabaseBackend = db_pool.get_database_backend();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            "CREATE TABLE order_items (id CHAR(36) PRIMARY KEY, item_id CHAR(36) NOT NULL, FOREIGN KEY (item_id) REFERENCES inventory(id))".to_string(),
        ))
        .await;

    // 1. Create an inventory item.
    let name: String = Sentence(1..3).fake();
    let new_item = json!({ "name": name, "quantity": 10, "price": 10.0 });
    let res = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let created_item: InventoryItem = res.json().await.unwrap();

    // 2. Create a record in `order_items` that references the inventory item.
    let order_item_id = Uuid::new_v4().to_string();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            format!(
                "INSERT INTO order_items (id, item_id) VALUES ('{}', '{}');",
                order_item_id, created_item.id
            ),
        ))
        .await;

    // 3. Try to delete the inventory item. This should fail due to the FK constraint.
    let res = client
        .delete(format!(
            "{}/v1/inventory/delete/{}",
            server_url, created_item.id
        ))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    // 4. Assert we get a 500 Internal Server Error.
    assert_eq!(res.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);

    // Cleanup: drop the temporary table
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            "DROP TABLE order_items;".to_string(),
        ))
        .await;

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_item_not_found() {
    let (db_pool, _meili_client, server_url, server_handle) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    let non_existent_id = Uuid::new_v4().to_string();
    let updated_data = json!({ "name": "this should fail" });

    let response = client
        .put(format!(
            "{server_url}/v1/inventory/update/{non_existent_id}"
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
