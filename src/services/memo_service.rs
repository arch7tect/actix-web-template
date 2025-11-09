use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedResponse, PaginationParams, PatchMemoDto,
        UpdateMemoDto,
    },
    entities::memos,
    error::AppError,
    repository::MemoRepository,
    utils::{sanitize_html, sanitize_optional_html},
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use validator::Validate;

pub struct MemoService {
    db: DatabaseConnection,
}

impl MemoService {
    pub fn new(db: DatabaseConnection) -> Self {
        tracing::debug!("Creating MemoService");
        Self { db }
    }

    #[tracing::instrument(skip(self), fields(limit, offset, completed))]
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

        tracing::debug!(
            limit,
            offset,
            completed = ?params.completed,
            sort_by,
            order,
            "Fetching all memos"
        );

        let (memos, total) =
            MemoRepository::find_all(&self.db, limit, offset, params.completed, sort_by, order)
                .await?;

        let memo_dtos: Vec<MemoResponseDto> = memos.into_iter().map(Self::entity_to_dto).collect();

        tracing::info!(count = memo_dtos.len(), total, "Successfully fetched memos");

        Ok(PaginatedResponse::new(memo_dtos, total, limit, offset))
    }

    #[tracing::instrument(skip(self), fields(memo_id = %id))]
    pub async fn get_memo_by_id(&self, id: Uuid) -> Result<MemoResponseDto, AppError> {
        tracing::debug!("Fetching memo by ID");

        let memo = MemoRepository::find_by_id(&self.db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Memo with id {} not found", id)))?;

        tracing::info!("Memo found successfully");

        Ok(Self::entity_to_dto(memo))
    }

    #[tracing::instrument(skip(self, dto), fields(has_description = dto.description.is_some()))]
    pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoResponseDto, AppError> {
        dto.validate()?;

        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        tracing::debug!(title = %sanitized_title, "Creating new memo with sanitized input");

        let memo = MemoRepository::create(
            &self.db,
            sanitized_title,
            sanitized_description,
            dto.date_to,
        )
        .await?;

        tracing::info!(memo_id = %memo.id, "Memo created successfully");

        Ok(Self::entity_to_dto(memo))
    }

    #[tracing::instrument(skip(self, dto), fields(memo_id = %id, has_description = dto.description.is_some()))]
    pub async fn update_memo(
        &self,
        id: Uuid,
        dto: UpdateMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        dto.validate()?;

        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        tracing::debug!("Updating memo with sanitized input");

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

        tracing::info!(memo_id = %memo.id, "Memo updated successfully");

        Ok(Self::entity_to_dto(memo))
    }

    #[tracing::instrument(skip(self, dto), fields(memo_id = %id))]
    pub async fn patch_memo(
        &self,
        id: Uuid,
        dto: PatchMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        dto.validate()?;

        tracing::debug!("Patching memo");

        let existing_memo = MemoRepository::find_by_id(&self.db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Memo with id {} not found", id)))?;

        let title = dto
            .title
            .map(|t| sanitize_html(&t))
            .unwrap_or(existing_memo.title);
        let description = match dto.description {
            Some(d) => sanitize_optional_html(Some(&d)),
            None => existing_memo.description,
        };
        let date_to = dto.date_to.unwrap_or_else(|| existing_memo.date_to.into());
        let completed = dto.completed.unwrap_or(existing_memo.completed);

        tracing::debug!("Patching memo with sanitized input");

        let memo =
            MemoRepository::update(&self.db, id, title, description, date_to, completed).await?;

        tracing::info!(memo_id = %memo.id, "Memo patched successfully");

        Ok(Self::entity_to_dto(memo))
    }

    #[tracing::instrument(skip(self), fields(memo_id = %id))]
    pub async fn delete_memo(&self, id: Uuid) -> Result<(), AppError> {
        tracing::debug!("Deleting memo");

        let deleted = MemoRepository::delete(&self.db, id).await?;

        if !deleted {
            tracing::warn!("Memo not found for deletion");
            return Err(AppError::NotFound(format!("Memo with id {} not found", id)));
        }

        tracing::info!("Memo deleted successfully");

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(memo_id = %id))]
    pub async fn toggle_complete(&self, id: Uuid) -> Result<MemoResponseDto, AppError> {
        tracing::debug!("Toggling memo completion status");

        let existing_memo = MemoRepository::find_by_id(&self.db, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Memo with id {} not found", id)))?;

        let new_completed = !existing_memo.completed;

        let memo = MemoRepository::update(
            &self.db,
            id,
            existing_memo.title,
            existing_memo.description,
            existing_memo.date_to.into(),
            new_completed,
        )
        .await?;

        tracing::info!(
            memo_id = %memo.id,
            completed = new_completed,
            "Memo completion status toggled"
        );

        Ok(Self::entity_to_dto(memo))
    }

    /// Creates multiple memos in a single transaction
    ///
    /// This demonstrates transaction coordination at the service layer.
    /// All memos are created atomically - if any fail, all are rolled back.
    ///
    /// # Arguments
    /// * `dtos` - Vector of memo creation DTOs
    ///
    /// # Returns
    /// Vector of created memos as DTOs
    ///
    /// # Example
    /// ```rust,no_run
    /// # use actix_web_template::services::MemoService;
    /// # use actix_web_template::dto::CreateMemoDto;
    /// # async fn example(service: MemoService, dto1: CreateMemoDto, dto2: CreateMemoDto, dto3: CreateMemoDto) -> Result<(), Box<dyn std::error::Error>> {
    /// let memos = vec![dto1, dto2, dto3];
    /// let created = service.create_memos_batch(memos).await?;
    /// // Either all 3 are created, or none are
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(skip(self, dtos), fields(count = dtos.len()))]
    pub async fn create_memos_batch(
        &self,
        dtos: Vec<CreateMemoDto>,
    ) -> Result<Vec<MemoResponseDto>, AppError> {
        use sea_orm::{ActiveModelTrait, ActiveValue, Set, TransactionTrait};

        tracing::debug!(count = dtos.len(), "Creating memos in batch transaction");

        // Validate all DTOs before starting transaction
        for dto in &dtos {
            dto.validate()?;
        }

        // Start a transaction
        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let mut created_memos = Vec::new();

        // Create all memos within the transaction
        for dto in dtos {
            let sanitized_title = sanitize_html(&dto.title);
            let sanitized_description = sanitize_optional_html(dto.description.as_deref());

            let active_model = memos::ActiveModel {
                id: ActiveValue::NotSet,
                title: Set(sanitized_title),
                description: Set(sanitized_description),
                date_to: Set(dto.date_to.into()),
                completed: Set(false),
                created_at: ActiveValue::NotSet,
                updated_at: ActiveValue::NotSet,
            };

            // Insert within transaction
            let memo = active_model
                .insert(&txn)
                .await
                .map_err(AppError::Database)?;

            created_memos.push(memo);
        }

        // Commit the transaction - all inserts succeed or all fail
        txn.commit().await.map_err(AppError::Database)?;

        tracing::info!(
            count = created_memos.len(),
            "Successfully created memos in batch"
        );

        Ok(created_memos.into_iter().map(Self::entity_to_dto).collect())
    }

    /// Deletes multiple memos in a single transaction
    ///
    /// All deletions happen atomically - if any fail, all are rolled back.
    ///
    /// # Arguments
    /// * `ids` - Vector of memo UUIDs to delete
    ///
    /// # Returns
    /// Number of memos deleted
    #[tracing::instrument(skip(self), fields(count = ids.len()))]
    pub async fn delete_memos_batch(&self, ids: Vec<Uuid>) -> Result<u64, AppError> {
        use crate::entities::prelude::Memos;
        use sea_orm::{EntityTrait, TransactionTrait};

        tracing::debug!(count = ids.len(), "Deleting memos in batch transaction");

        // Start transaction
        let txn = self.db.begin().await.map_err(AppError::Database)?;

        let mut total_deleted = 0u64;

        // Delete all memos in transaction
        for id in ids {
            let result = Memos::delete_by_id(id)
                .exec(&txn)
                .await
                .map_err(AppError::Database)?;

            total_deleted += result.rows_affected;
        }

        // Commit transaction
        txn.commit().await.map_err(AppError::Database)?;

        tracing::info!(count = total_deleted, "Successfully deleted memos in batch");

        Ok(total_deleted)
    }

    fn entity_to_dto(entity: memos::Model) -> MemoResponseDto {
        MemoResponseDto {
            id: entity.id,
            title: entity.title,
            description: entity.description,
            date_to: entity.date_to.into(),
            completed: entity.completed,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        }
    }
}
