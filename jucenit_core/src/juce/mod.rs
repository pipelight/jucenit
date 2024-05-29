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
// File
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fs, io::Read};
// Error Handling
use crate::nginx::Config as NginxConfig;
use miette::{Error, IntoDiagnostic, Result, WrapErr};

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub static JUCENIT_CONFIG: Lazy<Arc<Mutex<Config>>> =
    Lazy::new(|| Arc::new(Mutex::new(Config::default())));

/**
* A struct to store jucenit configuration and translate to unit when needed.
* Much more intuitive and closer to jucenit configuration file than the
* nginx-unit configuration.
*/
#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub units: IndexMap<Match, Unit>,
}

/**
* Force push the configuration to nginx-unit.
*/
impl Config {
    /**
     * Merge a chunk into the already existing configuration.
     * Apply the result to nginx-unit.
     * Serialize the configuration into the lock file at /var/spool/jucenit/config.rs
     */
    pub async fn push(chunk: &Config) -> Result<()> {
        // Merge chunk to main configuration
        // and try push to nginx-unit.
        let mut main = Self::pull()?;
        Config::merge(&mut main, chunk)?;
        NginxConfig::set(&NginxConfig::from(&main)).await?;
        Self::serialize(&main)?;

        Ok(())
    }
    /**
     * Deserialize the configuration from the lock file at /var/spool/jucenit/config.rs
     * Returns jucenit global configuration.
     */
    pub fn pull() -> Result<Config> {
        if JUCENIT_CONFIG.lock().unwrap().clone() == Config::default() {
            let config = Self::deserialize()?;
            *JUCENIT_CONFIG.lock().unwrap() = config;
        }
        let config = JUCENIT_CONFIG.lock().unwrap().clone();
        Ok(config)
    }

    /**
     * Insert a (Match,Unit) tuple into the configuration.
     */
    pub async fn add_unit((match_, unit): (Match, Unit)) -> Result<()> {
        let mut main = Self::pull()?;
        if unit.kind == UnitKind::SslChallenge {
            main.units.shift_insert(0, match_, unit);
        } else {
            main.units.insert(match_, unit);
        }

        NginxConfig::set(&NginxConfig::from(&main)).await?;
        Config::set(&main).await?;
        Ok(())
    }
    /**
     * Delete a (Match,Unit) tuple from the configuration.
     */
    pub async fn del_unit(match_: Match) -> Result<()> {
        let mut main = Self::pull()?;
        main.units.shift_remove(&match_);
        Config::set(&main).await?;

        Ok(())
    }

    /**
     * Merge a configuration chunk to the main configuration.
     */
    fn merge(main: &mut Config, chunk: &Config) -> Result<Config> {
        main.units.extend(chunk.units.clone());
        Ok(main.to_owned())
    }

    /**
     * DANGER!
     * Set the chunk as the global configuration, erasing the previous global configuration
     */
    pub async fn set(chunk: &Config) -> Result<()> {
        NginxConfig::set(&NginxConfig::from(chunk)).await?;
        Self::serialize(chunk)?;
        Ok(())
    }

    // Methods below are for convernient serializing.

    fn serialize(config: &Config) -> Result<()> {
        let file_path = Self::ensure_lock_file()?;

        // Write configuration to lock file
        let serialized = ron::to_string(config).into_diagnostic()?;
        let bytes = serialized.as_bytes();

        let message = format!("Couldn't write to data file at: {:?}", file_path);
        let mut file = fs::File::create(&file_path)
            .into_diagnostic()
            .wrap_err(message)?;

        file.write_all(bytes).into_diagnostic()?;

        Ok(())
    }
    fn deserialize() -> Result<Config> {
        let file_path = Self::ensure_lock_file()?;

        // Read lock file
        let message = format!("Couldn't read data file at: {:?}", file_path);
        let content = fs::read(&file_path).into_diagnostic().wrap_err(message)?;
        let string = String::from_utf8_lossy(&content);

        // If file is empty return default config
        let config: Config = match ron::from_str(&string).into_diagnostic() {
            Ok(res) => res,
            Err(e) => Config::default(),
        };
        Ok(config)
    }
    fn ensure_lock_file() -> Result<String> {
        // Create/Ensure directory
        let data_dir = "/var/spool/jucenit";
        let message = format!("Couldn't create data directory at: {:?}", data_dir);
        let res = fs::create_dir_all(data_dir)
            .into_diagnostic()
            .wrap_err(message)?;

        let file_path = format!("{}/config.rs", &data_dir);
        let message = format!("Couldn't create data file at: {:?}", file_path);
        let file = fs::File::create(&file_path)
            .into_diagnostic()
            .wrap_err(message)?;

        // Set main configuration file permissions (loose)
        let metadata = file.metadata().into_diagnostic()?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o766);
        fs::set_permissions(&file_path, perms).into_diagnostic()?;

        Ok(file_path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::Config as JuceConfig;
    use crate::cast::Config as ConfigFile;
    use miette::Result;
    use serde::Deserialize;

    #[tokio::test]
    async fn set_global_config() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = JuceConfig::from(&config_file);
        JuceConfig::set(&res).await?;
        Ok(())
    }
    #[test]
    fn deserialize_config() -> Result<()> {
        let res = JuceConfig::pull()?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn push_config_chunk() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let config = JuceConfig::from(&config_file);
        let res = JuceConfig::push(&config).await?;
        println!("{:#?}", res);
        Ok(())
    }
}
