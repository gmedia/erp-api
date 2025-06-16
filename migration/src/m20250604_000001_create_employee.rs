use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Employee::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Employee::Id)
                            .char_len(36)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Employee::Name).string().not_null())
                    .col(ColumnDef::new(Employee::Role).string_len(100).not_null())
                    .col(
                        ColumnDef::new(Employee::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Employee::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Employee {
    Table,
    Id,
    Name,
    Role,
    Email,
}