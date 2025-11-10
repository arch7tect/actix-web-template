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

### Service Layer
- Integrated tag operations into all memo methods
- Auto-create tags when creating/updating memos
- Auto-delete unused tags after operations
- Load tags for all responses

## Step 5: Integrate Tags in Service Layer

Now that we have the repository layer, let's integrate tags into the memo service.

**Update `src/services/memo_service.rs` imports:**

```rust
use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedResponse, PaginationParams, PatchMemoDto,
        UpdateMemoDto,
    },
    entities::memos,
    error::AppError,
    repository::{MemoRepository, TagRepository},  // Add TagRepository
    utils::{sanitize_html, sanitize_optional_html},
};
```

**Update `create_memo` method:**

```rust
#[tracing::instrument(skip(self, dto), fields(has_description = dto.description.is_some(), tag_count = dto.tags.len()))]
pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoResponseDto, AppError> {
    dto.validate()?;

    let sanitized_title = sanitize_html(&dto.title);
    let sanitized_description = sanitize_optional_html(dto.description.as_deref());

    tracing::debug!(
        title = %sanitized_title,
        tag_count = dto.tags.len(),
        "Creating new memo with sanitized input"
    );

    let memo = MemoRepository::create(
        &self.db,
        sanitized_title,
        sanitized_description,
        dto.date_to,
    )
    .await?;

    // Handle tags if provided
    if !dto.tags.is_empty() {
        let mut tag_ids = Vec::new();
        for tag_name in &dto.tags {
            let tag = TagRepository::get_or_create(&self.db, tag_name.trim().to_string()).await?;
            tag_ids.push(tag.id);
        }

        TagRepository::assign_tags_to_memo(&self.db, memo.id, tag_ids).await?;

        tracing::debug!(memo_id = %memo.id, tag_count = dto.tags.len(), "Tags assigned to memo");
    }

    tracing::info!(memo_id = %memo.id, "Memo created successfully with tags");

    // Load tags for response
    let tags = TagRepository::get_tags_for_memo(&self.db, memo.id).await?;

    Ok(Self::entity_to_dto_with_tags(memo, tags))
}
```

**Key changes:**
- Check if tags provided in DTO
- Loop through tag names, get or create each tag
- Collect tag IDs
- Assign all tags to the memo
- Load tags for response DTO

**Update `update_memo` method:**

```rust
pub async fn update_memo(
    &self,
    id: Uuid,
    dto: UpdateMemoDto,
) -> Result<MemoResponseDto, AppError> {
    dto.validate()?;

    let sanitized_title = sanitize_html(&dto.title);
    let sanitized_description = sanitize_optional_html(dto.description.as_deref());

    let memo = MemoRepository::update(
        &self.db,
        id,
        sanitized_title,
        sanitized_description,
        dto.date_to,
        dto.completed,
    )
    .await
    .map_err(|e| match e {
        sea_orm::DbErr::RecordNotFound(_) => {
            AppError::NotFound(format!("Memo with id {} not found", id))
        }
        _ => AppError::Database(e),
    })?;

    // Update tags: remove all existing and add new ones
    TagRepository::remove_all_tags_from_memo(&self.db, id).await?;

    if !dto.tags.is_empty() {
        let mut tag_ids = Vec::new();
        for tag_name in &dto.tags {
            let tag = TagRepository::get_or_create(&self.db, tag_name.trim().to_string()).await?;
            tag_ids.push(tag.id);
        }

        TagRepository::assign_tags_to_memo(&self.db, id, tag_ids).await?;
    }

    // Clean up unused tags
    TagRepository::delete_unused_tags(&self.db).await?;

    tracing::info!(memo_id = %memo.id, "Memo updated successfully with tags");

    // Load tags for response
    let tags = TagRepository::get_tags_for_memo(&self.db, id).await?;

    Ok(Self::entity_to_dto_with_tags(memo, tags))
}
```

**Tag update strategy:**
1. Remove all existing tag associations
2. Add new tag associations
3. Clean up any orphaned tags
4. Load tags for response

**Update `get_all_memos` to load tags:**

