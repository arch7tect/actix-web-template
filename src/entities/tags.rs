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
