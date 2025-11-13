use crate::entities::{memo_tags, prelude::*, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    FromQueryResult, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    Set, sea_query::Expr,
};
use uuid::Uuid;

/// Helper struct for tag count query results
#[derive(Debug, FromQueryResult)]
struct TagCount {
    name: String,
    count: i64,
}

pub struct TagRepository;

impl TagRepository {
    /// Get or create a tag by name
    ///
    /// If the tag exists, returns it. Otherwise, creates a new tag.
    #[tracing::instrument(skip(db))]
    pub async fn get_or_create(
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
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
        db: &impl ConnectionTrait,
    ) -> Result<Vec<(String, i64)>, DbErr> {
        let results: Vec<TagCount> = Tags::find()
            .select_only()
            .column(tags::Column::Name)
            .column_as(
                Expr::col((MemoTags, memo_tags::Column::MemoId)).count(),
                "count",
            )
            .join(JoinType::LeftJoin, tags::Relation::MemoTags.def())
            .group_by(tags::Column::Id)
            .group_by(tags::Column::Name)
            .order_by_desc(Expr::col((MemoTags, memo_tags::Column::MemoId)).count())
            .order_by_asc(tags::Column::Name)
            .into_model::<TagCount>()
            .all(db)
            .await?;

        let tags: Vec<(String, i64)> = results.into_iter().map(|tc| (tc.name, tc.count)).collect();

        Ok(tags)
    }

    /// Get tag IDs for a specific memo
    ///
    /// Returns tag IDs as a vector of UUIDs.
    pub async fn get_tag_ids_for_memo(
        db: &impl ConnectionTrait,
        memo_id: Uuid,
    ) -> Result<Vec<Uuid>, DbErr> {
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
    pub async fn count_memos_with_tag(
        db: &impl ConnectionTrait,
        tag_id: Uuid,
    ) -> Result<u64, DbErr> {
        let count = MemoTags::find()
            .filter(memo_tags::Column::TagId.eq(tag_id))
            .count(db)
            .await?;

        Ok(count)
    }

    /// Delete a specific tag by ID
    ///
    /// Deletes the tag from the database.
    pub async fn delete_tag(db: &impl ConnectionTrait, tag_id: Uuid) -> Result<(), DbErr> {
        tags::Entity::delete_by_id(tag_id).exec(db).await?;

        tracing::debug!(tag_id = %tag_id, "Deleted tag");

        Ok(())
    }
}
