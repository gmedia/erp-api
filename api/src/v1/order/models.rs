use entity::order;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub type Order = order::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateOrder {
    pub customer_id: String,
    pub total_amount: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateOrder {
    pub customer_id: Option<String>,
    pub total_amount: Option<f64>,
}
