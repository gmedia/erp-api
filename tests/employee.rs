use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use reqwest::Client as HttpClient;
use serde_json::json;

use api::v1::employee::models::Employee;
mod common;
use common::{setup_test_app, get_auth_token};

#[actix_rt::test]
async fn test_create_employee() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
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
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let created_employee: Employee = response.json().await.expect("Gagal parse response JSON");

    assert_eq!(created_employee.name, name);
    assert_eq!(created_employee.role, role);
    assert_eq!(created_employee.email, email);
}

#[actix_rt::test]
async fn test_create_employee_invalid_email() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
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
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_create_employee_internal_server_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
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
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}
