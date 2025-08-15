use fake::{
    Fake,
    faker::{internet::en::SafeEmail, name::en::Name},
};
use reqwest::Client as HttpClient;
use serde_json::json;
use uuid::Uuid;

use api::v1::employee::models::Employee;

use crate::helper::{get_auth_token, setup_test_app};
use sea_orm::{ConnectionTrait, Statement};

#[tokio::test]
async fn test_get_all_employees() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Clean up existing employees
    let backend: sea_orm::DatabaseBackend = db_pool.get_database_backend();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            "DELETE FROM employee".to_string(),
        ))
        .await;

    // Create test employees
    let employee1 = json!({
        "name": Name().fake::<String>(),
        "role": "Software Engineer",
        "email": SafeEmail().fake::<String>()
    });

    let employee2 = json!({
        "name": Name().fake::<String>(),
        "role": "Product Manager",
        "email": SafeEmail().fake::<String>()
    });

    // Create first employee
    let _ = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&employee1)
        .send()
        .await
        .unwrap();

    // Create second employee
    let _ = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&employee2)
        .send()
        .await
        .unwrap();

    // Test GET /v1/employee
    let response = client
        .get(format!("{server_url}/v1/employee"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let employees: Vec<Employee> = response.json().await.expect("Failed to parse response");
    assert!(employees.len() >= 2);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_get_employee_by_id() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test employee
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let new_employee = json!({
        "name": name.clone(),
        "role": "Data Scientist",
        "email": email.clone()
    });

    let create_response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&new_employee)
        .send()
        .await
        .expect("Failed to create employee");

    let created_employee: Employee = create_response.json().await.unwrap();
    let employee_id = created_employee.id;

    // Test GET /v1/employee/{id}
    let response = client
        .get(format!("{server_url}/v1/employee/{employee_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let employee: Employee = response.json().await.expect("Failed to parse response");
    assert_eq!(employee.id, employee_id);
    assert_eq!(employee.name, name);
    assert_eq!(employee.email, email);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_get_employee_by_nonexistent_id() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();

    // Test GET /v1/employee/{nonexistent_id}
    let response = client
        .get(format!("{server_url}/v1/employee/{nonexistent_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_employee() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test employee
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let new_employee = json!({
        "name": name.clone(),
        "role": "Frontend Developer",
        "email": email.clone()
    });

    let create_response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&new_employee)
        .send()
        .await
        .expect("Failed to create employee");

    let created_employee: Employee = create_response.json().await.unwrap();
    let employee_id = created_employee.id;

    // Update employee data
    let updated_data = json!({
        "name": format!("{} Updated", name),
        "role": "Senior Frontend Developer",
        "email": email
    });

    // Test PUT /v1/employee/{id}
    let response = client
        .put(format!("{server_url}/v1/employee/{employee_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let updated_employee: Employee = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_employee.id, employee_id);
    assert_eq!(updated_employee.name, format!("{} Updated", name));
    assert_eq!(updated_employee.role, "Senior Frontend Developer");

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_nonexistent_employee() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();
    let updated_data = json!({
        "name": "Updated Name",
        "role": "Updated Role",
        "email": SafeEmail().fake::<String>()
    });

    // Test PUT /v1/employee/{nonexistent_id}
    let response = client
        .put(format!("{server_url}/v1/employee/{nonexistent_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_update_employee_invalid_email() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test employee
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let new_employee = json!({
        "name": name.clone(),
        "role": "Backend Developer",
        "email": email.clone()
    });

    let create_response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&new_employee)
        .send()
        .await
        .expect("Failed to create employee");

    let created_employee: Employee = create_response.json().await.unwrap();
    let employee_id = created_employee.id;

    // Update with invalid email
    let updated_data = json!({
        "name": name,
        "role": "Backend Developer",
        "email": "invalid-email"
    });

    let response = client
        .put(format!("{server_url}/v1/employee/{employee_id}"))
        .bearer_auth(&token)
        .json(&updated_data)
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete_employee() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Create test employee
    let new_employee = json!({
        "name": Name().fake::<String>(),
        "role": "DevOps Engineer",
        "email": SafeEmail().fake::<String>()
    });

    let create_response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&new_employee)
        .send()
        .await
        .expect("Failed to create employee");

    let created_employee: Employee = create_response.json().await.unwrap();
    let employee_id = created_employee.id;

    // Test DELETE /v1/employee/{id}
    let response = client
        .delete(format!("{server_url}/v1/employee/{employee_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Verify employee is deleted
    let verify_response = client
        .get(format!("{server_url}/v1/employee/{employee_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to verify deletion");

    assert_eq!(verify_response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_delete_nonexistent_employee() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let nonexistent_id = Uuid::new_v4().to_string();

    // Test DELETE /v1/employee/{nonexistent_id}
    let response = client
        .delete(format!("{server_url}/v1/employee/{nonexistent_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_employee_unauthorized_access() {
    let (_db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();

    // Test GET all without token
    let response = client
        .get(format!("{server_url}/v1/employee"))
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test GET by ID without token
    let response = client
        .get(format!("{server_url}/v1/employee/123"))
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test PUT without token
    let response = client
        .put(format!("{server_url}/v1/employee/123"))
        .json(&json!({"name": "Test", "role": "Test", "email": "test@example.com"}))
        .send()
        .await
        .expect("Failed to send PUT request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Test DELETE without token
    let response = client
        .delete(format!("{server_url}/v1/employee/123"))
        .send()
        .await
        .expect("Failed to send DELETE request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_employee_duplicate_email() {
    let (db_pool, _meili_client, server_url, server_handle) =
        setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    let email = SafeEmail().fake::<String>();
    
    // Clean up
    let backend: sea_orm::DatabaseBackend = db_pool.get_database_backend();
    let _ = db_pool
        .execute(Statement::from_string(
            backend,
            format!("DELETE FROM employee where email = '{email}'"),
        ))
        .await;

    // Create first employee
    let employee1 = json!({
        "name": Name().fake::<String>(),
        "role": "Engineer",
        "email": email.clone()
    });

    let response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&employee1)
        .send()
        .await
        .expect("Failed to create first employee");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Try to create second employee with same email
    let employee2 = json!({
        "name": Name().fake::<String>(),
        "role": "Manager",
        "email": email.clone()
    });

    let response = client
        .post(format!("{server_url}/v1/employee/create"))
        .bearer_auth(&token)
        .json(&employee2)
        .send()
        .await
        .expect("Failed to send second create request");

    // Current implementation has database constraint on email, so expect 500
    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}