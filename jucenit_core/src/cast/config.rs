use super::common::{Action, Match};
use crate::nginx::Config as NginxConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utils::teleport::Portal;
// Error Handling
use crate::error::{TomlError, YamlError};
use miette::{Error, IntoDiagnostic, Result};
// Filesystem
use std::fs;
use std::path::Path;
use utils::files::FileType;

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
        let config = NginxConfig::from(self);
        let res = serde_json::to_string_pretty(&config).into_diagnostic()?;
        println!("{}", res);
        Ok(res)
    }

    /**
    Choose the appropriated method to load the config file
    according to the file extension(.ts, .toml, .yml...).

    Arguments:
      - file_path is the config file path
      - args are only to be used with scripting language (typescript) to pass args to the underlying script.

    Languages coming next after v1.0.0:
      - Rust, Hcl, Kcl, Python...
    */
    pub fn load(file_path: &str) -> Result<Config> {
        let extension = &Path::new(file_path)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let file_type = FileType::from(extension);
        let mut config = match file_type {
            FileType::Toml | FileType::Tml => Config::from_toml(file_path)?,
            FileType::Yaml | FileType::Yml => Config::from_yaml(file_path)?,
            _ => {
                let msg = format!("File type is unknown");
                return Err(Error::msg(msg));
            }
        };
        Ok(config)
    }
    /**
    Returns a Config struct from a provided toml file path.
    */
    pub fn from_toml(file_path: &str) -> Result<Config> {
        let tml = fs::read_to_string(file_path).into_diagnostic()?;
        let res = toml::from_str::<Config>(&tml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = TomlError::new(e, &tml);
                Err(err.into())
            }
        }
    }
    /**
     * Returns a Config struct from a provided yaml file path.
     */
    pub fn from_yaml(file_path: &str) -> Result<Config> {
        let yml = fs::read_to_string(file_path).into_diagnostic()?;
        let res = serde_yaml::from_str::<Config>(&yml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = YamlError::new(e, &yml);
                Err(err.into())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
    pub listeners: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::Config as ConfigFile;
    use miette::Result;

    #[test]
    fn read_config_file() -> Result<()> {
        let res = ConfigFile::from_toml("../examples/jucenit.toml")?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn adapt_config_file() -> Result<()> {
        let res = ConfigFile::from_toml("../examples/jucenit.toml")?;
        res.adapt()?;
        Ok(())
    }
    #[test]
    /**
     * Test loading a file from a given path
     */
    fn read_toml() -> Result<()> {
        let res = ConfigFile::from_toml("../examples/jucenit.toml")?;
        println!("{:#?}", res);
        Ok(())
    }

    #[test]
    fn seek_file() -> Result<()> {
        let res = ConfigFile::get()?;
        println!("{:#?}", res);
        Ok(())
    }
}
