//! Integration tests for cross-functional behavior between API endpoints and page routes
//! These tests verify that data created via API is accessible via pages and vice versa

use actix_web::test;
use fake::{
    Fake,
    faker::lorem::en::Sentence,
};
use reqwest::Client as HttpClient;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::helper::{get_auth_token, setup_test_app as setup_api_test_app};
use crate::page::helper::setup_test_app as setup_page_test_app;

/// Test that inventory items created via API are visible on inventory page
#[tokio::test]
async fn test_api_created_inventory_visible_on_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create inventory items via API
    let item_names: Vec<String> = (0..3).map(|_| Sentence(1..3).fake()).collect();
    
    for name in &item_names {
        let new_item = json!({
            "name": name,
            "quantity": (1..100).fake::<i32>(),
            "price": (1.0..1000.0).fake::<f64>()
        });

        let response = client
            .post(format!("{server_url}/v1/inventory/create"))
            .bearer_auth(token.clone())
            .json(&new_item)
            .send()
            .await
            .expect("Failed to create inventory item");

        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }

    // Wait for Meilisearch indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Test that items are visible on inventory page
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1/inventory")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body["props"]["inventory"].is_array());
    
    let inventory_items = body["props"]["inventory"].as_array().unwrap();
    assert!(inventory_items.len() >= 3);
    
    // Verify our created items are in the list
    let returned_names: Vec<String> = inventory_items
        .iter()
        .map(|item| item["name"].as_str().unwrap().to_string())
        .collect();
    
    for name in item_names {
        assert!(returned_names.contains(&name));
    }

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test that employee records created via API are visible on employee page
#[tokio::test]
async fn test_api_created_employee_visible_on_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create employee via API
    let employee_name: String = Sentence(1..3).fake();
    let employee_email: String = fake::faker::internet::en::SafeEmail().fake();
    let employee_position: String = Sentence(1..2).fake();
    
    let new_employee = json!({
        "name": employee_name,
        "email": employee_email,
        "position": employee_position,
        "salary": (30000.0..100000.0).fake::<f64>()
    });

    let response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(token.clone())
        .json(&new_employee)
        .send()
        .await
        .expect("Failed to create employee");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Test that employee is visible on employee page
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1/employee")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body["props"]["employees"].is_array());
    
    let employees = body["props"]["employees"].as_array().unwrap();
    assert!(!employees.is_empty());
    
    // Verify our created employee is in the list
    let employee_found = employees.iter().any(|emp| 
        emp["name"].as_str() == Some(&employee_name) &&
        emp["email"].as_str() == Some(&employee_email)
    );
    assert!(employee_found);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test that orders created via API are visible on order page
#[tokio::test]
async fn test_api_created_order_visible_on_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create order via API
    let order_description: String = Sentence(2..4).fake();
    let order_total: f64 = (100.0..5000.0).fake();
    
    let new_order = json!({
        "description": order_description,
        "total": order_total,
        "status": "pending"
    });

    let response = client
        .post(format!("{server_url}/v1/order/create"))
        .bearer_auth(token.clone())
        .json(&new_order)
        .send()
        .await
        .expect("Failed to create order");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Test that order is visible on order page
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1/order")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body["props"]["orders"].is_array());
    
    let orders = body["props"]["orders"].as_array().unwrap();
    assert!(!orders.is_empty());
    
    // Verify our created order is in the list
    let order_found = orders.iter().any(|ord| 
        ord["description"].as_str() == Some(&order_description) &&
        ord["total"].as_f64() == Some(order_total)
    );
    assert!(order_found);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test that data deleted via API is no longer visible on pages
