//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "match_action")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub match_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub action_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::action::Entity",
        from = "Column::ActionId",
        to = "super::action::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Action,
    #[sea_orm(
        belongs_to = "super::ng_match::Entity",
        from = "Column::MatchId",
        to = "super::ng_match::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    NgMatch,
}

impl Related<super::action::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Action.def()
    }
}

impl Related<super::ng_match::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NgMatch.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}