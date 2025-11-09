# Chapter 18: Adding Tags and Advanced Filtering

## Overview

In this chapter, you'll add a tagging system to your memo application. Users will be able to tag memos for better organization and filter memos by tags. This feature demonstrates many-to-many relationships in SeaORM, advanced querying patterns, and evolving APIs in a backward-compatible way.

Tags provide a flexible way to organize and find related memos without rigid categorization.

## Prerequisites

### Completed Chapters
- Chapters 0-17: Full application with observability

### Required Knowledge
- Understanding of many-to-many database relationships
- Junction tables and foreign keys
- SeaORM relationships and joins

### Required Software
- All previous tools from Chapters 0-17
- Database running (PostgreSQL)

## Learning Objectives

By the end of this chapter, you will:

- Design and implement many-to-many relationships
- Create junction tables with SeaORM migrations
- Work with related entities and joins
- Add features to existing APIs without breaking changes
- Implement tag filtering with OR logic
- Handle tag lifecycle (auto-creation, cleanup)
- Update both REST API and Web UI for tags

## Concepts Covered

### Many-to-Many Relationships

**The Problem:**
- Each memo can have multiple tags
- Each tag can belong to multiple memos
- This is a many-to-many (M:N) relationship

**The Solution:**
A junction table (also called join table or association table):

```
memos          memo_tags         tags
┌──────┐      ┌──────────┐      ┌──────┐
│ id   │◄────┤ memo_id  │      │ id   │
│ title│      │ tag_id   ├─────►│ name │
└──────┘      └──────────┘      └──────┘
```

**Example Data:**
```sql
-- Memos table
id: 1, title: "Buy groceries"
id: 2, title: "Doctor appointment"
id: 3, title: "Project deadline"

-- Tags table
id: A, name: "personal"
id: B, name: "urgent"
id: C, name: "work"

-- Memo_tags junction table
memo_id: 1, tag_id: A  -- Buy groceries is "personal"
memo_id: 1, tag_id: B  -- Buy groceries is "urgent"
memo_id: 2, tag_id: A  -- Doctor is "personal"
memo_id: 2, tag_id: B  -- Doctor is "urgent"
memo_id: 3, tag_id: C  -- Project is "work"
memo_id: 3, tag_id: B  -- Project is "urgent"
```

### Tag Management Strategies

**Tag Creation Approach (we'll use this):**
- **On-the-fly creation**: Tags are created automatically when creating/updating memos
- **Benefit**: User-friendly, no separate tag management needed
- **Challenge**: Need to handle duplicates, normalize names

**Tag Deletion Approach (we'll use this):**
- **Auto-cleanup**: Delete tags when no memos use them
- **Benefit**: No orphaned tags cluttering the database
- **Challenge**: Need to track usage counts

### Filtering Logic

**OR Logic (we'll implement this):**
- Return memos with ANY of the specified tags
- Query: `?tags=urgent,work`
- Result: Memos tagged "urgent" OR "work" OR both

**Example:**
```
Tags filter: "urgent,personal"
Results:
✓ Memo 1: ["personal", "urgent"]  -- Has both
✓ Memo 2: ["personal"]             -- Has "personal"
✓ Memo 3: ["urgent", "work"]       -- Has "urgent"
✗ Memo 4: ["work"]                 -- Has neither
```

## Step-by-Step Instructions

### Step 1: Create Database Migration

Create a migration to add the `tags` and `memo_tags` tables.

**Create migration file:**

```bash
# Create the migration file
touch migration/src/m20250110_000001_create_tags_tables.rs
```

**File: `migration/src/m20250110_000001_create_tags_tables.rs`**

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
```

**Key Points:**
- **Tags table**: Stores unique tag names
- **Junction table**: Links memos to tags with composite primary key
- **Foreign keys**: CASCADE delete - when memo/tag deleted, associations removed
- **Indexes**: Speed up lookups by memo_id and tag_id
- **Unique constraint**: Prevents duplicate tag names

**Register the migration in `migration/src/lib.rs`:**

```rust
pub use sea_orm_migration::prelude::*;

mod m20250109_000001_create_memos_table;
mod m20250110_000001_create_tags_tables;  // Add this

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250109_000001_create_memos_table::Migration),
            Box::new(m20250110_000001_create_tags_tables::Migration),  // Add this
        ]
    }
}
```

**Run the migration:**

```bash
cd migration
cargo run
cd ..
```

**Expected output:**
```
Applying migration 'm20250110_000001_create_tags_tables'
Migration 'm20250110_000001_create_tags_tables' has been applied
```

**Verify in database:**

```bash
psql $DATABASE_URL

\dt    -- List tables, should see: memos, tags, memo_tags
\d tags
\d memo_tags
\q
```

### Step 2: Create SeaORM Entities

Create entity files for the new tables.

**File: `src/entities/tags.rs`**

```rust
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::memo_tags::Entity")]
    MemoTags,
}

impl Related<super::memo_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemoTags.def()
    }
}

