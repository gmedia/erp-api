use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Queryable, Insertable, ToSchema)]
#[diesel(table_name = crate::db::schema::inventory)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub quantity: i32,
    pub price: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateInventoryItem {
    pub name: String,
    pub quantity: i32,
    pub price: f64,
}