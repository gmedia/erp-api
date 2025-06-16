use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use entity::inventory;

pub type InventoryItem = inventory::Model;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateInventoryItem {
    pub name: String,
    pub quantity: i32,
    pub price: f64,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateInventoryItem {
    pub name: Option<String>,
    pub quantity: Option<i32>,
    pub price: Option<f64>,
}