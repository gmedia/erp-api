use fake::Fake;
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
    let total_amount: f64 = (1.0..1000.0).fake();

    // Tes endpoint POST /v1/order/create
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": total_amount
    });

    let response = client
        .post(&format!("{}/v1/order/create", server_url))
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
    assert!((created_order.total_amount - total_amount).abs() < 1e-9);
}

#[tokio::test]
#[serial]
async fn test_create_order_negative_amount() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let customer_id = Uuid::new_v4().to_string();

    // Tes endpoint POST /v1/order/create dengan jumlah negatif
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": -150.75
    });

    let response = client
        .post(&format!("{}/v1/order/create", server_url))
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_create_order_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let customer_id = Uuid::new_v4().to_string();
    let total_amount: f64 = (1.0..1000.0).fake();

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": total_amount
    });

    let response = client
        .post(&format!("{}/v1/order/create", server_url))
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}
