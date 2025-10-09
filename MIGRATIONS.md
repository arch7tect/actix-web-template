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
│   └── m20250109_000001_create_memos_table.rs  # Individual migrations
```

## Running Migrations

**IMPORTANT: Always run migration commands from the project root directory, not from migration/ subdirectory**

### Apply all pending migrations

```bash
cd migration
cargo run -- up
```

### Check migration status

```bash
cd migration
cargo run -- status
```

### Rollback last migration

```bash
cd migration
cargo run -- down
```

### Reset database (rollback all, then apply all)

```bash
cd migration
cargo run -- fresh
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

### Manual approach

1. Create new migration file in `migration/src/`:
   ```
   m20YYMMDD_HHMMSS_description.rs
   ```

2. Add migration to `migration/src/lib.rs`:
   ```rust
   mod m20YYMMDD_HHMMSS_description;

   impl MigratorTrait for Migrator {
       fn migrations() -> Vec<Box<dyn MigrationTrait>> {
           vec![
               Box::new(m20250109_000001_create_memos_table::Migration),
               Box::new(m20YYMMDD_HHMMSS_description::Migration),  // Add here
           ]
       }
   }
   ```

3. Implement the migration (see existing migrations for examples)

## Common Mistakes to Avoid

❌ **Running migrations from wrong directory**
```bash
cd migration
cargo run  # Wrong - entities won't be in right place
```

✅ **Run from migration directory for migrations**
```bash
cd migration
cargo run -- up  # Correct
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

1. **Start database**: `docker-compose up -d postgres`
2. **Create migration**: Add new file in `migration/src/`
3. **Register migration**: Update `migration/src/lib.rs`
4. **Run migration**: `cd migration && cargo run -- up`
5. **Generate entities**: `sea-orm-cli generate entity --database-url $DATABASE_URL -o src/entities --with-serde both` (from project root)
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