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
    use super::connect;
    use crate::{ConfigFile, Match};
    use entity::{prelude::*, *};
    use miette::{IntoDiagnostic, Result};
    use sea_orm::{prelude::*, ActiveValue, MockDatabase};

    async fn prepare_mock_db() -> Result<DatabaseConnection> {
        let db: DatabaseConnection = MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
            // Add listeners
            .append_query_results([vec![
                listener::Model {
                    id: 1,
                    ip_socket: "*:443".to_owned(),
                    tls: None,
                },
                listener::Model {
                    id: 2,
                    ip_socket: "*:587".to_owned(),
                    tls: None,
                },
                listener::Model {
                    id: 3,
                    ip_socket: "*:993".to_owned(),
                    tls: None,
                },
            ]])
            // Add actions
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
            // Add nginx match conditions/parameters
            .append_query_results([vec![
                ng_match::Model {
                    id: 1,
                    action_id: Some(1),
                    raw_params: None,
                },
                ng_match::Model {
                    id: 2,
                    action_id: Some(2),
                    raw_params: None,
                },
            ]])
            // Add host names
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
            // Link host to match
            .append_query_results([vec![
                match_host::Model {
                    id: 1,
                    host_id: Some(1),
                    match_id: Some(1),
                },
                match_host::Model {
                    id: 2,
                    host_id: Some(2),
                    match_id: Some(2),
                },
            ]])
            // Link listener to match
            .append_query_results([vec![
                match_listener::Model {
                    id: 1,
                    match_id: Some(1),
                    listener_id: Some(1),
                },
                match_listener::Model {
                    id: 2,
                    match_id: Some(2),
                    listener_id: Some(2),
                },
                match_listener::Model {
                    id: 3,
                    match_id: Some(1),
                    listener_id: Some(3),
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
        let db = prepare_mock_db().await?;

        // Test db querying and joint
        let all_listeners: Vec<listener::Model> =
            Listener::find().all(&db).await.into_diagnostic()?;
        assert_eq!(
            all_listeners,
            vec![
                listener::Model {
                    id: 1,
                    ip_socket: "*:443".to_owned(),
                    tls: None
                },
                listener::Model {
                    id: 2,
                    ip_socket: "*:587".to_owned(),
                    tls: None
                },
                listener::Model {
                    id: 3,
                    ip_socket: "*:993".to_owned(),
                    tls: None
                },
            ]
        );
        let a_listener: Option<listener::Model> =
            Listener::find_by_id(1).one(&db).await.into_diagnostic()?;
        let a_listener: listener::Model = a_listener.unwrap();

        let matches: Vec<(listener::Model, Vec<ng_match::Model>)> = Listener::find()
            .find_with_related(NgMatch)
            .all(&db)
            .await
            .into_diagnostic()?;

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

    #[tokio::test]
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

        let db = connect().await?;

        let mut listeners = vec![];

        for l in &unit.listeners {
            let listener = listener::ActiveModel {
                ip_socket: ActiveValue::Set(l.to_owned()),
                ..Default::default()
            };
            listeners.push(listener);
        }

        // Host::insert()

        let mut match_params = unit.match_.clone();
        match_params.hosts = None;
        NgMatch::insert(ng_match::ActiveModel {
            raw_params: ActiveValue::Set(Some(
                serde_json::to_value(match_params)
                    .into_diagnostic()?
                    .to_string(),
            )),
            ..Default::default()
        })
        .exec(&db)
        .await
        .into_diagnostic()?;

        // Insert action and join
        Action::insert(action::ActiveModel {
            raw_params: ActiveValue::Set(Some(
                serde_json::to_value(unit.action.clone())
                    .into_diagnostic()?
                    .to_string(),
            )),

            ..Default::default()
        });

        Listener::insert_many(listeners)
            .exec(&db)
            .await
            .into_diagnostic()?;

        Ok(())
    }
}
