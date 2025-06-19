use api::v1::auth::models::TokenResponse;
use fake::{faker::internet::en::SafeEmail, Fake};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_register_and_login() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Test registration
    let register_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(&format!("{}/v1/auth/register", server_url))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send registration request");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Test login
    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(&format!("{}/v1/auth/login", server_url))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let token_response: TokenResponse = response
        .json()
        .await
        .expect("Failed to parse token response");

    assert!(!token_response.token.is_empty());
}

#[tokio::test]
#[serial]
async fn test_access_protected_route() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Register and login to get a token
    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(&format!("{}/v1/auth/register", server_url))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(&format!("{}/v1/auth/login", server_url))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    let token = token_response.token;

    // Access protected route with token
    let response = client
        .get(&format!("{}/v1/inventory/search?q=test", server_url))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Access protected route without token
    let response = client
        .get(&format!("{}/v1/inventory/search?q=test", server_url))
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}