use crate::page::helper::{setup_test_page_app_no_state, PageTestClient};
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_healthcheck_basic() {
    let (server_url, _server_handle, _temp_dir) = setup_test_page_app_no_state().await;
    
    let client = PageTestClient::new(server_url);
    
    let response = client.client
        .get(format!("{}/healthcheck", client.server_url))
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}

#[actix_web::test]
#[serial]
async fn test_server_responds_to_any_route() {
    let (server_url, _server_handle, _temp_dir) = setup_test_page_app_no_state().await;
    
    let client = PageTestClient::new(server_url);
    
    // Test various routes to ensure server responds
    let test_routes = vec![
        "/page/v1",
        "/page/v1/contact", 
        "/page/v1/todo",
        "/page/v1/todo/create",
        "/page/v1/todo/store",
        "/page/v1/foo",
        "/nonexistent",
    ];
    
    for route in test_routes {
        let response = client.client
            .get(format!("{}{}", client.server_url, route))
            .send()
            .await;
        
        // Each route should respond, regardless of status
        assert!(response.is_ok(), "Route {} should respond", route);
    }
}

#[actix_web::test]
#[serial]
async fn test_server_headers_present() {
    let (server_url, _server_handle, _temp_dir) = setup_test_page_app_no_state().await;
    
    let client = PageTestClient::new(server_url);
    
    let response = client.client
        .get(format!("{}/healthcheck", client.server_url))
        .send()
        .await
        .unwrap();
    
    // Check that response has basic headers
    assert!(response.headers().contains_key("content-type"));
    assert!(response.status().as_u16() > 0);
}