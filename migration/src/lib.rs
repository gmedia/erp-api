pub use sea_orm_migration::prelude::*;

mod m20250604_000000_create_inventory;
mod m20250604_000001_create_employee;
mod m20250604_000002_create_order;
mod m20250604_000003_create_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250604_000000_create_inventory::Migration),
            Box::new(m20250604_000001_create_employee::Migration),
            Box::new(m20250604_000002_create_order::Migration),
            Box::new(m20250604_000003_create_user::Migration),
        ]
    }
}
