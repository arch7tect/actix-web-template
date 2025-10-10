use crate::{
    dto::{
        CreateMemoDto, MemoResponseDto, PaginatedResponse, PaginationParams, PatchMemoDto,
        UpdateMemoDto,
    },
    entities::memos,
    error::AppError,
    repository::MemoRepository,
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

        tracing::debug!(title = %dto.title, "Creating new memo");

        let memo =
            MemoRepository::create(&self.db, dto.title, dto.description, dto.date_to).await?;

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

        tracing::debug!("Updating memo");

        let memo = MemoRepository::update(
            &self.db,
            id,
            dto.title,
            dto.description,
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

        let title = dto.title.unwrap_or(existing_memo.title);
        let description = dto.description.or(existing_memo.description);
        let date_to = dto.date_to.unwrap_or_else(|| existing_memo.date_to.into());
        let completed = dto.completed.unwrap_or(existing_memo.completed);

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
