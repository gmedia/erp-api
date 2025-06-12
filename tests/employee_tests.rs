use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use erp_api::api::v1::employee::models::Employee;
mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_create_employee() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();

    // Tes endpoint POST /employee/create
    let new_employee = json!({
        "name": "John Doe",
        "role": "Software Engineer",
        "email": "john.doe@example.com"
    });

    let response = client
        .post(&format!("{}/employee/create", server_url))
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_employee: Employee = response
        .json()
        .await
        .expect("Gagal parse response JSON");

    assert_eq!(created_employee.name, "John Doe");
    assert_eq!(created_employee.role, "Software Engineer");
    assert_eq!(created_employee.email, "john.doe@example.com");
}