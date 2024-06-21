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
use entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
use sea_orm::{Database, DatabaseConnection};

// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub async fn connect_db() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    Ok(db)
}
pub async fn fresh_db() -> Result<()> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let connection = sea_orm::Database::connect(database_url)
        .await
        .into_diagnostic()?;
    Migrator::fresh(&connection).await.into_diagnostic()?;
    Ok(())
}
