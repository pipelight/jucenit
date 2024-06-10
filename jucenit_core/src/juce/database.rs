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
    use entity::*;
    use miette::Result;
    use sea_orm::{prelude::*, MockDatabase};

    async fn prepare_mock_db() -> Result<DatabaseConnection> {
        let db: DatabaseConnection = MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
            .append_query_results([vec![
                listener::Model {
                    id: 1,
                    ip_socket: Some("*:443".to_owned()),
                },
                listener::Model {
                    id: 2,
                    ip_socket: Some("*:587".to_owned()),
                },
                listener::Model {
                    id: 3,
                    ip_socket: Some("*:993".to_owned()),
                },
            ]])
            .append_query_results([vec![
                action::Model {
                    id: 1,
                    raw_params: Some(
                        "{
                            \"proxy\" = \"http://127.0.0.1:9080\",
                        }"
                        .to_owned(),
                    ),
                },
                action::Model {
                    id: 2,
                    raw_params: Some(
                        "{
                            \"proxy\" = \"http://127.0.0.1:8333\",
                        }"
                        .to_owned(),
                    ),
                },
            ]])
            .append_query_results([vec![ng_match::Model {
                id: 1,
                action_id: Some(1),
                raw_params: None,
            }]])
            .append_query_results([vec![
                host::Model {
                    id: 1,
                    domain: "test.com".to_owned(),
                },
                host::Model {
                    id: 2,
                    domain: "example.com".to_owned(),
                },
            ]])
            .append_query_results([vec![
                match_host::Model {
                    id: 1,
                    host_id: Some(1),
                    match_id: Some(1),
                },
                match_host::Model {
                    id: 2,
                    host_id: Some(2),
                    match_id: Some(1),
                },
            ]])
            .into_connection();
        Ok(db)
    }

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        connect().await?;
        Ok(())
    }

    #[tokio::test]
    async fn query_mock_db() -> Result<()> {
        connect().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn insert_into_db() -> Result<()> {
        let db = connect().await?;
        let match_ = ng_match::ActiveModel {
            ..Default::default() // all other attributes are `NotSet`
        };
        let match_: ng_match::Model = match_.insert(&db).await.into_diagnostic()?;
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
