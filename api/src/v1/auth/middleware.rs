use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let data = match req.app_data::<actix_web::web::Data<config::app::AppState>>() {
            Some(data) => data.clone(),
            None => {
                return Box::pin(async {
                    Err(actix_web::error::ErrorInternalServerError(
                        "App state not configured",
                    ))
                });
            }
        };

        let token = match req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
        {
            Some(token) => token.to_string(),
            None => {
                return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Missing token")) });
            }
        };

        let claims = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(data.jwt_secret.as_ref()),
            &{
                let mut val = Validation::new(jsonwebtoken::Algorithm::HS256);
                val.validate_exp = true;
                val.leeway = 0;
                val
            },
        ) {
            Ok(token_data) => token_data.claims,
            Err(_) => {
                return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Invalid token")) });
            }
        };

        req.extensions_mut().insert(claims);
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}