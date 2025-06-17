use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

pub async fn init_db_pool(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let mut opt = ConnectOptions::new(database_url.to_owned());
    // Hanya set opsi yang paling penting untuk menghindari timeout
    opt.max_connections(100)
       .acquire_timeout(Duration::from_secs(30))
       .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    Ok(db)
}