pub use sea_orm_migration::prelude::*;

mod m20250604_000000_create_inventory;
mod m20250604_000001_create_employees;
mod m20250604_000002_create_orders;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250604_000000_create_inventory::Migration),
            Box::new(m20250604_000001_create_employees::Migration),
            Box::new(m20250604_000002_create_orders::Migration),
        ]
    }
}
