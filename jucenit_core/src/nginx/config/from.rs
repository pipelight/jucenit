use crate::{
    nginx::config::crud::{Action, ListenerOpts, Match},
    ConfigFile, ConfigUnit,
};
// Database / Sea orm
// use indexmap::IndexMap;
use entity::*;
use indexmap::IndexMap;
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

use super::Route;

// impl From<&entity::prelude::Listener> for ListenerOpts {
impl ListenerOpts {
    pub fn from(e: &listener::Model) -> (String, ListenerOpts) {
        let tuples = (
            e.ip_socket.to_owned(),
            ListenerOpts {
                pass: format!("routes/jucenit_[{}]", e.ip_socket),
                tls: None,
            },
        );
        tuples
    }
}

impl Match {
    pub fn from(e: &ng_match::Model, h: Option<host::Model>) -> Match {
        let mut host: Option<String> = None;
        if let Some(h) = h {
            host = Some(h.domain);
        }
        let match_ = Match {
            host,
            raw_params: e
                .raw_params
                .clone()
                .map(|x| serde_json::from_str(&x).unwrap()),
        };
        match_
    }
}
impl Action {
    pub fn from(e: &action::Model) -> Action {
        let action = Action {
            raw_params: e
                .raw_params
                .clone()
                .map(|x| serde_json::from_str(&x).unwrap()),
        };
        action
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        nginx::config::crud::{Action, ListenerOpts, Match},
        ConfigFile, ConfigUnit,
    };
    // SeaOrm
    use entity::*;
    // Error Handling
    use miette::{Error, IntoDiagnostic, Result, WrapErr};

    #[test]
    fn convert_listener() -> Result<()> {
        let listener = listener::Model {
            id: 4,
            ip_socket: "*:8082".to_owned(),
            tls: None,
        };
        let expect = (
            "*:8082".to_owned(),
            ListenerOpts {
                pass: "routes/jucenit_[*:8082]".to_owned(),
                tls: None,
            },
        );
        let res = ListenerOpts::from(&listener);
        assert_eq!(expect, res);
        Ok(())
    }
    #[test]
    fn convert_match() -> Result<()> {
        let host = host::Model {
            id: 9,
            domain: "example.com".to_owned(),
        };
        let match_ = ng_match::Model {
            id: 7,
            raw_params: None,
        };
        let action = action::Model {
            id: 2,
            raw_params: None,
        };
        let expect = Match {
            host: Some("example.com".to_owned()),
            raw_params: None,
        };
        let res = Match::from(&match_, Some(host));
        assert_eq!(expect, res);
        Ok(())
    }
    #[test]
    fn convert_action() -> Result<()> {
        let action = action::Model {
            id: 8,
            raw_params: None,
        };
        let expect = Action { raw_params: None };
        let res = Action::from(&action);
        assert_eq!(expect, res);
        Ok(())
    }
}
