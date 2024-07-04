use crate::cast::Config as ConfigFile;
use crate::{Action, Match};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
// File
use crate::nginx::Config as NginxConfig;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Sea orm
// use indexmap::IndexMap;
use super::entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    error::{ConnAcquireErr, DbErr},
    Database, DatabaseConnection,
};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};

// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

// Global vars
// use once_cell::sync::Lazy;
// use std::sync::Arc;
// use tokio::sync::Mutex;

pub async fn connect_db() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rw";
    // let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    let db = Database::connect(database_url).await;
    match &db {
        Err(e) => {
            let db = fresh_db().await?;
            return Ok(db);
        }
        _ => {}
    };
    Ok(db.into_diagnostic()?)
}
pub async fn fresh_db() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db = sea_orm::Database::connect(database_url)
        .await
        .into_diagnostic()?;
    Migrator::fresh(&db).await.into_diagnostic()?;
    Ok(db)
}
#[cfg(test)]
mod test {
    use super::*;
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        // connect_db().await?;
        fresh_db().await?;
        Ok(())
    }
}
