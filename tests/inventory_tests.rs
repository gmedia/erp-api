use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use erp_api::api::v1::inventory::models::{InventoryItem};
mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_create_and_search_inventory() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();

    // Tes endpoint POST /inventory/create
    let new_item = json!({
        "name": "Laptop Test",
        "quantity": 5,
        "price": 999.99
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
    
    assert_eq!(created_item.name, "Laptop Test");
    assert_eq!(created_item.quantity, 5);
    assert_eq!(created_item.price, 999.99);

    // Tunggu Meilisearch untuk mengindeks
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Tes endpoint GET /inventory/search
    let response = client
        .get(&format!("{}/inventory/search?q=Laptop", server_url))
        .send()
        .await
        .expect("Gagal mengirim request GET");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let search_results: Vec<InventoryItem> = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "Laptop Test");
}