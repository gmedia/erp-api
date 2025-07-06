use api::v1::auth::middleware::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
mod common;
use common::{setup_test_app, setup_test_app_no_state};
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::{ready, Ready};
use futures_util::task::noop_waker;
use std::task::{Context, Poll};
use api::v1::auth::middleware::JwtMiddleware;

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

#[actix_rt::test]
async fn test_jwt_middleware_logic() {
    let secret = "my-super-secret-key-that-is-long-enough".to_string();
    let (_db, _meili, server_url) =
        setup_test_app(None, None, Some(secret.clone()), None).await;
    let client = reqwest::Client::new();

    // Test case 1: Valid token
    let exp = (SystemTime::now() + Duration::from_secs(30))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &secret, exp);
    let res = client
        .get(format!("{}/v1/auth/me", server_url))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("user1"));

    // Test case 2: No token
    let res = client
        .get(format!("{}/v1/auth/me", server_url))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // Test case 3: Invalid token
    let res = client
        .get(format!("{}/v1/auth/me", server_url))
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
        .get(format!("{}/v1/auth/me", server_url))
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
        .get(format!("{}/v1/auth/me", server_url))
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
        .get(format!("{}/v1/auth/me", server_url))
        .header("Authorization", format!("Basic {}", token))
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
        .get(format!("{}/v1/auth/me", server_url))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[actix_rt::test]
async fn test_jwt_middleware_no_app_state() {
    let (_db, _meili, server_url) = setup_test_app_no_state().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/v1/auth/me", server_url))
        .bearer_auth("some-token")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 500);
}

use actix_web::{test, web};
use config::app::AppState;
use config::{db::Db, meilisearch::Meilisearch};
use db::mysql::init_db_pool;
use dotenv::dotenv;

#[actix_rt::test]
async fn test_jwt_middleware_call_logic() {
    dotenv().ok();
    let secret = "my-super-secret-key-that-is-long-enough".to_string();
    let config_db = Db::new("test");
    let config_meilisearch = Meilisearch::new("test");
    let db_pool = init_db_pool(&config_db.url).await.unwrap();
    let meili_client =
        search::meilisearch::init_meilisearch(&config_meilisearch.host, &config_meilisearch.api_key)
            .await
            .unwrap();
    let app_state = web::Data::new(AppState {
        db: db_pool,
        meilisearch: meili_client,
        jwt_secret: secret.clone(),
        jwt_expires_in_seconds: 3600,
        bcrypt_cost: 4,
        jwt_algorithm: jsonwebtoken::Algorithm::HS256,
    });

    let middleware = JwtMiddleware;
    let service = MockService;
    let middleware_service = middleware.new_transform(service).await.unwrap();

    // Test case: No Authorization header
    let req = test::TestRequest::default()
        .app_data(app_state.clone())
        .to_srv_request();
    let err = middleware_service.call(req).await.err().unwrap();
    assert_eq!(err.as_response_error().status_code(), 401);

    // Test case: Malformed header (no "Bearer " prefix)
    let req = test::TestRequest::default()
        .insert_header(("Authorization", "Token some-token"))
        .app_data(app_state.clone())
        .to_srv_request();
    let err = middleware_service.call(req).await.err().unwrap();
    assert_eq!(err.as_response_error().status_code(), 401);

    // Test case: Invalid token
    let req = test::TestRequest::default()
        .insert_header(("Authorization", "Bearer invalid-token"))
        .app_data(app_state.clone())
        .to_srv_request();
    let err = middleware_service.call(req).await.err().unwrap();
    assert_eq!(err.as_response_error().status_code(), 401);

    // Test case: Token with wrong secret
    let wrong_secret = "another-secret".to_string();
    let exp = (SystemTime::now() + Duration::from_secs(30))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let token = create_token("user1", &wrong_secret, exp);
    let req = test::TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .app_data(app_state.clone())
        .to_srv_request();
    let err = middleware_service.call(req).await.err().unwrap();
    assert_eq!(err.as_response_error().status_code(), 401);
}

struct MockService;
impl Service<ServiceRequest> for MockService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        ready(Ok(req.into_response(
            actix_web::HttpResponse::Ok()
                .finish()
                .map_into_boxed_body(),
        )))
    }
}

#[actix_rt::test]
async fn test_jwt_middleware_poll_ready_cover() {
    let middleware = JwtMiddleware;
    let service = MockService;
    let middleware_service = middleware.new_transform(service).await.unwrap();

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    let poll_result = middleware_service.poll_ready(&mut cx);
    assert!(poll_result.is_ready());
}