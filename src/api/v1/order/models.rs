use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::db::entities::orders;

pub type Order = orders::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateOrder {
    pub customer_id: String,
    pub total_amount: f64,
}