use api::v1::auth::models::TokenResponse;
use fake::{Fake, faker::internet::en::SafeEmail};
use reqwest::Client as HttpClient;
use serde_json::json;
use entity::user::{self, Entity as User};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

mod common;
use common::{setup_test_app, setup_test_app_no_state};

#[actix_rt::test]
async fn test_register_and_login() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
}

#[actix_rt::test]
async fn test_access_protected_route() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Register and login to get a token
    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    let token = token_response.token;

    // Access protected route with token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Access protected route without token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_register_existing_user() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
}

#[actix_rt::test]
async fn test_login_non_existent_user() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
}

#[actix_rt::test]
async fn test_login_wrong_password() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";
    let wrong_password = "wrongpassword";

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
}

#[actix_rt::test]
async fn test_access_protected_route_invalid_token() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let invalid_token = "this.is.an.invalid.token";

    // Access protected route with invalid token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth(invalid_token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_login_db_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
    db_pool.close().await.unwrap();

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
}

#[actix_rt::test]
async fn test_access_protected_route_malformed_header() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();

    // Access protected route with a malformed token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .header("Authorization", "NotBearer token")
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_access_protected_route_expired_token() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(Some(1), None, None, None).await; // 1 second token validity
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Register and login to get a token
    let register_req = json!({
        "username": username,
        "password": password,
    });

    client
        .post(format!("{server_url}/v1/auth/register"))
        .json(&register_req)
        .send()
        .await
        .unwrap();

    let login_req = json!({
        "username": username,
        "password": password,
    });

    let response = client
        .post(format!("{server_url}/v1/auth/login"))
        .json(&login_req)
        .send()
        .await
        .unwrap();

    let token_response: TokenResponse = response.json().await.unwrap();
    let token = token_response.token;

    // Wait for the token to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Access protected route with expired token
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_access_protected_route_no_app_state() {
    let (_db_pool, _meili_client, server_url) = setup_test_app_no_state().await;
    let client = HttpClient::new();

    // Access protected route without app state configured
    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .bearer_auth("some-token")
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}
use reqwest::header::{HeaderMap, HeaderValue};

#[actix_rt::test]
async fn test_access_protected_route_invalid_utf8_in_header() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_bytes(b"Bearer \x80").unwrap(),
    );

    let response = client
        .get(format!("{server_url}/v1/inventory/search?q=test"))
        .headers(headers)
        .send()
        .await
        .expect("Failed to send request to protected route");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_register_invalid_bcrypt_cost() {
    // bcrypt cost must be between 4 and 31.
    let invalid_cost = 99;
    let (_db_pool, _meili_client, server_url) =
        setup_test_app(None, Some(invalid_cost), None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
}

#[actix_rt::test]
async fn test_login_invalid_jwt_secret() {
    let (_db_pool, _meili_client, server_url) = setup_test_app(
        None,
        None,
        Some("".to_string()),
        Some(jsonwebtoken::Algorithm::RS256),
    )
    .await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
}

#[actix_rt::test]
async fn test_register_db_error() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

    // Close the database connection to simulate a database error
    db_pool.close().await.unwrap();

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
}

#[actix_rt::test]
async fn test_login_malformed_hash() {
    let (db_pool, _meili_client, server_url) = setup_test_app(None, None, None, None).await;
    let client = HttpClient::new();
    let username: String = SafeEmail().fake();
    let password = "password123";

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
        .one(&db_pool)
        .await
        .unwrap()
        .unwrap();

    let mut active_user: user::ActiveModel = user.into();
    active_user.password = Set("not_a_real_hash".to_owned());
    User::update(active_user).exec(&db_pool).await.unwrap();

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
}
