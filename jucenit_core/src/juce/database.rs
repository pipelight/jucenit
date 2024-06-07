use crate::mapping::{ListenerOpts, Route};

// Database
use indexmap::IndexMap;
use sea_orm::entity::prelude::*;
use sea_orm::{Database, DatabaseConnection};

// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

pub async fn connect() -> Result<()> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    Ok(())
}
// #[derive(Debug, Clone, Default, Eq, PartialEq, Hash, DeriveEntityModel)]
// #[sea_orm(table_name = "match")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     pub id: i32,
//     pub hosts: Vec<Host>,
//     pub uri: Option<String>,
//     pub source: Option<Vec<String>>,
// }
//
// #[derive(Debug, Clone, Default, Eq, PartialEq, Hash, DeriveEntity)]
// pub struct Host {
//     pub dns: Option<String>,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;
    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        connect().await?;
        Ok(())
    }
}
