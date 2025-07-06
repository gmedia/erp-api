use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use crate::error::ApiError;
use super::models::{CreateEmployee, Employee};
use entity::employee;

#[utoipa::path(
    post,
    path = "/v1/employee/create",
    request_body = CreateEmployee,
    responses(
        (status = 200, description = "Employee created successfully", body = Employee),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_employee(
    data: web::Data<config::app::AppState>,
    employee: web::Json<CreateEmployee>,
) -> Result<HttpResponse, ApiError> {
    if !employee.email.contains('@') {
        return Err(ApiError::ValidationError("Invalid email format".to_string()));
    }

    let new_uuid = Uuid::new_v4();
    let new_employee = employee::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(employee.name.clone()),
        role: Set(employee.role.clone()),
        email: Set(employee.email.clone()),
    };

    let inserted_employee = new_employee.insert(&data.db).await?;

    let employee_response: Employee = inserted_employee.into();
    Ok(HttpResponse::Ok().json(employee_response))
}