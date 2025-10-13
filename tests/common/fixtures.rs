use actix_web_template::dto::CreateMemoDto;
use chrono::Utc;

pub fn create_test_memo_dto(title: &str, description: Option<&str>) -> CreateMemoDto {
    CreateMemoDto {
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        date_to: Utc::now(),
    }
}
