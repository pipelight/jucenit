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
mod mock {
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
    use miette::{IntoDiagnostic, Result};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, MockDatabase};

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

        println!("{:#?}", config);
        let unit = config.unit.first().unwrap();

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

        // Insert Match
        let raw_params: Option<String> = unit.match_.clone().raw_params.map(|x| x.to_string());
        let match_ = ng_match::ActiveModel {
            raw_params: ActiveValue::Set(raw_params),
            ..Default::default()
        };
        NgMatch::insert(match_).exec(&db).await.into_diagnostic()?;

        // Insert Hosts
        // for h in &unit.match_.hosts {
        //     listeners.push(listener);
        // }

        // Insert Match

        // let mut match_params = unit.match_.clone();
        // match_params.hosts = None;
        // NgMatch::insert(ng_match::ActiveModel {
        //     raw_params: ActiveValue::Set(Some(
        //         serde_json::to_value(match_params)
        //             .into_diagnostic()?
        //             .to_string(),
        //     )),
        //     ..Default::default()
        // })
        // .exec(&db)
        // .await
        // .into_diagnostic()?;
        //
        // // Insert action and join
        // Action::insert(action::ActiveModel {
        //     raw_params: ActiveValue::Set(Some(
        //         serde_json::to_value(unit.action.clone())
        //             .into_diagnostic()?
        //             .to_string(),
        //     )),
        //
        //     ..Default::default()
        // });

        Ok(())
    }
}
