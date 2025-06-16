use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

use api::v1::order::models::Order;
mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_create_order() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let customer_id = Uuid::new_v4().to_string();

    // Tes endpoint POST /order/create
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": 150.75
    });

    let response = client
        .post(&format!("{}/order/create", server_url))
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_order: Order = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    assert_eq!(created_order.customer_id, customer_id);
    assert_eq!(created_order.total_amount, 150.75);
}

#[tokio::test]
#[serial]
async fn test_create_order_negative_amount() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let customer_id = Uuid::new_v4().to_string();

    // Tes endpoint POST /order/create dengan jumlah negatif
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": -150.75
    });

    let response = client
        .post(&format!("{}/order/create", server_url))
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}
