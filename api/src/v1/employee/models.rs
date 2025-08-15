use entity::employee;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type Employee = employee::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateEmployee {
    pub name: String,
    pub role: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateEmployee {
    pub name: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
}
