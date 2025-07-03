use actix_web::dev::Transform;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse},
    http, test, web, App, Error, HttpResponse,
};
use api::v1::auth::middleware::JwtMiddleware;
use futures_util::task::noop_waker_ref;
use std::cell::Cell;
use std::future::{ready, Ready};
use std::task::{Context, Poll};

struct NotReadyService {
    calls: Cell<u8>,
}

impl Service<ServiceRequest> for NotReadyService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.calls.get() < 1 {
            self.calls.set(self.calls.get() + 1);
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        ready(Ok(req.into_response(HttpResponse::Ok().finish())))
    }
}

#[actix_rt::test]
async fn test_poll_ready_is_covered() {
    let middleware = JwtMiddleware;
    let not_ready_service = NotReadyService {
        calls: Cell::new(0),
    };
    let transformed_service = middleware.new_transform(not_ready_service).await.unwrap();

    let mut ctx = Context::from_waker(&noop_waker_ref());

    // First poll is pending
    assert!(transformed_service.poll_ready(&mut ctx).is_pending());
    // Second poll is ready
    assert!(transformed_service.poll_ready(&mut ctx).is_ready());
}
struct ErrorService;

impl Service<ServiceRequest> for ErrorService {
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Err(actix_web::error::ErrorInternalServerError("Service not ready")))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        ready(Ok(req.into_response(HttpResponse::Ok().finish())))
    }
}

#[actix_rt::test]
async fn test_service_error_is_propagated() {
    let middleware = JwtMiddleware;
    let error_service = ErrorService;
    let transformed_service = middleware.new_transform(error_service).await.unwrap();

    let mut ctx = Context::from_waker(&noop_waker_ref());
    let poll_result = transformed_service.poll_ready(&mut ctx);

    assert!(matches!(poll_result, Poll::Ready(Err(_))));
}