use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Memos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Memos::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(Memos::Title)
                            .string_len(200)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Memos::Description).text())
                    .col(
                        ColumnDef::new(Memos::DateTo)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Memos::Completed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Memos::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(Memos::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_memos_date_to")
                    .table(Memos::Table)
                    .col(Memos::DateTo)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_memos_completed")
                    .table(Memos::Table)
                    .col(Memos::Completed)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_memos_created_at")
                    .table(Memos::Table)
                    .col(Memos::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Memos::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Memos {
    Table,
    Id,
    Title,
    Description,
    DateTo,
    Completed,
    CreatedAt,
    UpdatedAt,
}