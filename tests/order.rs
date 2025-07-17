use api::v1::auth::models::TokenResponse;
use api::v1::order::models::Order;
use fake::{faker::internet::en::SafeEmail, Fake};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;
use uuid::Uuid;

mod common;
use common::setup_test_app;

async fn get_auth_token(client: &HttpClient, server_url: &str) -> String {
    let username: String = SafeEmail().fake();
    let password = "password123";

    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(format!("{}/v1/auth/register", server_url))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{}/v1/auth/login", server_url))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    token_response.token
}

#[tokio::test]
#[serial]
async fn test_create_order() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url).await;
    let customer_id = Uuid::new_v4().to_string();
    let total_amount: f64 = (1.0..1000.0).fake();

    // Tes endpoint POST /v1/order/create
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": total_amount
    });

    let response = client
        .post(format!("{}/v1/order/create", server_url))
        .bearer_auth(token)
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
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url).await;
    let customer_id = Uuid::new_v4().to_string();

    // Tes endpoint POST /v1/order/create dengan jumlah negatif
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": -150.75
    });

    let response = client
        .post(format!("{}/v1/order/create", server_url))
        .bearer_auth(token)
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn test_create_order_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url).await;
    let customer_id = Uuid::new_v4().to_string();
    let total_amount: f64 = (1.0..1000.0).fake();

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": total_amount
    });

    let response = client
        .post(format!("{}/v1/order/create", server_url))
        .bearer_auth(token)
        .json(&new_order)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}
