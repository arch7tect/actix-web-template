use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tags table
        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tags::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        ColumnDef::new(Tags::Name)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Tags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on tag name for fast lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_tags_name")
                    .table(Tags::Table)
                    .col(Tags::Name)
                    .to_owned(),
            )
            .await?;

        // Create memo_tags junction table
        manager
            .create_table(
                Table::create()
                    .table(MemoTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MemoTags::MemoId).uuid().not_null())
                    .col(ColumnDef::new(MemoTags::TagId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(MemoTags::MemoId)
                            .col(MemoTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_memo_tags_memo_id")
                            .from(MemoTags::Table, MemoTags::MemoId)
                            .to(Memos::Table, Memos::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_memo_tags_tag_id")
                            .from(MemoTags::Table, MemoTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes on junction table for efficient queries
        manager
            .create_index(
                Index::create()
                    .name("idx_memo_tags_memo_id")
                    .table(MemoTags::Table)
                    .col(MemoTags::MemoId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_memo_tags_tag_id")
                    .table(MemoTags::Table)
                    .col(MemoTags::TagId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order (junction table first due to foreign keys)
        manager
            .drop_table(Table::drop().table(MemoTags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum MemoTags {
    Table,
    MemoId,
    TagId,
}

#[derive(DeriveIden)]
enum Memos {
    Table,
    Id,
}
