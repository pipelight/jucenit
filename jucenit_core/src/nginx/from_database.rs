use crate::nginx::config::{Action, ListenerOpts, Match, Route};
// Database
use crate::{ConfigFile, ConfigUnit, NginxConfig};
// Sea orm
// use indexmap::IndexMap;
use crate::database::connect_db;
use entity::{prelude::*, *};
use migration::{MatchHost, MatchListener, Migrator, MigratorTrait};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

pub async fn db_into_nginx_conf() -> Result<NginxConfig> {
    // Connect to database to retrieve config
    let db = connect_db().await?;

    let mut nginx_config = NginxConfig::default();

    // Select related listeners and match
    // And add them to config struct
    let listeners: Vec<(listener::Model, Vec<ng_match::Model>)> = Listener::find()
        .find_with_related(NgMatch)
        .all(&db)
        .await
        .into_diagnostic()?;

    for (listener, ng_matches) in listeners {
        // Convert to nginx struct
        let (ip_socket, listener) = ListenerOpts::from(&listener);
        // Append listeners and empty routes to nginx configuration
        nginx_config
            .listeners
            .insert(ip_socket.clone(), listener.clone());

        let route_name = format!("jucenit_[{}]", ip_socket);
        nginx_config.routes.insert(route_name.clone(), vec![]);

        // Select related  match and hosts
        // And add them to config struct
        let hosts: Vec<Vec<host::Model>> = ng_matches
            .load_many_to_many(Host, MatchHost, &db)
            .await
            .into_diagnostic()?;

        for (ng_match, hosts) in ng_matches.into_iter().zip(hosts.into_iter()) {
            // println!("{:#?}", ng_match);
            // Select related  match and action
            // And add them to config struct
            let action = ng_match
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
                    match_: Match::from(&ng_match, None),
                });
            } else {
                for host in hosts {
                    route.push(Route {
                        action: action.clone(),
                        match_: Match::from(&ng_match, Some(host)),
                    });
                }
            }
        }
        // let (match, action)
    }
    Ok(nginx_config)
}

#[cfg(test)]
mod test {

    use super::*;
    // Error Handling
    use miette::{Error, IntoDiagnostic, Result, WrapErr};

    #[tokio::test]
    async fn convert() -> Result<()> {
        let nginx_config = db_into_nginx_conf().await?;
        println!("{:#?}", nginx_config);
        Ok(())
    }
}
