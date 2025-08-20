use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::helper::setup_test_app_no_data;

#[tokio::test]
async fn test_index_page_returns_200() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1"))
        .send()
        .await
        .expect("Failed to send request");
    
    let status = response.status();
    println!("Response status: {}", status);
    println!("Response headers: {:?}", response.headers());
    
    if !status.is_success() {
        let body = response.text().await.unwrap_or_else(|_| "Failed to read body".to_string());
        println!("Response body: {}", body);
        panic!("Expected success status, got: {}", status);
    }
    
    assert!(status.is_success());
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_index_page_renders_correct_inertia_component() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1"))
        .header("X-Inertia", "true")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["component"], "Index");
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_index_page_has_correct_props() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1"))
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
async fn test_index_page_without_inertia_header() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1"))
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