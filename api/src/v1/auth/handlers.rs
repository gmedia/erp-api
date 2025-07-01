use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::v1::auth::models::{LoginRequest, RegisterRequest, TokenResponse};
use config::app::AppState;
use entity::{prelude::User, user};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "User already exists"),
    )
)]
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    let db = &data.db;
    let hashed_password = match hash(&req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(Uuid::new_v4().to_string()),
        username: sea_orm::ActiveValue::Set(req.username.clone()),
        password: sea_orm::ActiveValue::Set(hashed_password),
        created_at: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
        updated_at: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
    };

    match User::insert(new_user).exec(db).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::Conflict().finish(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn login(data: web::Data<AppState>, req: web::Json<LoginRequest>) -> impl Responder {
    let db = &data.db;
    log::info!("Attempting to log in user: {}", &req.username);
    let user = match User::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(db)
        .await
    {
        Ok(Some(user)) => {
            log::info!("User found: {:?}", &user);
            user
        }
        Ok(None) => {
            log::info!("User not found: {}", &req.username);
            return HttpResponse::Unauthorized().finish();
        }
        Err(e) => {
            log::error!("Database error during login: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let password_hash = user.password.clone();
    log::info!("Password hash from DB: {}", password_hash);
    if !verify(&req.password, &password_hash).unwrap_or(false) {
        log::warn!("Password verification failed for user: {}", &req.username);
        return HttpResponse::Unauthorized().finish();
    }

    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(
            data.jwt_expires_in_seconds as i64
        ))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.clone(),
        exp: exp as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(TokenResponse { token })
}