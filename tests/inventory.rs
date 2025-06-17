use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use api::v1::inventory::models::{InventoryItem};
mod common;
use common::run_test;

#[tokio::test]
#[serial]
async fn test_create_and_search_inventory() {
    run_test(|app| async move {
        let client = HttpClient::new();

        // Tes endpoint POST /inventory/create
        let new_item = json!({
            "name": "Laptop Test",
            "quantity": 5,
            "price": 999.99
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
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
            .get(&format!("{}/inventory/search?q=Laptop", app.server_url))
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
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_create_inventory_negative_quantity() {
    run_test(|app| async move {
        let client = HttpClient::new();

        let new_item = json!({
            "name": "Invalid Item",
            "quantity": -5,
            "price": 10.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
            .json(&new_item)
            .send()
            .await
            .expect("Gagal mengirim request POST");

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_create_inventory_negative_price() {
    run_test(|app| async move {
        let client = HttpClient::new();

        let new_item = json!({
            "name": "Invalid Item 2",
            "quantity": 5,
            "price": -10.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
            .json(&new_item)
            .send()
            .await
            .expect("Gagal mengirim request POST");

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    })
    .await;
}
#[tokio::test]
#[serial]
async fn test_update_inventory() {
    run_test(|app| async move {
        let client = HttpClient::new();

        // Buat item baru untuk diupdate
        let new_item = json!({
            "name": "Item to Update",
            "quantity": 10,
            "price": 20.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
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
        let updated_data = json!({
            "name": "Updated Item",
            "quantity": 15,
            "price": 25.5
        });

        let response = client
            .put(&format!(
                "{}/inventory/update/{}",
                app.server_url, created_item.id
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

        assert_eq!(updated_item.name, "Updated Item");
        assert_eq!(updated_item.quantity, 15);
        assert_eq!(updated_item.price, 25.5);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_update_inventory_negative_quantity() {
    run_test(|app| async move {
        let client = HttpClient::new();

        // Buat item baru
        let new_item = json!({
            "name": "Test Item",
            "quantity": 10,
            "price": 10.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
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
                app.server_url, created_item.id
            ))
            .json(&updated_data)
            .send()
            .await
            .expect("Gagal mengirim request PUT");

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_update_inventory_negative_price() {
    run_test(|app| async move {
        let client = HttpClient::new();

        // Buat item baru
        let new_item = json!({
            "name": "Test Item 2",
            "quantity": 10,
            "price": 10.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
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
                app.server_url, created_item.id
            ))
            .json(&updated_data)
            .send()
            .await
            .expect("Gagal mengirim request PUT");

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_delete_inventory() {
    run_test(|app| async move {
        let client = HttpClient::new();

        // Buat item baru untuk dihapus
        let new_item = json!({
            "name": "Item to Delete",
            "quantity": 5,
            "price": 15.0
        });

        let response = client
            .post(&format!("{}/inventory/create", app.server_url))
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
                app.server_url, created_item.id
            ))
            .send()
            .await
            .expect("Gagal mengirim request DELETE");

        assert_eq!(response.status(), reqwest::StatusCode::OK);

        // Coba hapus lagi, harusnya 404 Not Found
        let response = client
            .delete(&format!(
                "{}/inventory/delete/{}",
                app.server_url, created_item.id
            ))
            .send()
            .await
            .expect("Gagal mengirim request DELETE");

        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    })
    .await;
}