```rust
pub async fn get_all_memos(
    &self,
    params: PaginationParams,
) -> Result<PaginatedResponse<MemoResponseDto>, AppError> {
    params.validate()?;
    params.validate_order()?;

    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let order = params.order.as_deref().unwrap_or("desc");

    let tag_filter = params.parse_tags();  // Parse tags from query params

    let (memos, total) = MemoRepository::find_all(
        &self.db,
        limit,
        offset,
        params.completed,
        sort_by,
        order,
        tag_filter,  // Pass tag filter to repository
    )
    .await?;

    // Load tags for each memo
    let mut memo_dtos = Vec::new();
    for memo in memos {
        let tags = TagRepository::get_tags_for_memo(&self.db, memo.id).await?;
        memo_dtos.push(Self::entity_to_dto_with_tags(memo, tags));
    }

    tracing::info!(count = memo_dtos.len(), total, "Successfully fetched memos with tags");

    Ok(PaginatedResponse::new(memo_dtos, total, limit, offset))
}
```

**Add helper method:**

```rust
fn entity_to_dto_with_tags(entity: memos::Model, tags: Vec<String>) -> MemoResponseDto {
    MemoResponseDto {
        id: entity.id,
        title: entity.title,
        description: entity.description,
        date_to: entity.date_to.into(),
        completed: entity.completed,
        created_at: entity.created_at.into(),
        updated_at: entity.updated_at.into(),
        tags,  // Include tags in response
    }
}
```

**Update `delete_memo` to cleanup tags:**

```rust
pub async fn delete_memo(&self, id: Uuid) -> Result<(), AppError> {
    let deleted = MemoRepository::delete(&self.db, id).await?;

    if !deleted {
        return Err(AppError::NotFound(format!("Memo with id {} not found", id)));
    }

    // Clean up unused tags after deletion (CASCADE already removed memo_tags entries)
    TagRepository::delete_unused_tags(&self.db).await?;

    tracing::info!("Memo deleted successfully, cleaned up unused tags");

    Ok(())
}
```

## Step 6: Create Tags Listing Endpoint

Create a handler to list all tags with their usage counts.

**Create `src/handlers/tags.rs`:**

```rust
use crate::{dto::TagResponseDto, error::AppError, repository::TagRepository, state::AppState};
use actix_web::{get, web, HttpResponse, Result};

/// List all tags with usage counts
#[utoipa::path(
    get,
    path = "/api/v1/tags",
    tag = "Tags",
    responses(
        (status = 200, description = "List of all tags with usage counts", body = Vec<TagResponseDto>),
        (status = 500, description = "Internal server error")
    )
)]
#[get("/api/v1/tags")]
#[tracing::instrument(name = "GET /api/v1/tags", skip(state))]
pub async fn list_tags(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    tracing::debug!("Fetching all tags");

    let tags_with_counts = TagRepository::get_all_tags_with_counts(&state.db).await?;

    let response: Vec<TagResponseDto> = tags_with_counts
        .into_iter()
        .map(|(name, count)| TagResponseDto { name, count })
        .collect();

    tracing::info!(tag_count = response.len(), "Successfully fetched tags");

    Ok(HttpResponse::Ok().json(response))
}
```

**Register the handler in `src/handlers/mod.rs`:**

```rust
pub mod tags;  // Add this

pub use tags::list_tags;  // Export it
```

**Add route in `src/main.rs`:**

```rust
.service(handlers::list_tags)  // Add after other memo routes
```

**Export TagResponseDto from `src/dto/mod.rs`:**

```rust
pub use memo_dto::{
    CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PaginatedResponse, PaginationParams,
    PatchMemoDto, TagResponseDto, UpdateMemoDto,  // Add TagResponseDto
};
```

## Step 7: Update OpenAPI Documentation

Update the API documentation to include tags.

**Update `src/docs/openapi.rs`:**

