pub use sea_orm_migration::prelude::*;

mod m20250109_000001_create_memos_table;
mod m20250110_000001_create_tags_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250109_000001_create_memos_table::Migration),
            Box::new(m20250110_000001_create_tags_tables::Migration),
        ]
    }
}