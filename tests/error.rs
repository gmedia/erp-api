use reqwest::Client as HttpClient;
use serde_json::json;

mod common;
use common::{setup_test_app, get_auth_token};

#[actix_rt::test]
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
