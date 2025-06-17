use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use api::v1::employee::models::Employee;
mod common;
use common::setup_test_app;

#[tokio::test]
#[serial]
async fn test_create_employee() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Software Engineer";

    // Tes endpoint POST /employee/create
    let new_employee = json!({
        "name": name,
        "role": role,
        "email": email
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

    assert_eq!(created_employee.name, name);
    assert_eq!(created_employee.role, role);
    assert_eq!(created_employee.email, email);
}

#[tokio::test]
#[serial]
async fn test_create_employee_invalid_email() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let name: String = Name().fake();
    let role = "Product Manager";

    let new_employee = json!({
        "name": name,
        "role": role,
        "email": "jane.doe"
    });

    let response = client
        .post(&format!("{}/employee/create", server_url))
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}