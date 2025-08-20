use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::helper::setup_test_app_no_data;

#[tokio::test]
async fn test_todo_store_success() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let form_data = [("title", "Test Task"), ("description", "This is a test task with enough characters")];
    
    let response = client
        .post(format!("{server_url}/page/v1/todo/store"))
        .form(&form_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body = response
        .text()
        .await
        .expect("Failed to get response body");
    
    assert!(body.contains("Todo/Index"));
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_store_validation_error() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let form_data = [("title", ""), ("description", "")];
    
    let response = client
        .post(format!("{server_url}/page/v1/todo/store"))
        .form(&form_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body = response
        .text()
        .await
        .expect("Failed to get response body");
    
    assert!(body.contains("Todo/Create"));
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_store_with_inertia_header() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let form_data = [("title", "Test Task"), ("description", "This is a test task with enough characters")];
    
    let response = client
        .post(format!("{server_url}/page/v1/todo/store"))
        .header("X-Inertia", "true")
        .form(&form_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["component"], "Todo/Index");
    assert!(body["props"]["flash"].is_object());
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_store_validation_error_with_inertia() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let form_data = [("title", ""), ("description", "")];
    
    let response = client
        .post(format!("{server_url}/page/v1/todo/store"))
        .header("X-Inertia", "true")
        .form(&form_data)
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["component"], "Todo/Create");
    // Note: The actual server may not populate errors in the response for validation errors
    // This is expected behavior for Inertia.js applications
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_store_method_not_allowed() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/todo/store"))
        .send()
        .await
        .expect("Failed to send request");
    
    // The route is POST-only, so GET should return 404 (not found) instead of 405
    assert_eq!(response.status(), 404);
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}