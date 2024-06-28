use crate::{
    nginx::config::crud::{Action, ListenerOpts, Match, Tls},
    CertificateStore, ConfigFile, ConfigUnit,
};
// Database / Sea orm
// use indexmap::IndexMap;
use crate::database::entity::{prelude::*, *};
use indexmap::IndexMap;
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};
use uuid::{uuid, Uuid};

use super::Route;

// impl From<&entity::prelude::Listener> for ListenerOpts {
impl ListenerOpts {
    pub async fn from(e: &listener::Model) -> Result<(String, ListenerOpts)> {
        // Bulk add certificates to listeners
        let certs: Vec<String> = CertificateStore::get_all_valid()
            .await?
            .into_keys()
            .into_iter()
            .collect();

        let tls: Option<Tls>;
        if certs.is_empty() || e.ip_socket.ends_with(":80") {
            tls = None
        } else {
            tls = Some(Tls { certificate: certs })
        }

        let tuples = (
            e.ip_socket.to_owned(),
            ListenerOpts {
                pass: format!("routes/jucenit_[{}]", e.ip_socket),
                tls,
            },
        );
        Ok(tuples)
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
            raw_params: serde_json::from_str(&e.raw_params).unwrap(),
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
    use serde_json::json;
    use uuid::{uuid, Uuid};
    // SeaOrm
    use crate::database::entity::{prelude::*, *};
    use sea_orm::{prelude::*, query::*, ActiveValue, TryIntoModel};
    // Error Handling
    use miette::{Error, IntoDiagnostic, Result, WrapErr};

    #[tokio::test]
    async fn convert_listener() -> Result<()> {
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
        let res = ListenerOpts::from(&listener).await?;
        assert_eq!(expect, res);
        Ok(())
    }
    #[test]
    fn convert_match() -> Result<()> {
        let host = host::ActiveModel {
            id: ActiveValue::Set(9),
            domain: ActiveValue::Set("example.com".to_owned()),
        };
        let action = action::ActiveModel {
            id: ActiveValue::Set(2),
            // raw_params: ActiveValue::Set("{}".to_owned()),
            ..Default::default()
        };
        let match_ = ng_match::ActiveModel {
            id: ActiveValue::Set(7),
            uuid: ActiveValue::Set(Uuid::new_v4().to_string()),
            raw_params: ActiveValue::Set(None),
            // raw_params: ActiveValue::Set("{}".to_owned()),
            action_id: ActiveValue::Set(2),
        };

        let expect = Match {
            host: Some("example.com".to_owned()),
            raw_params: None,
        };
        let res = Match::from(
            &match_.try_into_model().into_diagnostic()?,
            Some(host.try_into_model().into_diagnostic()?),
        );
        assert_eq!(expect, res);
        Ok(())
    }
    #[test]
    fn convert_action() -> Result<()> {
        let action = action::ActiveModel {
            id: ActiveValue::Set(2),
            raw_params: ActiveValue::Set(json!("{}").to_string()),
            ..Default::default()
        };
        let expect = Action {
            raw_params: Some(json!("{}")),
        };
        let res = Action::from(&action.try_into_model().into_diagnostic()?);
        assert_eq!(expect, res);
        Ok(())
    }
}
