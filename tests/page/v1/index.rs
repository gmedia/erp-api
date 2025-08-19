use actix_web::test;
use serde_json::Value;

use crate::page::helper::setup_test_app;

#[actix_web::test]
async fn test_index_page_returns_200() {
    let app = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_index_page_renders_correct_inertia_component() {
    let app = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["component"], "Index");
}

#[actix_web::test]
async fn test_index_page_has_correct_props() {
    let app = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1")
        .insert_header(("X-Inertia", "true"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body["props"].is_object());
    assert!(body["props"]["errors"].is_object());
    assert!(body["props"]["flash"].is_object());
}

#[actix_web::test]
async fn test_index_page_without_inertia_header() {
    let app = setup_test_app().await;
    
    let req = test::TestRequest::get()
        .uri("/page/v1")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body = test::read_body(resp).await;
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain the HTML structure
    assert!(body_str.contains("<!DOCTYPE html>"));
    assert!(body_str.contains("<div id=\"app\""));
}