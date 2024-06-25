//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "ng_match")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub action_id: Option<i32>,
    #[sea_orm(unique)]
    pub raw_params: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::action::Entity",
        from = "Column::ActionId",
        to = "super::action::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Action,
    #[sea_orm(has_many = "super::match_host::Entity")]
    MatchHost,
    #[sea_orm(has_many = "super::match_listener::Entity")]
    MatchListener,
}

impl Related<super::action::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Action.def()
    }
}

impl Related<super::match_host::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchHost.def()
    }
}

impl Related<super::match_listener::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MatchListener.def()
    }
}

impl Related<super::host::Entity> for Entity {
    fn to() -> RelationDef {
        super::match_host::Relation::Host.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::match_host::Relation::NgMatch.def().rev())
    }
}

impl Related<super::listener::Entity> for Entity {
    fn to() -> RelationDef {
        super::match_listener::Relation::Listener.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::match_listener::Relation::NgMatch.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
