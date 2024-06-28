use super::{Action, Match};

use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult};

impl Action {
    pub fn from(e: &action::Model) -> Action {
        let action = Action {
            raw_params: serde_json::from_str(&e.raw_params).unwrap(),
        };
        action
    }
}

impl Match {
    pub fn from(e: &ng_match::Model, hosts: Vec<host::Model>) -> Self {
        let domain_names: Vec<String> = hosts.iter().map(|x| x.clone().domain).collect();
        let hosts;
        if domain_names.is_empty() {
            hosts = None;
        } else {
            hosts = Some(domain_names)
        }
        let match_ = Self {
            hosts,
            raw_params: e
                .raw_params
                .clone()
                .map(|x| serde_json::from_str(&x).unwrap()),
        };
        match_
    }
}