```rust
use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedMemoResponse, PatchMemoDto, TagResponseDto,  // Add
        UpdateMemoDto,
    },
    error::ErrorResponse,
    handlers::{health, memos, tags},  // Add tags
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Memos API",
        version = "0.2.0",  // Increment version
        description = "A RESTful API for managing memos with tags, full CRUD operations, pagination, filtering, and sorting.",
    ),
    paths(
        memos::list_memos,
        memos::get_memo,
        memos::create_memo,
        memos::update_memo,
        memos::patch_memo,
        memos::delete_memo,
        memos::toggle_complete,
        tags::list_tags,  // Add tags endpoint
        health::health,
        health::ready,
    ),
    components(
        schemas(
            MemoResponseDto,
            CreateMemoDto,
            UpdateMemoDto,
            PatchMemoDto,
            PaginatedMemoResponse,
            TagResponseDto,  // Add tag schema
            ErrorResponse,
            health::HealthResponse,
            health::ReadyResponse,
        )
    ),
    tags(
        (name = "memos", description = "Memo management endpoints"),
        (name = "Tags", description = "Tag management and listing endpoints"),  // Add
        (name = "Observability", description = "Health checks and monitoring endpoints.")
    )
)]
pub struct ApiDoc;
```

## Step 8: Test the Tags Feature

Let's verify the complete tags implementation works.

**Build and run:**

```bash
cargo build
cargo run
```

**Test creating a memo with tags:**

```bash
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Team Meeting",
    "description": "Discuss Q1 goals",
    "date_to": "2025-12-31T10:00:00Z",
    "tags": ["work", "urgent", "meeting"]
  }'
```

**Expected response:**

```json
{
  "id": "uuid-here",
  "title": "Team Meeting",
  "description": "Discuss Q1 goals",
  "date_to": "2025-12-31T10:00:00Z",
  "completed": false,
  "created_at": "2025-01-10T...",
  "updated_at": "2025-01-10T...",
  "tags": ["work", "urgent", "meeting"]
}
```

**Test listing all tags:**

```bash
curl http://localhost:3737/api/v1/tags
```

**Expected response:**

```json
[
  {"name": "work", "count": 5},
  {"name": "urgent", "count": 3},
  {"name": "meeting", "count": 2},
  {"name": "personal", "count": 1}
]
```

**Test filtering memos by tags:**

```bash
# Get all memos tagged with "work" OR "urgent"
curl "http://localhost:3737/api/v1/memos?tags=work,urgent"

# Should return only memos that have at least one of these tags
```

**Test tag auto-creation:**

```bash
# Create memo with new tags
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Grocery Shopping",
    "date_to": "2025-12-31T10:00:00Z",
    "tags": ["personal", "shopping"]
  }'

# List tags - should now include "personal" and "shopping"
curl http://localhost:3737/api/v1/tags
```

**Test tag auto-deletion:**

```bash
# Create a memo with unique tag
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Temporary Task",
    "date_to": "2025-12-31T10:00:00Z",
    "tags": ["temporary"]
  }' | jq -r '.id'

# Note the memo ID from response

# Delete the memo
curl -X DELETE http://localhost:3737/api/v1/memos/<memo-id>

# List tags - "temporary" should be gone
curl http://localhost:3737/api/v1/tags
```

**Test updating tags:**

```bash
# Update memo, changing its tags
curl -X PUT http://localhost:3737/api/v1/memos/<memo-id> \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Title",
    "date_to": "2025-12-31T10:00:00Z",
    "completed": false,
    "tags": ["updated", "different"]
  }'

# Old tags (if unused) should be deleted
```

## Checkpoint

Verify your complete tags implementation:

```bash
# 1. All code compiles
cargo check

# 2. Create a memo with tags
MEMO_ID=$(curl -s -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test Memo",
    "date_to": "2025-12-31T10:00:00Z",
    "tags": ["test", "demo"]
  }' | jq -r '.id')

# 3. Verify tags in response
curl -s http://localhost:3737/api/v1/memos/$MEMO_ID | jq '.tags'
# Expected: ["test", "demo"]

# 4. List all tags
curl -s http://localhost:3737/api/v1/tags | jq
# Expected: Array with tag objects

# 5. Filter by tag
curl -s "http://localhost:3737/api/v1/memos?tags=test" | jq '.data | length'
# Expected: At least 1

# 6. Check Swagger UI
open http://localhost:3737/swagger-ui/
# Look for "Tags" section and GET /api/v1/tags endpoint
```

