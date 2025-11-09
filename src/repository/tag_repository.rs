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
    ///
    /// Creates tag associations in the memo_tags junction table.
    /// Tags should already exist (call get_or_create first).
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
    ///
    /// Deletes all entries in memo_tags for the given memo_id.
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
    ///
    /// Returns tag names as a vector of strings.
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
    ///
    /// Returns a list of (tag_name, count) tuples sorted by count descending.
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
    ///
    /// Returns the number of tags deleted.
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
