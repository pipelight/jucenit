//!
//! The main/core struct to be manipulated
//!
//! Jucenit uses a kind of main store struct that eases the generation of
//! an nginx-unit Json configuration.
//!
//! This is a powerful intermediate
//! that is, in the end, lossy converted to a nginx-unit configuration.
//!
mod from;
use crate::{Action, Match};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

/**
* A struct to store jucenit configuration and translate to unit when needed.
* Much more intuitive and closer to jucenit configuration file than the
* nginx-unit configuration.
*/
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
struct Config {
    pub units: IndexMap<Match, Unit>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub action: Option<Action>,
    pub listeners: Vec<String>,
    pub kind: UnitKind,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitKind {
    #[default]
    Managed,
    SslChallenge,
    Unmanaged,
}
