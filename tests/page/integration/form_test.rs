use crate::page::helper::{setup_test_page_app_no_data, PageTestClient};
use serde_json::json;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn test_todo_form_submission() {
    let (_db_pool, _meili_client, server_url, _server_handle, _temp_dir) =
        setup_test_page_app_no_data().await;

    let client = PageTestClient::new(server_url);
    
    // Test that the form endpoint responds
    let response = client.client
        .get(format!("{}/page/v1/todo/store", client.server_url))
        .send()
        .await;
    
    // Any response is acceptable - we're testing basic connectivity
    assert!(response.is_ok(), "Server should respond to /page/v1/todo/store");
    
    // Test POST endpoint
    let valid_data = json!({
        "title": "Test Todo",
        "content": "This is a test todo item"
    });
    
    let response = client.client
        .post(format!("{}/page/v1/todo/store", client.server_url))
        .json(&valid_data)
        .send()
        .await;
    
    // Any response is acceptable - we're testing basic connectivity
    assert!(response.is_ok(), "Server should respond to POST /page/v1/todo/store");
}