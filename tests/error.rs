use api::v1::auth::models::TokenResponse;
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

mod common;
use common::setup_test_app;

async fn get_auth_token(client: &HttpClient, server_url: &str) -> String {
    let username: String = SafeEmail().fake();
    let password = "password123";

    let register_req = json!({
        "username": username.clone(),
        "password": password,
    });

    let _ = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await;

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    token_response.token
}

#[tokio::test]
#[serial]
async fn test_internal_server_error_response_format() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();

    // Get a valid token first
    let token = get_auth_token(&client, &server_url).await;

    // Now, close the DB pool to force a DB error on the next request
    let _ = db_pool.close().await;

    let new_item = json!({
        "name": "This will fail",
        "quantity": 1,
        "price": 1.0
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token)
        .json(&new_item)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse error response");

    // Verify the structure of the error response
    assert_eq!(body["error"]["code"], 500);
    assert!(body["error"]["message"].as_str().is_some());
    assert_eq!(body["error"]["message"], "Database error");
}
