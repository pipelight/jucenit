use crate::nginx::config::{Action, ListenerOpts, Match, Route};
// Database
use crate::{ConfigFile, ConfigUnit, NginxConfig};
// Sea orm
// use indexmap::IndexMap;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use migration::{MatchHost, MatchListener, Migrator, MigratorTrait};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase,
};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

impl NginxConfig {
    /**
     * Generate an nginx unit configuration
     * from what has been pushed to the database ever since
     */
    pub async fn pull() -> Result<NginxConfig> {
        let db = connect_db().await?;
        let mut nginx_config = NginxConfig::default();

        // Select related listeners and match
        // And add them to config struct
        let listeners: Vec<(listener::Model, Vec<ng_match::Model>)> = Listener::find()
            .find_with_related(NgMatch)
            .all(&db)
            .await
            .into_diagnostic()?;
        for (listener, matches) in listeners {
            // Append listeners and routes to nginx configuration
            let (ip_socket, listener) = ListenerOpts::from(&listener).await?;
            nginx_config
                .listeners
                .insert(ip_socket.clone(), listener.clone());
            let route_name = format!("jucenit_[{}]", ip_socket);
            nginx_config.routes.insert(route_name.clone(), vec![]);

            // Select related  match and hosts
            let matches: Vec<(ng_match::Model, Vec<host::Model>)> = NgMatch::find()
                .find_with_related(Host)
                .filter(
                    Condition::all().add(ng_match::Column::Id.is_in(matches.iter().map(|x| x.id))),
                )
                .all(&db)
                .await
                .into_diagnostic()?;

            for (match_, hosts) in &matches {
                let action = match_
                    .find_related(Action)
                    .one(&db)
                    .await
                    .into_diagnostic()?;
                // Convert to nginx struct
                let action = action.clone().map(|x| Action::from(&x));
                let route_name = format!("jucenit_[{}]", ip_socket);
                let route = nginx_config.routes.get_mut(&route_name);
                let route = route.unwrap();

                if hosts.is_empty() {
                    route.push(Route {
                        action: action.clone(),
                        match_: Match::from(&match_, None),
                    });
                } else {
                    for host in hosts {
                        route.push(Route {
                            action: action.clone(),
                            match_: Match::from(&match_, Some(host.to_owned())),
                        });
                    }
                }
            }
        }
        Ok(nginx_config)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    // Error Handling
    use miette::{Error, IntoDiagnostic, Result, WrapErr};
    use std::path::PathBuf;

    /**
     * Set a fresh testing environment:
     * - clean certificate store
     * - set minimal nginx configuration
     */
    async fn set_testing_config() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config = ConfigFile::load(path.to_str().unwrap())?;
        config.set().await?;

        Ok(())
    }

    #[tokio::test]
    async fn convert() -> Result<()> {
        set_testing_config().await?;
        let nginx_config = NginxConfig::pull().await?;
        println!("{:#?}", nginx_config);
        Ok(())
    }
}
