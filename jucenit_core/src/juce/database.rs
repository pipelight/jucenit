// Database
use indexmap::IndexMap;
use sea_orm::entity::prelude::*;
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

pub async fn connect() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    Ok(db)
}

#[cfg(test)]
mod mock {
    use super::connect;
    use crate::{ConfigFile, Match};
    use entity::{prelude::*, *};
    use sea_orm::{prelude::*, ActiveValue, MockDatabase};
    // Error Handling
    use miette::{IntoDiagnostic, Result};

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
            .into_connection();
        Ok(db)
    }

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        connect().await?;
        Ok(())
    }

    #[tokio::test]
    async fn find_all_listeners() -> Result<()> {
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
        Ok(())
    }

    #[tokio::test]
    async fn find_one_listener() -> Result<()> {
        let db = prepare_mock_db().await?;

        let listener: Option<listener::Model> =
            Listener::find_by_id(1).one(&db).await.into_diagnostic()?;
        assert_eq!(
            listener,
            Some(listener::Model {
                id: 1,
                ip_socket: "*:443".to_owned(),
                tls: None
            })
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::connect;
    use crate::{ConfigFile, Match};
    use entity::{prelude::*, *};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
    // Logging
    use tracing::{debug, Level};
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn seed_db() -> Result<()> {
        let db = connect().await?;

        // Get struct from config
        let toml = "
            [[unit]]
            listeners = ['*:443']

            [unit.match]
            hosts = ['test.com']
            stuff = 'ee'

            [unit.action]
            proxy = 'http://127.0.0.1:8333'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        // println!("{:#?}", config);

        let unit = config.unit.first().unwrap();

        // Insert Match
        let raw_params: Option<String> = unit.match_.clone().raw_params.map(|x| x.to_string());
        let mut match_ = ng_match::ActiveModel {
            raw_params: ActiveValue::Set(raw_params.clone()),
            ..Default::default()
        };
        let res = NgMatch::insert(match_)
            .on_conflict(
                OnConflict::column(ng_match::Column::RawParams)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&db)
            .await
            .into_diagnostic();

        println!("{:#?}", res);

        // Return the existing entity
        match_ = match res {
            Ok(model) => model.into(),
            Err(e) => {
                // debug!("{}", e);
                println!("{}", e);
                let model = NgMatch::find()
                    .filter(ng_match::Column::RawParams.contains(raw_params.unwrap()))
                    .one(&db)
                    .await
                    .into_diagnostic()?
                    .unwrap()
                    .into();
                println!("{:#?}", model);
                model
            }
        };

        // let res = NgMatch::(res.last_insert_id)
        //     .one(&db)
        //     .await
        //     .into_diagnostic()?
        //     .unwrap()
        //     .into();
        // println!("{:#?}", res);

        // Insert listeners
        let mut listeners: Vec<listener::ActiveModel> = vec![];
        for l in &unit.listeners {
            let listener = listener::ActiveModel {
                ip_socket: ActiveValue::Set(l.to_owned()),
                ..Default::default()
            };
            listeners.push(listener);
        }
        Listener::insert_many(listeners.clone())
            .on_conflict(
                OnConflict::column(listener::Column::IpSocket)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&db)
            .await
            .into_diagnostic()?;

        // Insert Hosts
        let mut hosts: Vec<host::ActiveModel> = vec![];
        if let Some(dns_list) = &unit.match_.hosts {
            for host in dns_list {
                let host = host::ActiveModel {
                    domain: ActiveValue::Set(host.to_owned()),
                    ..Default::default()
                };
                hosts.push(host);
            }
        }
        Host::insert_many(hosts)
            .on_conflict(
                OnConflict::column(host::Column::Domain)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&db)
            .await
            .into_diagnostic()?;
        // Join Match and Host
        // let mut match_host_list: Vec<host::ActiveModel> = vec![];
        // for host in hosts {
        //     let match_host = match_host::ActiveModel {
        //         host_id: host.id,
        //         match_id: match_.id,
        //     };
        // }

        // Insert Action
        if let Some(action) = &unit.action.clone() {
            let raw_params: Option<String> = action.clone().raw_params.map(|x| x.to_string());
            let action = action::ActiveModel {
                raw_params: ActiveValue::Set(raw_params),
                ..Default::default()
            };
            Action::insert(action)
                .on_conflict(
                    OnConflict::column(action::Column::RawParams)
                        .do_nothing()
                        .to_owned(),
                )
                .do_nothing()
                .exec(&db)
                .await
                .into_diagnostic()?;
        }

        Ok(())
    }
}
