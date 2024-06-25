//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "match_listener")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub match_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub listener_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::listener::Entity",
        from = "Column::ListenerId",
        to = "super::listener::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    Listener,
    #[sea_orm(
        belongs_to = "super::ng_match::Entity",
        from = "Column::MatchId",
        to = "super::ng_match::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    NgMatch,
}

impl Related<super::listener::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listener.def()
    }
}

impl Related<super::ng_match::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NgMatch.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
