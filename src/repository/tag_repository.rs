use crate::entities::{memo_tags, prelude::*, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct TagRepository;

impl TagRepository {
    /// Get or create a tag by name
    ///
    /// If the tag exists, returns it. Otherwise, creates a new tag.
    #[tracing::instrument(skip(db))]
    pub async fn get_or_create<C>(db: &C, name: String) -> Result<tags::Model, DbErr>
    where
        C: ConnectionTrait,
    {
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
    pub async fn assign_tags_to_memo<C>(
        db: &C,
        memo_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
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
    pub async fn remove_all_tags_from_memo<C>(db: &C, memo_id: Uuid) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
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
    pub async fn get_tags_for_memo<C>(db: &C, memo_id: Uuid) -> Result<Vec<String>, DbErr>
    where
        C: ConnectionTrait,
    {
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
    pub async fn get_all_tags_with_counts<C>(db: &C) -> Result<Vec<(String, i64)>, DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_orm::Statement;

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

    /// Delete unused tags (tags with no memos) - DEPRECATED
    ///
    /// WARNING: This function has race conditions and should not be used in production.
    /// It deletes ALL unused tags globally, which can interfere with concurrent operations.
    ///
    /// Instead, use scoped cleanup: get tag IDs for a specific memo, check if each is unused,
    /// and delete only those tags within a transaction.
    ///
    /// This function is kept for backwards compatibility but will be removed in a future version.
    #[deprecated(
        since = "0.2.6",
        note = "Use scoped tag cleanup instead to avoid race conditions"
    )]
    #[tracing::instrument(skip(db))]
    pub async fn delete_unused_tags<C>(db: &C) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_orm::Statement;

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

        tracing::warn!(count = deleted_count, "DEPRECATED: delete_unused_tags() called - use scoped cleanup instead");

        Ok(deleted_count)
    }

    /// Get tag IDs for a specific memo
    ///
    /// Returns tag IDs as a vector of UUIDs.
    pub async fn get_tag_ids_for_memo<C>(db: &C, memo_id: Uuid) -> Result<Vec<Uuid>, DbErr>
    where
        C: ConnectionTrait,
    {
        let tag_ids = MemoTags::find()
            .filter(memo_tags::Column::MemoId.eq(memo_id))
            .all(db)
            .await?
            .into_iter()
            .map(|mt| mt.tag_id)
            .collect();

        Ok(tag_ids)
    }

    /// Count how many memos are using a specific tag
    ///
    /// Returns the count of memos associated with this tag.
    pub async fn count_memos_with_tag<C>(db: &C, tag_id: Uuid) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        let count = MemoTags::find()
            .filter(memo_tags::Column::TagId.eq(tag_id))
            .count(db)
            .await?;

        Ok(count)
    }

    /// Delete a specific tag by ID
    ///
    /// Deletes the tag from the database.
    pub async fn delete_tag<C>(db: &C, tag_id: Uuid) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        tags::Entity::delete_by_id(tag_id).exec(db).await?;

        tracing::debug!(tag_id = %tag_id, "Deleted tag");

        Ok(())
    }
}
