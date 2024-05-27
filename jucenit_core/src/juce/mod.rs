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
use crate::nginx::Config as NginxConfig;
use miette::{Error, IntoDiagnostic, Result};

/**
* A struct to store jucenit configuration and translate to unit when needed.
* Much more intuitive and closer to jucenit configuration file than the
* nginx-unit configuration.
*/
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub units: IndexMap<Match, Unit>,
}
/**
* Force push the configuration to nginx-unit.
*/
impl Config {
    pub async fn push(&self) -> Result<()> {
        let res = NginxConfig::set(&NginxConfig::from(self)).await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: Option<String>,
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
