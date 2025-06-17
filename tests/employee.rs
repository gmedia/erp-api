use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use api::v1::employee::models::Employee;
mod common;
use common::run_test;

#[tokio::test]
#[serial]
async fn test_create_employee() {
    run_test(|app| async move {
        let client = HttpClient::new();

        let new_employee = json!({
            "name": "John Doe",
            "role": "Software Engineer",
            "email": "john.doe@example.com"
        });

        let response = client
            .post(&format!("{}/employee/create", app.server_url))
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
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_create_employee_invalid_email() {
    run_test(|app| async move {
        let client = HttpClient::new();

        let new_employee = json!({
            "name": "Jane Doe",
            "role": "Product Manager",
            "email": "jane.doe"
        });

        let response = client
            .post(&format!("{}/employee/create", app.server_url))
            .json(&new_employee)
            .send()
            .await
            .expect("Gagal mengirim request POST");

        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    })
    .await;
}