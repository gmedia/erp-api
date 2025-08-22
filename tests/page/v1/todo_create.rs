use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::helper::TestAppBuilder;

#[tokio::test]
async fn test_todo_create_page_returns_200() {
    let app = TestAppBuilder::new()
        .clear_tables()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/todo/create"))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_create_page_renders_correct_inertia_component() {
    let app = TestAppBuilder::new()
        .clear_tables()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/todo/create"))
        .header("X-Inertia", "true")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["component"], "Todo/Create");
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_create_page_has_correct_props() {
    let app = TestAppBuilder::new()
        .clear_tables()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/todo/create"))
        .header("X-Inertia", "true")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert!(body["props"].is_object());
    assert!(body["props"]["errors"].is_null() || body["props"]["errors"].is_object());
    assert!(body["props"]["flash"].is_object());
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_todo_create_page_without_inertia_header() {
    let app = TestAppBuilder::new()
        .clear_tables()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/todo/create"))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body = response
        .text()
        .await
        .expect("Failed to read response body");
    
    // Should contain the HTML structure
    assert!(body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<div id=\"app\"") || body.contains("<div id='app'"));
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}