use api::v1::auth::models::TokenResponse;
use entity::user::{self, Entity as User};
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use reqwest::header::{HeaderMap, HeaderValue};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;


use crate::helper::{get_auth_token, TestAppBuilder};

#[tokio::test]
async fn test_register_and_login() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Test registration
    let register_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send registration request");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Test login
    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let token_response: TokenResponse = response
        .json()
        .await
        .expect("Failed to parse token response");

    assert!(!token_response.token.is_empty());

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Access protected route with token
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Access protected route without token
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_register_existing_user() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    let register_req = json!({
        "username": &username,
        "password": password,
    });

    // First registration
    let response = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send registration request");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);

    // Second registration with the same username
    let response = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send second registration request");

    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_login_non_existent_user() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = "Not User".to_string();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_login_wrong_password() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";
    let wrong_password = "wrongpassword";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Register user
    let register_req = json!({
        "username": &username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    // Attempt to login with wrong password
    let login_req = json!({
        "username": username,
        "password": wrong_password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route_invalid_token() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;

    let client = HttpClient::new();
    let invalid_token = "this.is.an.invalid.token";

    // Access protected route with invalid token
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(invalid_token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_login_db_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Register user
    let register_req = json!({
        "username": &username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    // Close the database connection to simulate a database error
    app.db.close().await.unwrap();

    // Attempt to login
    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route_malformed_header() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;

    let client = HttpClient::new();

    // Access protected route with a malformed token
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .header("Authorization", "NotBearer token")
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route_expired_token() {
    let app = TestAppBuilder::new()
        .jwt_expires_in_seconds(1)
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;
    
    let client = HttpClient::new();
    let token = get_auth_token(&client, &server_url, &db_pool).await;

    // Wait for the token to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Access protected route with expired token
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route_no_app_state() {
    let app = TestAppBuilder::new()
        .skip_app_state()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;

    let client = HttpClient::new();

    // Access protected route without app state configured
    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth("some-token")
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_access_protected_route_invalid_utf8_in_header() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;

    let client = HttpClient::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_bytes(b"Bearer \x80").unwrap(),
    );

    let response = client
        .get(format!("{server_url}/v1/auth/me"))
        .headers(headers)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_register_invalid_bcrypt_cost() {
    let app = TestAppBuilder::new()
        .bcrypt_cost(99)
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;
    
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    let register_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send registration request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_login_invalid_jwt_secret() {
    let app = TestAppBuilder::new()
        .jwt_secret("".to_string())
        .jwt_algorithm(jsonwebtoken::Algorithm::RS256)
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;
    
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Register user
    let register_req = json!({
        "username": &username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    // Attempt to login
    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_register_db_error() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Close the database connection to simulate a database error
    app.db.close().await.unwrap();

    let register_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .expect("Failed to send registration request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_login_malformed_hash() {
    let app = TestAppBuilder::new()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    let db_pool = &app.db;

    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Clean user using entity-based approach
    let _ = User::delete_many()
        .filter(user::Column::Username.eq(&username))
        .exec(db_pool)
        .await;

    // Register user
    let register_req = json!({
        "username": &username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    // Find the user and update the password to a malformed hash
    let user = User::find()
        .filter(user::Column::Username.eq(&username))
        .one(db_pool)
        .await
        .unwrap()
        .unwrap();

    let mut active_user: user::ActiveModel = user.into();
    active_user.password = Set("not_a_real_hash".to_owned());
    User::update(active_user).exec(db_pool).await.unwrap();

    // Attempt to login
    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
