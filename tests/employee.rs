use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use reqwest::Client as HttpClient;
use serde_json::json;

use api::v1::employee::models::Employee;
mod common;
use common::{get_auth_token, setup_test_app};
use sea_orm::{ConnectionTrait, Statement};

#[tokio::test(flavor = "multi_thread")]
async fn test_create_employee() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Software Engineer";

    // Clean employee
    let backend: sea_orm::DatabaseBackend = db_pool.get_database_backend();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            format!("DELETE FROM employee where email = '{email}'"),
        ))
        .await;

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

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_employee_invalid_email() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_employee_internal_server_error() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;
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

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
