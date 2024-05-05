use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utils::teleport::Portal;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

// Config file related structs
/**
*  Config file
*/
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub unit: Option<Vec<Unit>>,
}

impl Config {
    pub fn get() -> Result<Config> {
        let mut portal = Portal::new()?;
        portal.seed("jucenit");

        let res = portal.search()?;
        let config = Config::load(&portal.target.file_path.unwrap())?;
        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Option<Match>,
    pub listeners: Option<Vec<String>>,
}

// Common structs to file config and unit config
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Action {
    // Reverse proxy
    pub proxy: Option<String>,
    // Public folder
    pub share: Option<Vec<String>>,
    pub chroot: Option<String>,
    // Error
    #[serde(rename = "return")]
    pub return_number: Option<String>,

    pub rewrite: Option<String>,
    pub pass: Option<String>,

    pub fallback: Option<Box<Action>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Match {
    pub uri: Option<Vec<String>>,
    pub source: Option<Vec<String>>,
}
