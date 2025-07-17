use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;

use crate::error::ApiError;
use super::models::{CreateOrder, Order};
use entity::order;

#[utoipa::path(
    post,
    path = "/v1/order/create",
    request_body = CreateOrder,
    responses(
        (status = 200, description = "Order created successfully", body = Order),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_order(
    data: web::Data<config::app::AppState>,
    order: web::Json<CreateOrder>,
) -> Result<HttpResponse, ApiError> {
    if order.total_amount < 0.0 {
        return Err(ApiError::ValidationError("Total amount cannot be negative".to_string()));
    }

    let new_uuid = Uuid::new_v4();
    let now = Utc::now().naive_utc();
    let new_order = order::ActiveModel {
        id: Set(new_uuid.to_string()),
        customer_id: Set(order.customer_id.clone()),
        total_amount: Set(order.total_amount),
        created_at: Set(now),
    };

    let inserted_order = new_order.insert(&data.db).await?;

    let order_response: Order = inserted_order;
    Ok(HttpResponse::Ok().json(order_response))
}