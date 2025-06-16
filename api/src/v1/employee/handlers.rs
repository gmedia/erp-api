use actix_web::{web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use super::models::{CreateEmployee, Employee};
use entity::employee;
use sea_orm::DatabaseConnection;

#[utoipa::path(
    post,
    path = "/employee/create",
    request_body = CreateEmployee,
    responses(
        (status = 200, description = "Employee created successfully", body = Employee),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearerAuth" = [])
    )
)]
pub async fn create_employee(
    db: web::Data<DatabaseConnection>,
    employee: web::Json<CreateEmployee>,
) -> impl Responder {
    if !employee.email.contains('@') {
        return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid email format" }));
    }

    let new_uuid = Uuid::new_v4();
    let new_employee = employee::ActiveModel {
        id: Set(new_uuid.to_string()),
        name: Set(employee.name.clone()),
        role: Set(employee.role.clone()),
        email: Set(employee.email.clone()),
    };

    let result = new_employee.insert(db.get_ref()).await;

    match result {
        Ok(inserted_employee) => {
            let employee_response: Employee = inserted_employee.into();
            HttpResponse::Ok().json(employee_response)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}