// Many-to-many relationship with memos through memo_tags
impl Related<super::memos::Entity> for Entity {
    fn to() -> RelationDef {
        super::memo_tags::Relation::Memos.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::memo_tags::Relation::Tags.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

**File: `src/entities/memo_tags.rs`**

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "memo_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub memo_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::memos::Entity",
        from = "Column::MemoId",
        to = "super::memos::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Memos,
    #[sea_orm(
        belongs_to = "super::tags::Entity",
        from = "Column::TagId",
        to = "super::tags::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Tags,
}

impl Related<super::memos::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Memos.def()
    }
}

impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

**Update `src/entities/memos.rs` to add tag relations:**

Add these relations to the existing memos entity:

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::memo_tags::Entity")]
    MemoTags,
}

impl Related<super::memo_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemoTags.def()
    }
}

// Many-to-many relationship with tags through memo_tags
impl Related<super::tags::Entity> for Entity {
    fn to() -> RelationDef {
        super::memo_tags::Relation::Tags.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::memo_tags::Relation::Memos.def().rev())
    }
}
```

**Update `src/entities/mod.rs`:**

```rust
pub mod prelude;

pub mod memo_tags;  // Add
pub mod memos;
pub mod tags;       // Add
```

**Update `src/entities/prelude.rs`:**

```rust
pub use super::memo_tags::Entity as MemoTags;  // Add
pub use super::memos::Entity as Memos;
pub use super::tags::Entity as Tags;           // Add
```

**Verify entities compile:**

```bash
cargo check
```

### Step 3: Update DTOs for Tags

Add tag fields to all memo DTOs to support tags in the API.

**Update `src/dto/memo_dto.rs`:**

Add tags to CreateMemoDto:

```rust
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateMemoDto {
    #[validate(length(
        min = 1,
        max = 200,
        message = "Title must be between 1 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,

    pub date_to: DateTime<Utc>,

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    #[serde(default)]
    pub tags: Vec<String>,  // Add this
}
```

Add tags to UpdateMemoDto:

```rust
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateMemoDto {
    // ... existing fields ...

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    #[serde(default)]
    pub tags: Vec<String>,  // Add this
}
```

Add tags to PatchMemoDto:

```rust
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PatchMemoDto {
    // ... existing fields ...

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    pub tags: Option<Vec<String>>,  // Add this
}
```

Add tags to MemoResponseDto:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MemoResponseDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date_to: DateTime<Utc>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,  // Add this
}
```

Add tags filter to PaginationParams:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    // ... existing fields ...

    /// Filter by tags (comma-separated), matches memos with ANY of the specified tags (OR logic)
    pub tags: Option<String>,  // Add this
}

impl PaginationParams {
    // ... existing methods ...

