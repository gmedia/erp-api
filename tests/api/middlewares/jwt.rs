use api::middlewares::jwt::Claims;
use jsonwebtoken::{EncodingKey, Header, encode};
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::helper::TestAppBuilder;

fn create_token(sub: &str, secret: &str, exp: usize) -> String {
    let claims = Claims {
        sub: sub.to_owned(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

#[tokio::test]
async fn test_logic() {
    let secret = "my-super-secret-key-that-is-long-enough".to_string();
    let app = TestAppBuilder::new()
        .jwt_secret(secret.clone())
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = reqwest::Client::new();

    // Test case 1: Valid token
    let exp = (SystemTime::now() + Duration::from_secs(30))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &secret, exp);
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("user1"));

    // Test case 2: No token
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 3: Invalid token
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth("invalid-token")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 4: Expired token
    let exp = (SystemTime::now() - Duration::from_secs(60))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &secret, exp);
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 5: Malformed Authorization header (not a valid UTF-8 string)
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_bytes(b"Bearer \x80").unwrap(),
    );
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .headers(headers)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 6: Wrong scheme in Authorization header
    let exp = (SystemTime::now() + Duration::from_secs(30))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &secret, exp);
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .header("Authorization", format!("Basic {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 7: Token with wrong secret
    let wrong_secret = "another-secret-that-is-definitely-not-right".to_string();
    let exp = (SystemTime::now() + Duration::from_secs(30))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &wrong_secret, exp);
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_no_app_state() {
    let app = TestAppBuilder::new()
        .skip_app_state()
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .bearer_auth("some-token")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 500);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_invalid_utf8_header() {
    let secret = "my-super-secret-key-that-is-long-enough".to_string();
    let app = TestAppBuilder::new()
        .jwt_secret(secret.clone())
        .build()
        .await
        .expect("Failed to build test app");

    let server_url = &app.server_url;
    let server_handle = &app.server_handle;
    
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_bytes(b"Bearer \x80").unwrap(),
    );
    let res = client
        .get(format!("{server_url}/v1/auth/me"))
        .headers(headers)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    server_handle.stop(true).await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
