use actix_web::{web, HttpResponse};
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use uuid::Uuid;

use super::models::{CreateEmployee, Employee, UpdateEmployee};
use crate::error::ApiError;
use entity::employee;
use serde_json::json;

#[utoipa::path(
    post,
    path = "/v1/employee",
    request_body = CreateEmployee,
    responses(
        (status = 201, description = "Employee created successfully", body = Employee),
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
        return Err(ApiError::ValidationError(
            "Invalid email format".to_string(),
        ));
    }

    let new_uuid = Uuid::new_v4();
    let new_employee = employee::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(employee.name.clone()),
        role: Set(employee.role.clone()),
        email: Set(employee.email.clone()),
    };

    let inserted_employee = new_employee.insert(&data.db).await?;

    let employee_response: Employee = inserted_employee;
    Ok(HttpResponse::Ok().json(employee_response))
}

#[utoipa::path(
    get,
    path = "/v1/employee",
    responses(
        (status = 200, description = "List of employees", body = Vec<Employee>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_all_employees(
    data: web::Data<config::app::AppState>,
) -> Result<HttpResponse, ApiError> {
    let employees = employee::Entity::find()
        .order_by_asc(entity::employee::Column::Name)
        .all(&data.db)
        .await?;

    let employee_responses: Vec<Employee> = employees.into_iter().map(|e| e.into()).collect();

    Ok(HttpResponse::Ok().json(employee_responses))
}

#[utoipa::path(
    get,
    path = "/v1/employee/{id}",
    responses(
        (status = 200, description = "Employee found", body = Employee),
        (status = 404, description = "Employee not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn get_employee_by_id(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let employee = employee::Entity::find_by_id(id.into_inner())
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Employee not found".to_string()))?;

    let employee_response: Employee = employee.into();
    Ok(HttpResponse::Ok().json(employee_response))
}

#[utoipa::path(
    put,
    path = "/v1/employee/{id}",
    request_body = UpdateEmployee,
    responses(
        (status = 200, description = "Employee updated successfully", body = Employee),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Employee not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn update_employee(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
    employee: web::Json<UpdateEmployee>,
) -> Result<HttpResponse, ApiError> {
    let employee_id = id.into_inner();

    let existing_employee = employee::Entity::find_by_id(&employee_id)
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Employee not found".to_string()))?;

    let mut employee_model: employee::ActiveModel = existing_employee.into();

    if let Some(name) = &employee.name {
        employee_model.name = Set(name.clone());
    }
    if let Some(role) = &employee.role {
        employee_model.role = Set(role.clone());
    }
    if let Some(email) = &employee.email {
        if !email.contains('@') {
            return Err(ApiError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }
        employee_model.email = Set(email.clone());
    }

    let updated_employee = employee_model.update(&data.db).await?;
    let employee_response: Employee = updated_employee.into();
    Ok(HttpResponse::Ok().json(employee_response))
}

#[utoipa::path(
    delete,
    path = "/v1/employee/{id}",
    responses(
        (status = 200, description = "Employee deleted successfully"),
        (status = 404, description = "Employee not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn delete_employee(
    data: web::Data<config::app::AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let employee = employee::Entity::find_by_id(id.into_inner())
        .one(&data.db)
        .await?
        .ok_or(ApiError::NotFound("Employee not found".to_string()))?;

    let employee_active: employee::ActiveModel = employee.into();
    employee_active.delete(&data.db).await?;
    Ok(HttpResponse::Ok().json(json!({"message": "Employee deleted successfully"})))
}
