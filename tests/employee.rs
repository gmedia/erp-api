use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use reqwest::Client as HttpClient;
use serde_json::json;
use serial_test::serial;

use api::v1::employee::models::Employee;
mod common;
use api::v1::auth::models::TokenResponse;
use common::setup_test_app;

async fn get_auth_token(client: &HttpClient, server_url: &str) -> String {
    let username: String = SafeEmail().fake();
    let password = "password123";

    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(&format!("{}/v1/auth/register", server_url))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(&format!("{}/v1/auth/login", server_url))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    token_response.token
}

#[tokio::test]
#[serial]
async fn test_create_employee() {
    let (_db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url).await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Software Engineer";

    // Tes endpoint POST /v1/employee/create
    let new_employee = json!({
        "name": name,
        "role": role,
        "email": email
    });

    let response = client
        .post(&format!("{}/v1/employee/create", server_url))
        .bearer_auth(token)
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
    let token = get_auth_token(&client, &server_url).await;
    let name: String = Name().fake();
    let role = "Product Manager";

    let new_employee = json!({
        "name": name,
        "role": role,
        "email": "jane.doe"
    });

    let response = client
        .post(&format!("{}/v1/employee/create", server_url))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}
#[tokio::test]
#[serial]
async fn test_create_employee_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app().await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url).await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Chaos Engineer";

    // Simulate database connection error by closing the pool
    let _ = db_pool.close().await;

    let new_employee = json!({
        "name": name,
        "role": role,
        "email": email
    });

    let response = client
        .post(&format!("{}/v1/employee/create", server_url))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}