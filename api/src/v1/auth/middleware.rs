use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use crate::error::ApiError;
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            match Self::verify_token(&req) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    service.call(req).await
                }
                Err(e) => Err(e.into()),
            }
        })
    }
}

impl<S> JwtMiddlewareService<S> {
    fn extract_token(req: &ServiceRequest) -> Option<&str> {
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
    }

    fn verify_token(req: &ServiceRequest) -> Result<Claims, ApiError> {
        let data = req
            .app_data::<actix_web::web::Data<config::app::AppState>>()
            .ok_or(ApiError::InternalServerError)?;

        let token = match Self::extract_token(req) {
            Some(token) => token,
            None => return Err(ApiError::Unauthorized("Missing token".to_string())),
        };

        let mut validation = Validation::new(data.jwt_algorithm);
        validation.validate_exp = true;
        validation.leeway = 0;

        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(data.jwt_secret.as_ref()),
            &validation,
        ) {
            Ok(token_data) => Ok(token_data.claims),
            Err(_) => Err(ApiError::Unauthorized("Invalid token".to_string())),
        }
    }
}