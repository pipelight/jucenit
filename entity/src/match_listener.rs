//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "match_listener")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub match_id: Option<i32>,
    pub listener_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::r#match::Entity",
        from = "Column::MatchId",
        to = "super::r#match::Column::Id"
    )]
    Match,
    #[sea_orm(
        belongs_to = "super::listener::Entity",
        from = "Column::ListenerId",
        to = "super::listener::Column::Id"
    )]
    Listener,
}

impl ActiveModelBehavior for ActiveModel {}
