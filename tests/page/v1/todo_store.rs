use actix_web::test;
use serde_json::Value;

use crate::page::helper::setup_test_app;

#[actix_web::test]
async fn test_todo_store_success() {
    let app = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/page/v1/todo/store")
        .set_form(&[("title", "Test Task"), ("description", "This is a test task")])
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());
    
    let location = resp.headers().get("Location").unwrap();
    assert_eq!(location.to_str().unwrap(), "/page/v1/todo");
}

#[actix_web::test]
async fn test_todo_store_validation_error() {
    let app = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/page/v1/todo/store")
        .set_form(&[("title", ""), ("description", "")])
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());
    
    let location = resp.headers().get("Location").unwrap();
    assert_eq!(location.to_str().unwrap(), "/page/v1/todo/create");
}

#[actix_web::test]
async fn test_todo_store_with_inertia_header() {
    let app = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/page/v1/todo/store")
        .insert_header(("X-Inertia", "true"))
        .set_form(&[("title", "Test Task"), ("description", "This is a test task")])
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["component"], "Todo/Index");
    assert!(body["props"]["flash"].is_object());
}

#[actix_web::test]
async fn test_todo_store_validation_error_with_inertia() {
    let app = setup_test_app().await;

    let req = test::TestRequest::post()
        .uri("/page/v1/todo/store")
        .insert_header(("X-Inertia", "true"))
        .set_form(&[("title", ""), ("description", "")])
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["component"], "Todo/Create");
    assert!(body["props"]["errors"].is_object());
    assert!(body["props"]["errors"]["title"].is_string());
}

#[actix_web::test]
async fn test_todo_store_method_not_allowed() {
    let app = setup_test_app().await;

    let req = test::TestRequest::get()
        .uri("/page/v1/todo/store")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 405);
}