use actix_web::{web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;

use super::models::{CreateOrder, Order};
use entity::order;
use sea_orm::DatabaseConnection;

#[utoipa::path(
    post,
    path = "/v1/order/create",
    request_body = CreateOrder,
    responses(
        (status = 200, description = "Order created successfully", body = Order),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_order(
    db: web::Data<DatabaseConnection>,
    order: web::Json<CreateOrder>,
) -> impl Responder {
    if order.total_amount < 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Total amount cannot be negative" }));
    }

    let new_uuid = Uuid::new_v4();
    let now = Utc::now().naive_utc();
    let new_order = order::ActiveModel {
        id: Set(new_uuid.to_string()),
        customer_id: Set(order.customer_id.clone()),
        total_amount: Set(order.total_amount),
        created_at: Set(now),
    };

    let result = new_order.insert(db.get_ref()).await;

    match result {
        Ok(inserted_order) => {
            let order_response: Order = inserted_order.into();
            HttpResponse::Ok().json(order_response)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}