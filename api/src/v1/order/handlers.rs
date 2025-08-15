use actix_web::{web, HttpResponse};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use uuid::Uuid;

use super::models::{CreateOrder, Order, UpdateOrder};
use crate::error::ApiError;
use entity::order;
use serde_json::json;

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
        return Err(ApiError::ValidationError(
            "Total amount cannot be negative".to_string(),
        ));
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

#[utoipa::path(
    get,
    path = "/v1/order",
    responses(
        (status = 200, description = "List of orders", body = Vec<Order>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_all_orders(
    data: web::Data<config::app::AppState>,
) -> Result<HttpResponse, ApiError> {
    let orders = order::Entity::find()
        .order_by_asc(entity::order::Column::CreatedAt)
        .all(&data.db)
        .await?;

    let order_responses: Vec<Order> = orders
        .into_iter()
        .map(|o| o.into())
        .collect();

    Ok(HttpResponse::Ok().json(order_responses))
}

#[utoipa::path(
    get,
    path = "/v1/order/{id}",
    responses(
        (status = 200, description = "Order found", body = Order),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_order_by_id(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let order = order::Entity::find_by_id(id.into_inner())
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Order not found".to_string()))?;

    let order_response: Order = order.into();
    Ok(HttpResponse::Ok().json(order_response))
}

#[utoipa::path(
    put,
    path = "/v1/order/{id}",
    request_body = UpdateOrder,
    responses(
        (status = 200, description = "Order updated successfully", body = Order),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn update_order(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
    order: web::Json<UpdateOrder>,
) -> Result<HttpResponse, ApiError> {
    let order_id = id.into_inner();
    
    let existing_order = order::Entity::find_by_id(&order_id)
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Order not found".to_string()))?;

    let mut order_model: order::ActiveModel = existing_order.into();
    
    if let Some(customer_id) = &order.customer_id {
        order_model.customer_id = Set(customer_id.clone());
    }
    if let Some(total_amount) = order.total_amount {
        if total_amount < 0.0 {
            return Err(ApiError::ValidationError(
                "Total amount cannot be negative".to_string(),
            ));
        }
        order_model.total_amount = Set(total_amount);
    }

    let updated_order = order_model.update(&data.db).await?;
    let order_response: Order = updated_order.into();
    Ok(HttpResponse::Ok().json(order_response))
}

#[utoipa::path(
    delete,
    path = "/v1/order/{id}",
    responses(
        (status = 200, description = "Order deleted successfully"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn delete_order(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let order = order::Entity::find_by_id(id.into_inner())
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Order not found".to_string()))?;

    let order_active: order::ActiveModel = order.into();
    order_active.delete(&data.db).await?;
    Ok(HttpResponse::Ok().json(json!({"message": "Order deleted successfully"})))
}
