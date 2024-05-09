use super::common::{Action, Match};
use super::unit::Config as ConfigUnit;
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
    pub unit: Vec<Unit>,
}

impl Config {
    pub fn get() -> Result<Config> {
        let mut portal = Portal::new()?;
        portal.seed("jucenit");

        let res = portal.search()?;
        let config = Config::load(&portal.target.file_path.unwrap())?;
        Ok(config)
    }
    pub fn adapt(&self) -> Result<String> {
        let config = ConfigUnit::from(self);
        let res = serde_json::to_string_pretty(&config).into_diagnostic()?;
        println!("{}", res);
        Ok(res)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
    pub listeners: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use miette::Result;

    #[test]
    fn read_config_file() -> Result<()> {
        let res = Config::from_toml("../examples/jucenit.toml")?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn adapt_config_file() -> Result<()> {
        let res = Config::from_toml("../examples/jucenit.toml")?;
        res.adapt()?;
        Ok(())
    }
}