## Common Issues and Solutions

### Issue: Tags not appearing in response

**Cause:** Service layer not loading tags

**Solution:**
```rust
// Make sure you're using entity_to_dto_with_tags, not entity_to_dto
let tags = TagRepository::get_tags_for_memo(&self.db, memo.id).await?;
Ok(Self::entity_to_dto_with_tags(memo, tags))
```

### Issue: Duplicate tags created

**Cause:** Not using get_or_create

**Solution:**
```rust
// Use get_or_create which checks for existing tags first
let tag = TagRepository::get_or_create(&self.db, tag_name.trim().to_string()).await?;
```

### Issue: Old tags not cleaned up

**Cause:** Missing cleanup call after updates/deletes

**Solution:**
```rust
// Call after removing tag associations
TagRepository::delete_unused_tags(&self.db).await?;
```

### Issue: Tag filtering returns empty results

**Cause:** Case sensitivity or whitespace in tag names

**Solution:**
```rust
// Trim whitespace when creating/searching tags
let tag_name = tag_name.trim().to_lowercase();  // Optional: normalize case
```

## Summary

Congratulations! You've successfully implemented a complete tagging system. Here's what you accomplished:

### Features Implemented

✅ **Many-to-Many Relationships**
- Tags table with unique constraint
- Junction table (memo_tags) with composite primary key
- CASCADE delete behavior for data integrity

✅ **Tag Management**
- Auto-create tags on-the-fly when creating/updating memos
- Auto-delete unused tags after operations
- Get-or-create pattern prevents duplicates

✅ **Tag Filtering**
- OR logic: return memos with ANY of the specified tags
- Query parameter: `?tags=work,urgent`
- Efficient multi-step queries with indexes

✅ **API Endpoints**
- `POST /api/v1/memos` with tags array
- `PUT/PATCH /api/v1/memos/{id}` to update tags
- `GET /api/v1/memos?tags=...` to filter
- `GET /api/v1/tags` to list all tags with counts

✅ **Service Integration**
- Tags loaded for all memo responses
- Tags managed in all CRUD operations
- Automatic cleanup lifecycle

✅ **Documentation**
- OpenAPI/Swagger UI updated
- TagResponseDto schema
- All endpoints documented

### Key Concepts Learned

**Database Design:**
- When to use junction tables
- Composite primary keys
- Foreign key constraints and CASCADE behavior
- Index strategy for many-to-many queries

**SeaORM Patterns:**
- Related trait for bidirectional relationships
- Many-to-many via() implementation
- Custom relationship queries

**Service Layer Patterns:**
- Get-or-create for avoiding duplicates
- Lifecycle management (auto-create, auto-delete)
- Loading related data efficiently

**API Design:**
- Backward-compatible evolution
- Filtering with query parameters
- OR vs AND filtering logic

## Next Steps

You now have a fully functional tagging system! Consider these enhancements:

### Optional Improvements

1. **AND Filtering** - Add `?tags_match=all` for memos with ALL tags
2. **Tag Limits** - Prevent users from creating too many tags per memo
3. **Tag Suggestions** - Autocomplete based on existing tags
4. **Tag Colors** - Add color field to tags table for visual distinction
5. **Popular Tags** - Show most-used tags in UI
6. **Tag Analytics** - Track tag usage over time

### Web UI (Future Chapter)

To add tag support to the web interface:
- Add tag input field to memo forms (comma-separated or chip-based)
- Display tags as clickable badges on memo cards
- Click tag to filter memos by that tag
- Show tag cloud or tag list in sidebar

### Testing (Exercise for Reader)

Write tests for:
- Creating memos with tags
- Updating tags
- Filtering by tags
- Tag auto-deletion
- Concurrent tag operations

---

**You've completed Chapter 18!** You now have a production-ready tagging system with proper database design, efficient queries, and a clean API.

The tags feature demonstrates advanced Rust and web development concepts: many-to-many relationships, lifecycle management, and API evolution. These patterns apply to many other features you might build.

**Next:** Chapter 19 will cover more advanced topics like full-text search, real-time updates with WebSockets, or other features you'd like to add to your application.
