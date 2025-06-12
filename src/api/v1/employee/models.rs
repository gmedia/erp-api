use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::entities::employees;

pub type Employee = employees::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateEmployee {
    pub name: String,
    pub role: String,
    pub email: String,
}