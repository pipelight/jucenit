//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "listener")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub ip_socket: Option<String>,
}

impl Related<super::r#match::Entity> for Entity {
    fn to() -> RelationDef {
        super::match_listener::Relation::Match.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::match_listener::Relation::Listener.def().rev())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
