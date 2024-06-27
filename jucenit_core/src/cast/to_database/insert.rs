// Database
use crate::database::{connect_db, fresh_db};
use crate::nginx::db_into_nginx_conf;
use crate::{ConfigFile, ConfigUnit};
// Sea orm
// use indexmap::IndexMap;
use entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase,
};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};
use watchexec::filter;

impl ConfigFile {
    /**
     * Push file to database
     */
    pub async fn push_to_db(&self) -> Result<()> {
        for unit in &self.unit {
            unit.push().await?;
        }
        Ok(())
    }
    /**
     * Clean up database and push file to database
     */
    async fn push_to_fresh_db(&self) -> Result<()> {
        fresh_db().await?;
        for unit in &self.unit {
            unit.push().await?;
        }
        Ok(())
    }
    /**
     * Push file to database
     * and update nginx
     */
    pub async fn push(&self) -> Result<()> {
        self.push_to_db().await?;
        let nginx_config = db_into_nginx_conf().await?;
        nginx_config.set().await?;
        Ok(())
    }
    /**
     * Clean up database and push file to database and
     */
    pub async fn set(&self) -> Result<()> {
        self.push_to_fresh_db().await?;
        let nginx_config = db_into_nginx_conf().await?;
        nginx_config.set().await?;
        Ok(())
    }
}
impl ConfigUnit {
    pub async fn push(&self) -> Result<()> {
        self.push_to_db().await?;
        let nginx_config = db_into_nginx_conf().await?;
        nginx_config.set().await?;
        Ok(())
    }
    pub async fn push_to_db(&self) -> Result<()> {
        let unit = self;
        // Logic Guards
        // Ignore gracefully if matching pattern lakes parameters
        if unit.match_.raw_params.is_none()
            || unit.match_.raw_params == Some(serde_json::from_str("{}").into_diagnostic()?)
                && unit.match_.hosts.is_none()
        {
            return Ok(());
        }
        // Ignore gracefully if no action
        if let Some(action) = &unit.action {
            if action.raw_params == Some(serde_json::from_str("{}").into_diagnostic()?) {
                return Ok(());
            }
        }

        let db = connect_db().await?;

        // Insert Action
        let raw_params: String = unit.action.clone().unwrap().raw_params.unwrap().to_string();

        let mut action = action::ActiveModel {
            raw_params: ActiveValue::Set(raw_params.clone()),
            ..Default::default()
        };

        let res = Action::insert(action)
            .on_conflict(
                OnConflict::column(action::Column::RawParams)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&db)
            .await
            .into_diagnostic();

        // Populate entities with ids
        action = match res {
            Ok(model) => model.into(),
            Err(e) => {
                // debug!("{}", e);
                // println!("{}", e);
                let model = Action::find()
                    .filter(action::Column::RawParams.eq(raw_params))
                    .one(&db)
                    .await
                    .into_diagnostic()?
                    .unwrap()
                    .into();
                model
            }
        };

        // Insert Match
        let raw_params: Option<String> = unit.match_.clone().raw_params.map(|x| x.to_string());
        let mut match_ = ng_match::ActiveModel {
            uuid: ActiveValue::Set(unit.uuid.clone()),
            action_id: action.id,
            raw_params: ActiveValue::Set(raw_params.clone()),
            ..Default::default()
        };
        let res = NgMatch::insert(match_)
            .on_conflict(
                OnConflict::column(ng_match::Column::Uuid)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_with_returning(&db)
            .await
            .into_diagnostic();

        // Return the existing entity
        match_ = match res {
            Ok(model) => model.into(),
            Err(e) => {
                // debug!("{}", e);
                // println!("{}", e);
                // println!("{:#?}", raw_params);
                let model = NgMatch::find()
                    .filter(ng_match::Column::Uuid.eq(raw_params.unwrap()))
                    .one(&db)
                    .await
                    .into_diagnostic()?
                    .unwrap()
                    .into();
                model
            }
        };

        // Insert listeners
        assert!(!&unit.listeners.is_empty());
        let mut listeners: Vec<listener::ActiveModel> = vec![];
        for l in &unit.listeners {
            let listener = listener::ActiveModel {
                ip_socket: ActiveValue::Set(l.to_owned()),
                ..Default::default()
            };
            listeners.push(listener);
        }
        let res = Listener::insert_many(listeners.clone())
            .on_conflict(
                OnConflict::column(listener::Column::IpSocket)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec_without_returning(&db)
            .await
            .into_diagnostic()?;

        // Populate entities with ids
        let models = Listener::find()
            .filter(listener::Column::IpSocket.is_in(&unit.listeners))
            .all(&db)
            .await
            .into_diagnostic()?;
        listeners = models
            .iter()
            .map(|x| listener::ActiveModel::from(x.to_owned()))
            .collect();

        // Join Match and Listener
        assert!(!&listeners.is_empty());
        let mut list: Vec<match_listener::ActiveModel> = vec![];
        for listener in listeners {
            let match_listener = match_listener::ActiveModel {
                match_id: match_.id.clone(),
                listener_id: listener.id,
            };
            list.push(match_listener)
        }
        let _ = MatchListener::insert_many(list)
            .on_conflict(
                OnConflict::columns(vec![
                    match_listener::Column::MatchId,
                    match_listener::Column::ListenerId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .do_nothing()
            .exec(&db)
            .await
            .into_diagnostic()?;

        // Insert Hosts
        if unit.match_.hosts.is_some() {
            let mut hosts: Vec<host::ActiveModel> = vec![];
            if let Some(dns) = &unit.match_.hosts {
                for host in dns {
                    let host = host::ActiveModel {
                        domain: ActiveValue::Set(host.to_owned()),
                        ..Default::default()
                    };
                    hosts.push(host);
                }
                let res = Host::insert_many(hosts.clone())
                    .on_conflict(
                        OnConflict::column(host::Column::Domain)
                            .do_nothing()
                            .to_owned(),
                    )
                    .do_nothing()
                    .exec_without_returning(&db)
                    .await
                    .into_diagnostic();
                // Populate entities with ids
                // debug!("{}", e);
                // println!("{}", e);
                let models = Host::find()
                    .filter(host::Column::Domain.is_in(dns))
                    .all(&db)
                    .await
                    .into_diagnostic()?;
                hosts = models
                    .iter()
                    .map(|x| host::ActiveModel::from(x.to_owned()))
                    .collect();
            };

            // Join Match and Host
            let mut list: Vec<match_host::ActiveModel> = vec![];
            for host in hosts {
                let match_host = match_host::ActiveModel {
                    match_id: match_.id.clone(),
                    host_id: host.id,
                };
                list.push(match_host)
            }
            let _ = MatchHost::insert_many(list)
                .on_conflict(
                    OnConflict::columns(vec![
                        match_host::Column::MatchId,
                        match_host::Column::HostId,
                    ])
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

#[cfg(test)]
mod test {
    use crate::database::{connect_db, fresh_db};
    use crate::{ConfigFile, Match};
    use entity::{prelude::*, *};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
    // Logging
    use tracing::{debug, Level};
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    async fn set_default_config() -> Result<()> {
        let db = fresh_db().await?;
        // Get struct from config
        let toml = "
        [[unit]]
        uuid = 'd3630938-5851-43ab-a523-84e0c6af9eb1'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com', 'example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'

        [[unit]]
        uuid = 'd462482d-21f7-48d6-8360-528f9e664c2f'
        listeners = ['*:443']
        [unit.match]
        uri = ['/home']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'

        [[unit]]
        uuid = 'cc4e626a-9354-480e-a78b-f9f845148984'
        listeners = ['*:443']
        [unit.match]
        hosts = ['api.example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8222'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        config.push().await?;
        let nginx_config = crate::nginx::db_into_nginx_conf().await?;
        // println!("{:#?}", nginx_config);
        Ok(())
    }

    #[tokio::test]
    async fn connect_to_db() -> Result<()> {
        connect_db().await?;
        Ok(())
    }

    #[tokio::test]
    async fn seed_db() -> Result<()> {
        set_default_config().await?;
        Ok(())
    }

    #[tokio::test]
    async fn remove_unit_by_uuid() -> Result<()> {
        set_default_config().await?;
        let toml = "
        [[unit]]
        uuid = 'd3630938-5851-43ab-a523-84e0c6af9eb1'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com', 'example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        config.remove_from_db().await?;
        let nginx_config = crate::nginx::db_into_nginx_conf().await?;
        Ok(())
    }
}
