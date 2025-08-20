use reqwest::Client as HttpClient;
use serde_json::Value;

use crate::helper::setup_test_app_no_data;

#[tokio::test]
async fn test_contact_page_returns_200() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/contact"))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_contact_page_renders_correct_inertia_component() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/contact"))
        .header("X-Inertia", "true")
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body: Value = response
        .json()
        .await
        .expect("Failed to parse response");
    
    assert_eq!(body["component"], "Contact");
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_contact_page_has_correct_props() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/contact"))
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
async fn test_contact_page_without_inertia_header() {
    let (_db_pool, _meili_client, server_url, server_handle) = setup_test_app_no_data().await;
    let client = HttpClient::new();
    
    let response = client
        .get(format!("{server_url}/page/v1/contact"))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success());
    
    let body = response
        .text()
        .await
        .expect("Failed to read response body");
    
    println!("Response body: {}", body);
    println!("Body length: {}", body.len());

    // Should contain the HTML structure
    assert!(body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<div id=\"app\"") || body.contains("<div id='app'"));
    
    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}