use sea_orm::{Database, DatabaseConnection};

pub async fn init_db_pool(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}
