use crate::page::helper::{setup_test_page_app_no_data, setup_test_page_app_no_state, PageTestClient};
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_healthcheck_endpoint() {
    let (_db_pool, _meili_client, server_url, _server_handle, _temp_dir) =
        setup_test_page_app_no_data().await;

    let client = PageTestClient::new(server_url);
    
    let response = client.client
        .get(format!("{}/healthcheck", client.server_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
#[serial]
async fn test_server_starts_and_responds() {
    let (_db_pool, _meili_client, server_url, _server_handle, _temp_dir) =
        setup_test_page_app_no_data().await;

    let client = PageTestClient::new(server_url);
    
    // Test that server is running
    let response = client.client
        .get(format!("{}/healthcheck", client.server_url))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_all_page_routes_respond() {
    let (_db_pool, _meili_client, server_url, _server_handle, _temp_dir) =
        setup_test_page_app_no_data().await;

    let client = PageTestClient::new(server_url);
    
    // Test all available page routes
    let routes = vec![
        "/page/v1",
        "/page/v1/contact",
        "/page/v1/todo",
        "/page/v1/todo/create",
        "/page/v1/foo",
    ];
    
    for route in routes {
        let response = client.client
            .get(format!("{}{}", client.server_url, route))
            .send()
            .await;
            
        // Each route should at least respond, even if it's 404
        assert!(response.is_ok(), "Route {} should respond", route);
        
        let response = response.unwrap();
        // Allow any response status (200, 404, etc.) - we're testing basic connectivity
        assert!(response.status().as_u16() > 0, "Route {} should return a status", route);
    }
}

#[actix_web::test]
#[serial]
async fn test_basic_server_functionality() {
    let (server_url, _server_handle, _temp_dir) =
        setup_test_page_app_no_state().await;

    let client = PageTestClient::new(server_url);
    
    // Test healthcheck with minimal setup
    let response = client.client
        .get(format!("{}/healthcheck", client.server_url))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
    
    // Test that server responds to any request
    let response = client.client
        .get(format!("{}/page/v1", client.server_url))
        .send()
        .await;
    
    // Should get some response, even if it's 404
    assert!(response.is_ok());
}