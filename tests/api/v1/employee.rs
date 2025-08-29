use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use reqwest::Client as HttpClient;
use serde_json::json;

use api::v1::employee::models::Employee;

use crate::helper::{TestAppBuilder, get_auth_token};
use entity::employee::{Column as EmployeeColumn, Entity as EmployeeEntity};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[tokio::test]
async fn test_create() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Software Engineer";

    // Clean employee using entity-based approach
    let _ = EmployeeEntity::delete_many()
        .filter(EmployeeColumn::Email.eq(&email))
        .exec(db_pool)
        .await;

    // Tes endpoint POST /v1/employee
    let new_employee = json!({
        "name": name,
        "role": role,
        "email": email
    });

    let response = client
        .post(format!("{server_url}/v1/employee"))
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

#[tokio::test]
async fn test_create_invalid_email() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Name().fake();
    let role = "Product Manager";

    let new_employee = json!({
        "name": name,
        "role": role,
        "email": "jane.doe"
    });

    let response = client
        .post(format!("{server_url}/v1/employee"))
        .bearer_auth(token)
        .json(&new_employee)
        .send()
        .await
        .expect("Gagal mengirim request POST");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_create_internal_server_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, server_url, db_pool).await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let role = "Chaos Engineer";

    // Simulate database connection error by closing the pool
    let _ = app.db.close().await;

    let new_employee = json!({
        "name": name,
        "role": role,
        "email": email
    });

    let response = client
        .post(format!("{server_url}/v1/employee"))
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