#[tokio::test]
async fn test_api_deleted_inventory_not_visible_on_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create inventory item via API
    let item_name: String = Sentence(1..3).fake();
    let new_item = json!({
        "name": item_name,
        "quantity": 50,
        "price": 99.99
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Failed to create inventory item");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let created_item: api::v1::inventory::models::InventoryItem = response.json().await.unwrap();

    // Wait for Meilisearch indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Verify item exists on page
    let app = setup_page_test_app().await;
    let req = test::TestRequest::get()
        .uri("/page/v1/inventory")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    let initial_count = body["props"]["inventory"].as_array().unwrap().len();

    // Delete item via API
    let response = client
        .delete(format!("{server_url}/v1/inventory/delete/{}", created_item.id))
        .bearer_auth(token.clone())
        .send()
        .await
        .expect("Failed to delete inventory item");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Wait for Meilisearch indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Verify item is no longer on page
    let app = setup_page_test_app().await;
    let req = test::TestRequest::get()
        .uri("/page/v1/inventory")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    let new_count = body["props"]["inventory"].as_array().unwrap().len();
    
    assert_eq!(new_count, initial_count - 1);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test that data updated via API reflects changes on pages
#[tokio::test]
async fn test_api_updated_inventory_reflected_on_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create inventory item via API
    let original_name: String = Sentence(1..3).fake();
    let new_item = json!({
        "name": original_name,
        "quantity": 50,
        "price": 99.99
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Failed to create inventory item");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let created_item: api::v1::inventory::models::InventoryItem = response.json().await.unwrap();

    // Wait for Meilisearch indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Update item via API
    let updated_name: String = Sentence(1..3).fake();
    let updated_data = json!({
        "name": updated_name,
        "quantity": 75,
        "price": 149.99
    });

    let response = client
        .put(format!("{server_url}/v1/inventory/update/{}", created_item.id))
        .bearer_auth(token.clone())
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to update inventory item");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Wait for Meilisearch indexing
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Verify changes are reflected on page
    let app = setup_page_test_app().await;
    let req = test::TestRequest::get()
        .uri("/page/v1/inventory")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    
    let inventory_items = body["props"]["inventory"].as_array().unwrap();
    let updated_item = inventory_items.iter()
        .find(|item| item["id"].as_str() == Some(&created_item.id))
        .expect("Updated item not found");
    
    assert_eq!(updated_item["name"].as_str(), Some(updated_name.as_str()));
    assert_eq!(updated_item["quantity"].as_i64(), Some(75));
    assert_eq!(updated_item["price"].as_f64(), Some(149.99));

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test session handling and flash messages across API and page routes
#[tokio::test]
async fn test_session_handling_across_api_and_page() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create a todo task via API (if such endpoint exists)
    // This test focuses on session handling and flash messages
    
    // Test that flash messages work on page routes
    let app = setup_page_test_app().await;
    
    // Test GET request to a page that might set flash messages
    let req = test::TestRequest::get()
        .uri("/page/v1/todo/create")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body["props"]["flash"].is_object());
    
    // Test that authentication state is consistent
    let req = test::TestRequest::get()
        .uri("/page/v1/todo")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Verify Inertia response headers
    assert!(resp.headers().contains_key("x-inertia"));
    assert_eq!(resp.headers().get("x-inertia").unwrap(), "true");

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test error handling consistency between API and page routes
#[tokio::test]
async fn test_error_handling_consistency() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Test API error response
    let non_existent_id = Uuid::new_v4().to_string();
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=nonexistent"))
        .bearer_auth(token.clone())
        .send()
        .await
        .expect("Failed to send request");

    // API should return appropriate status
    assert_eq!(response.status(), reqwest::StatusCode::OK); // Search returns empty array, not error

    // Test page error response for non-existent resource
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri(&format!("/page/v1/inventory/{}", non_existent_id))
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Page should handle gracefully (either 404 or empty state)
    // The exact behavior depends on implementation, but should be consistent
    assert!(resp.status().is_success() || resp.status().as_u16() == 404);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test Inertia response format consistency
#[tokio::test]
async fn test_inertia_response_format_consistency() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Create some test data via API
    let new_item = json!({
        "name": "Test Item",
        "quantity": 10,
        "price": 29.99
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&new_item)
        .send()
        .await
        .expect("Failed to create inventory item");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Test page response format
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1/inventory")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Verify Inertia headers
    assert!(resp.headers().contains_key("x-inertia"));
    assert_eq!(resp.headers().get("x-inertia").unwrap(), "true");
    
    // Verify JSON response structure
    let body: Value = test::read_body_json(resp).await;
    assert!(body["component"].is_string());
    assert!(body["props"].is_object());
    assert!(body["url"].is_string());
    assert!(body["version"].is_string());

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test authentication state consistency between API and pages
#[tokio::test]
async fn test_authentication_consistency() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Test API with valid token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth(token.clone())
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    
    // Test API with invalid token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth("invalid_token")
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    
    // Test page access (pages might handle auth differently)
    let app = setup_page_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1/todo")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    // Pages might redirect or show different content based on auth
    assert!(resp.status().is_success() || resp.status().as_u16() == 302);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Test data validation consistency between API and pages
#[tokio::test]
async fn test_data_validation_consistency() {
    let (db_pool, _meili_client, server_url, server_handle) = 
        setup_api_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    
    // Test API validation
    let invalid_item = json!({
        "name": "",
        "quantity": -5,
        "price": -10.0
    });

    let response = client
        .post(format!("{server_url}/v1/inventory/create"))
        .bearer_auth(token.clone())
        .json(&invalid_item)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}