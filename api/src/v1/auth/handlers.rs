use crate::error::ApiError;
use crate::middlewares::jwt::Claims;
use crate::v1::auth::models::{LoginRequest, RefreshRequest, RegisterRequest, TokenResponse};
use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify};
use config::app::AppState;
use entity::user::{self, Entity as User};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn register(
    data: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let db = &data.db;
    let hashed_password =
        hash(&req.password, data.bcrypt_cost).map_err(|_| ApiError::InternalServerError)?;

    let new_user = user::ActiveModel {
        id: sea_orm::ActiveValue::Set(Uuid::new_v4().to_string()),
        username: sea_orm::ActiveValue::Set(req.username.clone()),
        password: sea_orm::ActiveValue::Set(hashed_password),
        created_at: sea_orm::ActiveValue::Set(chrono::Utc::now()),
        updated_at: sea_orm::ActiveValue::Set(chrono::Utc::now()),
    };

    User::insert(new_user).exec(db).await.map_err(|db_err| {
        if let DbErr::Exec(ref err_msg) = db_err {
            if err_msg.to_string().contains("Duplicate entry") {
                return ApiError::Conflict("User already exists".to_string());
            }
        }
        ApiError::DatabaseError(db_err)
    })?;

    Ok(HttpResponse::Created().finish())
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn login(
    data: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let db = &data.db;
    log::info!("Attempting to log in user: {}", &req.username);

    let user = User::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(db)
        .await?
        .ok_or_else(|| {
            log::info!("User not found: {}", &req.username);
            ApiError::Unauthorized("Invalid username or password".to_string())
        })?;

    log::info!("User found: {:?}", &user);
    let password_hash = user.password.clone();

    let valid_password =
        verify(&req.password, &password_hash).map_err(|_| ApiError::InternalServerError)?;

    if !valid_password {
        log::warn!("Password verification failed for user: {}", &req.username);
        return Err(ApiError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(
            data.jwt_expires_in_seconds as i64,
        ))
        .ok_or(ApiError::InternalServerError)?
        .timestamp();

    let claims = Claims {
        sub: user.id.clone(),
        exp: exp as usize,
    };

    let token = encode(
        &Header::new(data.jwt_algorithm),
        &claims,
        &EncodingKey::from_secret(data.jwt_secret.as_ref()),
    )
    .map_err(|_| ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().json(TokenResponse { token }))
}

#[utoipa::path(
    get,
    path = "/v1/auth/me",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Authenticated user data", body = Claims)
    )
)]
pub async fn me(claims: web::ReqData<Claims>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(claims.into_inner()))
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn logout(
    _claims: web::ReqData<Claims>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // In a real implementation, you would invalidate the token
    // For now, we'll just return success
    Ok(HttpResponse::Ok().json(json!({"message": "Logged out successfully"})))
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout-all",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "All sessions logged out successfully"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn logout_all(
    _claims: web::ReqData<Claims>,
    _data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // In a real implementation, you would invalidate all tokens for this user
    Ok(HttpResponse::Ok().json(json!({"message": "All sessions logged out successfully"})))
}

#[utoipa::path(
    post,
    path = "/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = TokenResponse),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn refresh(
    data: web::Data<AppState>,
    req: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let token = &req.refresh_token;

    // Decode and validate the refresh token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(data.jwt_secret.as_ref()),
        &Validation::new(data.jwt_algorithm),
    )
    .map_err(|_| ApiError::Unauthorized("Invalid refresh token".to_string()))?;

    let claims = token_data.claims;

    // Check if user still exists
    let user = User::find_by_id(&claims.sub)
        .one(&data.db)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;

    // Generate new token
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(
            data.jwt_expires_in_seconds as i64,
        ))
        .ok_or(ApiError::InternalServerError)?
        .timestamp();

    let new_claims = Claims {
        sub: user.id.clone(),
        exp: exp as usize,
    };

    let new_token = encode(
        &Header::new(data.jwt_algorithm),
        &new_claims,
        &EncodingKey::from_secret(data.jwt_secret.as_ref()),
    )
    .map_err(|_| ApiError::InternalServerError)?;

    Ok(HttpResponse::Ok().json(TokenResponse { token: new_token }))
}