    /// Parse comma-separated tags into a vector
    pub fn parse_tags(&self) -> Option<Vec<String>> {
        self.tags.as_ref().map(|tags_str| {
            tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(10),
            offset: Some(0),
            completed: None,
            sort_by: Some("created_at".to_string()),
            order: Some("desc".to_string()),
            tags: None,  // Add this
        }
    }
}
```

Add TagResponseDto for the tags endpoint:

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagResponseDto {
    pub name: String,
    pub count: i64,
}
```

**Fix existing code that creates DTOs:**

You'll need to add `tags: vec![]` or `tags: None` to existing code that constructs these DTOs. The compiler will tell you where. Common places:

- `src/handlers/web.rs` - form submissions
- `src/handlers/test_service.rs` - test handlers
- Test files

**Verify:**

```bash
cargo check
```

### Step 4: Create Tag Repository

Create a repository layer for tag-specific database operations.

**File: `src/repository/tag_repository.rs`**

```rust
use crate::entities::{memo_tags, prelude::*, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub struct TagRepository;

impl TagRepository {
    /// Get or create a tag by name
    ///
    /// If the tag exists, returns it. Otherwise, creates a new tag.
    #[tracing::instrument(skip(db))]
    pub async fn get_or_create(
        db: &DatabaseConnection,
        name: String,
    ) -> Result<tags::Model, DbErr> {
        // Try to find existing tag
        if let Some(existing_tag) = Tags::find()
            .filter(tags::Column::Name.eq(&name))
            .one(db)
            .await?
        {
            tracing::debug!(tag_id = %existing_tag.id, tag_name = %name, "Found existing tag");
            return Ok(existing_tag);
        }

        // Create new tag
        let active_model = tags::ActiveModel {
            id: ActiveValue::NotSet,
            name: Set(name.clone()),
            created_at: ActiveValue::NotSet,
        };

        let tag = active_model.insert(db).await?;

        tracing::info!(tag_id = %tag.id, tag_name = %name, "Created new tag");

        Ok(tag)
    }

    /// Assign tags to a memo
    #[tracing::instrument(skip(db))]
    pub async fn assign_tags_to_memo(
        db: &DatabaseConnection,
        memo_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<(), DbErr> {
        for tag_id in tag_ids {
            let active_model = memo_tags::ActiveModel {
                memo_id: Set(memo_id),
                tag_id: Set(tag_id),
            };

            active_model.insert(db).await?;
        }

        tracing::debug!(memo_id = %memo_id, "Assigned tags to memo");

        Ok(())
    }

    /// Remove all tags from a memo
    #[tracing::instrument(skip(db))]
    pub async fn remove_all_tags_from_memo(
        db: &DatabaseConnection,
        memo_id: Uuid,
    ) -> Result<(), DbErr> {
        MemoTags::delete_many()
            .filter(memo_tags::Column::MemoId.eq(memo_id))
            .exec(db)
            .await?;

        tracing::debug!(memo_id = %memo_id, "Removed all tags from memo");

        Ok(())
    }

    /// Get all tags for a specific memo
    #[tracing::instrument(skip(db))]
    pub async fn get_tags_for_memo(
        db: &DatabaseConnection,
        memo_id: Uuid,
    ) -> Result<Vec<String>, DbErr> {
        let tag_names = MemoTags::find()
            .filter(memo_tags::Column::MemoId.eq(memo_id))
            .find_also_related(Tags)
            .all(db)
            .await?
            .into_iter()
            .filter_map(|(_, tag)| tag.map(|t| t.name))
            .collect();

        Ok(tag_names)
    }

    /// Get all tags with usage counts
    #[tracing::instrument(skip(db))]
    pub async fn get_all_tags_with_counts(
        db: &DatabaseConnection,
    ) -> Result<Vec<(String, i64)>, DbErr> {
        use sea_orm::{ConnectionTrait, Statement};

        let query = r#"
            SELECT t.name, COUNT(mt.memo_id) as count
            FROM tags t
            LEFT JOIN memo_tags mt ON t.id = mt.tag_id
            GROUP BY t.id, t.name
            ORDER BY count DESC, t.name ASC
        "#;

        let result = db
            .query_all(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                query,
            ))
            .await?;

        let tags: Vec<(String, i64)> = result
            .into_iter()
            .map(|row| {
                let name: String = row.try_get("", "name").unwrap_or_default();
                let count: i64 = row.try_get("", "count").unwrap_or(0);
                (name, count)
            })
            .collect();

        Ok(tags)
    }

    /// Delete unused tags (tags with no memos)
    #[tracing::instrument(skip(db))]
    pub async fn delete_unused_tags(db: &DatabaseConnection) -> Result<u64, DbErr> {
        use sea_orm::{ConnectionTrait, Statement};

        let query = r#"
            DELETE FROM tags
            WHERE id NOT IN (
                SELECT DISTINCT tag_id FROM memo_tags
            )
        "#;

        let result = db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                query,
            ))
            .await?;

        let deleted_count = result.rows_affected();

        tracing::info!(count = deleted_count, "Deleted unused tags");

        Ok(deleted_count)
    }
}
```

**Update `src/repository/mod.rs`:**

```rust
pub mod memo_repository;
pub mod tag_repository;  // Add

pub use memo_repository::MemoRepository;
pub use tag_repository::TagRepository;  // Add
```

**Verify:**

```bash
cargo check
```

## Checkpoint

At this point you should have:

✓ Database migration creating `tags` and `memo_tags` tables
✓ SeaORM entities for tags, memo_tags, and updated memos
✓ DTOs updated with tags fields
✓ TagRepository with all tag operations
✓ Code compiles with no errors

**Test the migration:**

```bash
# Check tables exist
psql $DATABASE_URL -c "\dt"

# Should see: memos, tags, memo_tags, seaql_migrations
```

## Summary

In this chapter, you've laid the foundation for a tagging system by:

### Database Layer
- Created a many-to-many relationship using a junction table
- Added proper indexes for query performance
- Implemented CASCADE deletes for data integrity

### Entity Layer
- Created SeaORM entities for tags and junction table
- Configured bidirectional relationships
- Updated memos entity with tag relations

### DTO Layer
- Added tags to all memo DTOs for API compatibility
- Implemented tag filtering in pagination
- Created helper methods for parsing tags

### Repository Layer
- Implemented get-or-create pattern for tags
- Built tag assignment and removal operations
- Created tag listing with usage counts
- Added cleanup for unused tags

## Next Steps

In the next sections (to be added to this chapter), you'll:

1. **Update Service Layer** - Integrate tag operations into memo service
2. **Update Handlers** - Add tag support to REST API endpoints
3. **Add Tags Endpoint** - Create `GET /api/v1/tags` for listing tags
4. **Update Web UI** - Add tag input and display to templates
5. **Add Tests** - Test tag creation, filtering, and cleanup
6. **Update Documentation** - Add tags to OpenAPI spec

The infrastructure is now in place. The remaining work is integrating tags into the service layer and user-facing interfaces.

---

**Note:** This is a work-in-progress chapter. Additional sections will be added to complete the implementation.
