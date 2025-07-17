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
