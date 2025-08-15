use crate::error::ApiError;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::info;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::sync::Arc;
use std::task::{Context, Poll};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtMiddleware {
    token_prefix: String,
}

impl JwtMiddleware {
    pub fn new(token_prefix: String) -> Self {
        JwtMiddleware { token_prefix }
    }
}

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
            service: Arc::new(service),
            token_prefix: self.token_prefix.clone(),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Arc<S>,
    token_prefix: String,
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
        let token_prefix = self.token_prefix.clone();

        Box::pin(async move {
            match Self::verify_token(&req, &token_prefix) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    service.call(req).await
                }
                Err(e) => {
                    info!("Error verifying token: {}", e);
                    Err(e.into())
                }
            }
        })
    }
}

impl<S> JwtMiddlewareService<S> {
    fn extract_token<'a>(req: &'a ServiceRequest, token_prefix: &'a str) -> Option<&'a str> {
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix(token_prefix).map(|s| s.trim()))
    }

    fn verify_token(req: &ServiceRequest, token_prefix: &str) -> Result<Claims, ApiError> {
        let data = req
            .app_data::<actix_web::web::Data<config::app::AppState>>()
            .ok_or(ApiError::InternalServerError)?;

        let token = match Self::extract_token(req, token_prefix) {
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
            Err(e) => {
                info!("Error decoding token: {}", e);
                Err(ApiError::Unauthorized("Invalid token".to_string()))
            }
        }
    }
}
