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
