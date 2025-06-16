use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use entity::orders;

pub type Order = orders::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateOrder {
    pub customer_id: String,
    pub total_amount: f64,
}