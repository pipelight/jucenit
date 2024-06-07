use crate::mapping::{ListenerOpts, Route};

// Database
use indexmap::IndexMap;
use sea_orm::entity::prelude::*;
use sea_orm::{Database, DatabaseConnection};

// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

pub async fn connect() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConfigFile;
    use entity;
    use miette::Result;
    use sea_orm::prelude::*;

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        connect().await?;
        Ok(())
    }

    #[tokio::test]
    async fn insert_into_db() -> Result<()> {
        let db = connect().await?;

        let match_ = entity::r#match::ActiveModel {
            ..Default::default() // all other attributes are `NotSet`
        };
        let match_: entity::r#match::Model = match_.insert(&db).await.into_diagnostic()?;
        Ok(())
    }

    // #[tokio::test]
    async fn seed_db() -> Result<()> {
        // Get struct from config
        let toml = "
            [[unit]]
            listeners = ['*:443']

            [unit.match]
            hosts = ['test.com']

            [unit.action]
            proxy = 'http://127.0.0.1:8333'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        let unit = config.unit.first().unwrap();

        let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
        let connection = sea_orm::Database::connect(database_url)
            .await
            .into_diagnostic()?;
        Ok(())
    }
}
