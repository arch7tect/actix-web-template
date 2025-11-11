# Database Migrations Guide

This guide explains how to work with SeaORM migrations in this project.

## Prerequisites

- PostgreSQL running (via `docker-compose up -d postgres`)
- `DATABASE_URL` environment variable set in `.env`

## Directory Structure

```
migration/
├── Cargo.toml          # Migration crate dependencies
├── src/
│   ├── lib.rs          # Migration registry
│   ├── main.rs         # Migration CLI entry point
│   ├── m20250109_000001_create_memos_table.rs  # Initial memos table
│   └── m20250110_000001_create_tags_tables.rs  # Tags and memo_tags tables (Chapter 18)
```

## Running Migrations

With the workspace setup, you can run migrations from the project root:

### Apply all pending migrations

```bash
cargo run -p migration -- up
```

### Check migration status

```bash
cargo run -p migration -- status
```

### Rollback last migration

```bash
cargo run -p migration -- down
```

### Reset database (rollback all, then apply all)

```bash
cargo run -p migration -- fresh
```

**Alternative**: You can still run from the migration directory if needed:

```bash
cd migration && cargo run -- up
```

## Generating Entities

**IMPORTANT: Always generate entities from the project root directory**

After running migrations, generate SeaORM entities:

```bash
# From project root
sea-orm-cli generate entity \
  --database-url postgresql://postgres:postgres@localhost:5432/memos_db \
  -o src/entities \
  --with-serde both
```

Or use environment variable:

```bash
# From project root (requires DATABASE_URL in .env)
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/memos_db
sea-orm-cli generate entity -o src/entities --with-serde both
```

## Creating New Migrations

### Using SeaORM CLI

Generate a new migration using the CLI from the migration directory:

```bash
cd migration
sea-orm-cli migrate generate create_tags_tables
```

This creates a new migration file with a timestamp, e.g., `m20250110_000001_create_tags_tables.rs`, and automatically registers it in `lib.rs`.

**Or from root** (if sea-orm-cli is installed globally):

```bash
cd migration && sea-orm-cli migrate generate create_tags_tables && cd ..
```

**Example from Chapter 18:**

```bash
cd migration
sea-orm-cli migrate generate create_tags_tables
```

Then implement the migration in the generated file:

```rust
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
                    .col(ColumnDef::new(Tags::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tags::Name).string().not_null().unique_key())
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
                    .primary_key(Index::create().col(MemoTags::MemoId).col(MemoTags::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(MemoTags::Table, MemoTags::MemoId)
                            .to(Memos::Table, Memos::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(MemoTags::Table, MemoTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(MemoTags::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Tags::Table).to_owned()).await
    }
}
```

## Common Mistakes to Avoid

❌ **Forgetting the `-p migration` flag**
```bash
cargo run -- up  # Wrong - runs main app, not migration
```

✅ **Use -p flag to specify package**
```bash
cargo run -p migration -- up  # Correct
```

❌ **Generating entities from migration directory**
```bash
cd migration
sea-orm-cli generate entity -o src/entities  # Wrong - wrong path
```

✅ **Generate from project root**
```bash
# From project root
sea-orm-cli generate entity --database-url <URL> -o src/entities --with-serde both
```

❌ **Forgetting database URL**
```bash
sea-orm-cli generate entity -o src/entities  # Will fail
```

✅ **Always provide database URL**
```bash
sea-orm-cli generate entity --database-url postgresql://postgres:postgres@localhost:5432/memos_db -o src/entities --with-serde both
```

## Workflow Summary

With workspace setup:

1. **Start database**: `docker-compose up -d postgres`
2. **Create migration**: `cd migration && sea-orm-cli migrate generate your_migration_name && cd ..`
3. **Implement migration**: Edit the generated file in `migration/src/`
4. **Run migration**: `cargo run -p migration -- up` (from root)
5. **Generate entities**: `sea-orm-cli generate entity --database-url $DATABASE_URL -o src/entities --with-serde both`
6. **Verify**: `cargo build` and test the application

## Troubleshooting

### "pathspec did not match any files"
- Make sure you're in the correct directory
- For migrations: must be in `migration/` directory
- For entity generation: must be in project root

### "could not connect to server"
- Check PostgreSQL is running: `docker-compose ps postgres`
- Verify DATABASE_URL is correct
- Check port 5432 is not in use by another process

### "entities not found after generation"
- Make sure you ran `sea-orm-cli` from project root, not `migration/` directory
- Check entities were created in `src/entities/`

## References

- [SeaORM Migration Docs](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [SeaORM CLI Docs](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/)