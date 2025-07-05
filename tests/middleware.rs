use api::v1::auth::middleware::Claims;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
mod common;
use common::setup_test_app;
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