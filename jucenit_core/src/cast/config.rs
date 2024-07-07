use crate::nginx::Config as NginxConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Error Handling
use crate::error::{TomlError, YamlError};
use miette::{Error, IntoDiagnostic, Result};
// Filesystem
use std::fs;
use std::path::Path;

// use utils::{files::FileType, teleport::Portal};
use pipelight::utils::{error::PipelightError, files::FileType, teleport::Portal};

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
    /**
     * Search the filesystem for a config file.
     */
    pub fn get() -> Result<Config> {
        let mut portal = Portal::new().into_diagnostic()?;
        portal.seed("jucenit");
        let res = portal.search().into_diagnostic()?;
        let config = Config::load(&portal.target.file_path.unwrap())?;
        Ok(config)
    }

    /**
     * Choose the appropriated method to load the config file
     * according to the file extension(.toml or .yml).
     *
     *  Arguments:
     *  - file_path is the config file path
     *  - args are only to be used with scripting language (typescript) to pass args to the underlying script.
     */
    pub fn load(file_path: &str) -> Result<Config> {
        // TODO: add Hcl and Kcl.
        let extension = &Path::new(file_path)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let file_type = FileType::from(extension);
        let config = match file_type {
            FileType::Toml | FileType::Tml => Config::from_toml_file(file_path)?,
            FileType::Yaml | FileType::Yml => Config::from_yaml_file(file_path)?,
            _ => {
                let msg = format!("File type is unknown");
                return Err(Error::msg(msg));
            }
        };
        Ok(config)
    }
    /**
    Returns a jucenit configuration from a provided toml file path.
    */
    pub fn from_toml_file(file_path: &str) -> Result<Config> {
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
    Returns a jucenit configuration from a provided toml string.
    */
    pub fn from_toml_str(toml: &str) -> Result<Self> {
        let res = toml::from_str::<Self>(&toml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = TomlError::new(e, toml);
                Err(err.into())
            }
        }
    }
    pub fn to_toml(&self) -> Result<String> {
        let res = toml::to_string_pretty(self).into_diagnostic();
        res
    }
    /**
     * Returns a jucenit configuration from a provided yaml file path.
     */
    pub fn from_yaml_file(file_path: &str) -> Result<Config> {
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
    /**
    Returns a jucenit configuration from a provided yaml string.
    */
    pub fn from_yaml_str(yml: &str) -> Result<Self> {
        let res = serde_yaml::from_str::<Self>(&yml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = YamlError::new(e, &yml);
                Err(err.into())
            }
        }
    }
    pub fn to_yaml(&self) -> Result<String> {
        let res = serde_yaml::to_string(self).into_diagnostic();
        res
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub uuid: String,
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
    pub listeners: Vec<String>,
}
impl Unit {
    /**
    Returns a jucenit configuration from a provided toml string.
    */
    pub fn from_toml_str(toml: &str) -> Result<Self> {
        let res = toml::from_str::<Self>(&toml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = TomlError::new(e, toml);
                Err(err.into())
            }
        }
    }
    pub fn to_toml(&self) -> Result<String> {
        let res = toml::to_string_pretty(self).into_diagnostic();
        res
    }
    /**
    Returns a jucenit configuration from a provided yaml string.
    */
    pub fn from_yaml_str(yml: &str) -> Result<Self> {
        let res = serde_yaml::from_str::<Self>(&yml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = YamlError::new(e, &yml);
                Err(err.into())
            }
        }
    }
    pub fn to_yaml(&self) -> Result<String> {
        let res = serde_yaml::to_string(self).into_diagnostic();
        res
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Action {
    // Reverse proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub raw_params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub raw_params: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::Config as ConfigFile;
    use miette::Result;

    #[test]
    fn get_from_toml_file() -> Result<()> {
        let res = ConfigFile::from_toml_file("../examples/jucenit.toml")?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn seek_a_config_file() -> Result<()> {
        let res = ConfigFile::get()?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn get_from_toml_string() -> Result<()> {
        let toml = "
        [[unit]]
        uuid = 'd3630938-5851-43ab-a523-84e0c6af9eb1'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com', 'example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'
       ";
        let res = ConfigFile::from_toml_str(toml)?;
        println!("{:#?}", res);
        Ok(())
    }
}
