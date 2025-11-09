use crate::entities::{memo_tags, memos, prelude::*, tags};
use chrono::{DateTime, Utc};
use sea_orm::*;
use uuid::Uuid;

pub struct MemoRepository;

impl MemoRepository {
    #[tracing::instrument(skip(db), fields(limit, offset, completed, sort_by, order, tag_count = tag_names.as_ref().map(|t| t.len())))]
    pub async fn find_all(
        db: &DatabaseConnection,
        limit: u64,
        offset: u64,
        completed: Option<bool>,
        sort_by: &str,
        order: &str,
        tag_names: Option<Vec<String>>,
    ) -> Result<(Vec<memos::Model>, u64), DbErr> {
        tracing::debug!(
            limit,
            offset,
            completed,
            sort_by,
            order,
            tag_filter = ?tag_names,
            "Finding all memos with filters"
        );

        let mut query = Memos::find();

        if let Some(completed_filter) = completed {
            query = query.filter(memos::Column::Completed.eq(completed_filter));
        }

        // Filter by tags if provided (OR logic - memo has ANY of the tags)
        if let Some(ref tags) = tag_names {
            if !tags.is_empty() {
                // Find tag IDs for the given tag names
                let tag_models = Tags::find()
                    .filter(tags::Column::Name.is_in(tags.clone()))
                    .all(db)
                    .await?;

                let tag_ids: Vec<Uuid> = tag_models.iter().map(|t| t.id).collect();

                if !tag_ids.is_empty() {
                    // Find memo IDs that have any of these tags
                    let memo_ids: Vec<Uuid> = MemoTags::find()
                        .filter(memo_tags::Column::TagId.is_in(tag_ids))
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|mt| mt.memo_id)
                        .collect();

                    // Filter memos by these IDs
                    query = query.filter(memos::Column::Id.is_in(memo_ids));
                } else {
                    // No matching tags found, return empty result
                    return Ok((vec![], 0));
                }
            }
        }

        let sort_column = match sort_by {
            "title" => memos::Column::Title,
            "date_to" => memos::Column::DateTo,
            "completed" => memos::Column::Completed,
            "updated_at" => memos::Column::UpdatedAt,
            _ => memos::Column::CreatedAt,
        };

        query = if order == "asc" {
            query.order_by_asc(sort_column)
        } else {
            query.order_by_desc(sort_column)
        };

        let total = query.clone().count(db).await?;

        let memos = query.limit(limit).offset(offset).all(db).await?;

        tracing::info!(found = memos.len(), total, "Successfully retrieved memos with tag filter");

        Ok((memos, total))
    }

    #[tracing::instrument(skip(db), fields(memo_id = %id))]
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<memos::Model>, DbErr> {
        tracing::debug!("Finding memo by ID");

        let memo = Memos::find_by_id(id).one(db).await?;

        if memo.is_some() {
            tracing::info!("Memo found");
        } else {
            tracing::info!("Memo not found");
        }

        Ok(memo)
    }

    #[tracing::instrument(skip(db), fields(title, has_description = description.is_some()))]
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
    ) -> Result<memos::Model, DbErr> {
        tracing::debug!("Creating new memo");

        let now = Utc::now();
        let id = Uuid::new_v4();

        let new_memo = memos::ActiveModel {
            id: Set(id),
            title: Set(title),
            description: Set(description),
            date_to: Set(date_to.into()),
            completed: Set(false),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        let memo = new_memo.insert(db).await?;

        tracing::info!(memo_id = %memo.id, "Memo created successfully");

        Ok(memo)
    }

    #[tracing::instrument(skip(db), fields(memo_id = %id, has_description = description.is_some(), completed))]
    pub async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        title: String,
        description: Option<String>,
        date_to: DateTime<Utc>,
        completed: bool,
    ) -> Result<memos::Model, DbErr> {
        tracing::debug!("Updating memo");

        let memo = Memos::find_by_id(id).one(db).await?;

        if let Some(existing_memo) = memo {
            let mut active_memo: memos::ActiveModel = existing_memo.into();
            active_memo.title = Set(title);
            active_memo.description = Set(description);
            active_memo.date_to = Set(date_to.into());
            active_memo.completed = Set(completed);
            active_memo.updated_at = Set(Utc::now().into());

            let updated_memo = active_memo.update(db).await?;

            tracing::info!(memo_id = %updated_memo.id, "Memo updated successfully");

            Ok(updated_memo)
        } else {
            tracing::warn!("Memo not found for update");
            Err(DbErr::RecordNotFound(format!(
                "Memo with id {} not found",
                id
            )))
        }
    }

    #[tracing::instrument(skip(db), fields(memo_id = %id))]
    pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<bool, DbErr> {
        tracing::debug!("Deleting memo");

        let memo = Memos::find_by_id(id).one(db).await?;

        if let Some(existing_memo) = memo {
            let active_memo: memos::ActiveModel = existing_memo.into();
            active_memo.delete(db).await?;

            tracing::info!("Memo deleted successfully");

            Ok(true)
        } else {
            tracing::warn!("Memo not found for deletion");
            Ok(false)
        }
    }
}
