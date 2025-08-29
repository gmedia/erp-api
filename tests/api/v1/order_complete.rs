use api::v1::order::models::Order;
use fake::Fake;
use reqwest::Client as HttpClient;
use serde_json::json;
use uuid::Uuid;

use crate::helper::{get_auth_token, TestAppBuilder};
use entity::order::{Entity as OrderEntity};
use sea_orm::{EntityTrait};

#[tokio::test]
async fn test_get_all_orders() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Clean up existing orders using entity-based approach
    let _ = OrderEntity::delete_many().exec(db_pool).await;

    // Create test orders
    let customer_id1 = Uuid::new_v4().to_string();
    let customer_id2 = Uuid::new_v4().to_string();
    let total_amount1: f64 = (10.0..100.0).fake();
    let total_amount2: f64 = (100.0..1000.0).fake();

    let order1 = json!({
        "customer_id": customer_id1,
        "total_amount": total_amount1
    });

    let order2 = json!({
        "customer_id": customer_id2,
        "total_amount": total_amount2
    });

    // Create first order
    let _ = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&order1)
        .send()
        .await
        .unwrap();

    // Create second order
    let _ = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&order2)
        .send()
        .await
        .unwrap();

    // Test GET /v1/order
    let response = client
        .get(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let orders: Vec<Order> = response.json().await.expect("Failed to parse response");
    assert!(orders.len() >= 2);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_get_order_by_id() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test order
    let customer_id = Uuid::new_v4().to_string();
    let total_amount: f64 = (50.0..500.0).fake();
    let new_order = json!({
        "customer_id": customer_id.clone(),
        "total_amount": total_amount
    });

    let create_response = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&new_order)
        .send()
        .await
        .expect("Failed to create order");

    let created_order: Order = create_response.json().await.unwrap();
    let order_id = created_order.id;

    // Test GET /v1/order/{id}
    let response = client
        .get(format!("{server_url}/v1/order/{order_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let order: Order = response.json().await.expect("Failed to parse response");
    assert_eq!(order.id, order_id);
    assert_eq!(order.customer_id, customer_id);
    assert!((order.total_amount - total_amount).abs() < 1e-9);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_get_order_by_nonexistent_id() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();

    // Test GET /v1/order/{nonexistent_id}
    let response = client
        .get(format!("{server_url}/v1/order/{nonexistent_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_order() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test order
    let customer_id = Uuid::new_v4().to_string();
    let original_amount: f64 = 100.0;
    let new_order = json!({
        "customer_id": customer_id.clone(),
        "total_amount": original_amount
    });

    let create_response = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&new_order)
        .send()
        .await
        .expect("Failed to create order");

    let created_order: Order = create_response.json().await.unwrap();
    let order_id = created_order.id;

    // Update order data
    let updated_amount: f64 = 150.0;
    let updated_data = json!({
        "customer_id": customer_id,
        "total_amount": updated_amount
    });

    // Test PUT /v1/order/{id}
    let response = client
        .put(format!("{server_url}/v1/order/{order_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_order: Order = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_order.id, order_id);
    assert!((updated_order.total_amount - updated_amount).abs() < 1e-9);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_nonexistent_order() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();
    let updated_data = json!({
        "customer_id": Uuid::new_v4().to_string(),
        "total_amount": 200.0
    });

    // Test PUT /v1/order/{nonexistent_id}
    let response = client
        .put(format!("{server_url}/v1/order/{nonexistent_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_order_negative_amount() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test order
    let customer_id = Uuid::new_v4().to_string();
    let original_amount: f64 = 100.0;
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": original_amount
    });

    let create_response = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&new_order)
        .send()
        .await
        .expect("Failed to create order");

    let created_order: Order = create_response.json().await.unwrap();
    let order_id = created_order.id;

    // Update with negative amount
    let updated_data = json!({
        "customer_id": customer_id,
        "total_amount": -50.0
    });

    let response = client
        .put(format!("{server_url}/v1/order/{order_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete_order() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test order
    let customer_id = Uuid::new_v4().to_string();
    let total_amount: f64 = 75.0;
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": total_amount
    });

    let create_response = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&new_order)
        .send()
        .await
        .expect("Failed to create order");

    let created_order: Order = create_response.json().await.unwrap();
    let order_id = created_order.id;

    // Test DELETE /v1/order/{id}
    let response = client
        .delete(format!("{server_url}/v1/order/{order_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Verify order is deleted
    let verify_response = client
        .get(format!("{server_url}/v1/order/{order_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to verify deletion");

    assert_eq!(verify_response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete_nonexistent_order() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();

    // Test DELETE /v1/order/{nonexistent_id}
    let response = client
        .delete(format!("{server_url}/v1/order/{nonexistent_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_order_unauthorized_access() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;

    let client = HttpClient::new();

    // Test GET all without token
    let response = client
        .get(format!("{server_url}/v1/order"))
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test GET by ID without token
    let response = client
        .get(format!("{server_url}/v1/order/123"))
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test PUT without token
    let response = client
        .put(format!("{server_url}/v1/order/123"))
        .json(&json!({"customer_id": Uuid::new_v4().to_string(), "total_amount": 100.0}))
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test DELETE without token
    let response = client
        .delete(format!("{server_url}/v1/order/123"))
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_order_zero_amount() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let customer_id = Uuid::new_v4().to_string();
    let new_order = json!({
        "customer_id": customer_id,
        "total_amount": 0.0
    });

    let response = client
        .post(format!("{server_url}/v1/order"))
        .bearer_auth(&token)
        .json(&new_order)
        .send()
        .await
        .expect("Failed to send POST request");

    // Should accept zero amount
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}