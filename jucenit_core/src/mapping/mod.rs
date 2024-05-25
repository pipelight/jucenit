use crate::cast::{Action, Match};
use std::{collections::HashMap, env::temp_dir};

use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub listeners: HashMap<String, ListenerOpts>,
    pub routes: HashMap<String, Vec<Route>>,
    // pub settings: Option<Settings>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ListenerOpts {
    pub pass: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Tls {
    pub certificate: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
}
