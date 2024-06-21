use crate::cast::Config as ConfigFile;
use crate::{Action, Match};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
// File
use crate::nginx::Config as NginxConfig;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Sea orm
// use indexmap::IndexMap;
use entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
use sea_orm::{Database, DatabaseConnection};

// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub async fn connect_db() -> Result<DatabaseConnection> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let db: DatabaseConnection = Database::connect(database_url).await.into_diagnostic()?;
    Ok(db)
}
pub async fn fresh_db() -> Result<()> {
    let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
    let connection = sea_orm::Database::connect(database_url)
        .await
        .into_diagnostic()?;
    Migrator::fresh(&connection).await.into_diagnostic()?;
    Ok(())
}

pub static JUCENIT_CONFIG: Lazy<Arc<Mutex<Config>>> =
    Lazy::new(|| Arc::new(Mutex::new(Config::default())));

/**
 * Merge a chunk into the already existing configuration.
 * Apply the result to nginx-unit.
 * Serialize the configuration into the lock file at /var/spool/jucenit/config.rs
 */
pub async fn push(chunk: &Config) -> Result<()> {
    // Merge chunk to main configuration
    // and try push to nginx-unit.
    // let mut main = Self::pull().await?;
    // Config::merge(&mut main, chunk)?;
    //
    // let nginx = NginxConfig::from(&main).await?;
    // NginxConfig::set(&nginx).await?;
    //
    // Self::serialize(&main).await?;
    Ok(())
}
/**
 * Deserialize the configuration from the lock file at /var/spool/jucenit/config.rs
 * Returns jucenit global configuration.
 */
pub async fn pull() -> Result<Config> {
    if JUCENIT_CONFIG.lock().unwrap().clone() == Config::default() {
        let config = match Self::deserialize().await {
            Ok(res) => res,
            Err(_) => Config::default(),
        };
        *JUCENIT_CONFIG.lock().unwrap() = config;
    }
    let config = JUCENIT_CONFIG.lock().unwrap().clone();
    Ok(config)
}

/**
 * Insert a (Match,Unit) tuple into the configuration.
 */
pub async fn add_unit((match_, unit): (Match, Unit)) -> Result<()> {
    // let mut main = Self::pull().await?;
    // if unit.kind == UnitKind::HttpChallenge {
    //     // Clean unremoved challenges assossiated to host if any.
    //     for k in main.units.clone().keys() {
    //         if let Some(uri) = &k.uri {
    //             if uri.contains("/.well-known/acme-challenge/") && k.host == match_.host {
    //                 main.units.shift_remove(k);
    //             }
    //         }
    //     }
    //     main.units.shift_insert(0, match_, unit);
    // } else {
    //     main.units.insert(match_, unit);
    // }
    // let nginx = NginxConfig::from(&main).await?;
    // NginxConfig::set(&nginx).await?;
    // Config::set(&main).await?;
    Ok(())
}
/**
 * Delete a (Match,Unit) tuple from the configuration.
 */
pub async fn del_unit(match_: Match) -> Result<()> {
    // let mut main = Self::pull().await?;
    // main.units.shift_remove(&match_);
    // Config::set(&main).await?;
    //
    Ok(())
}

pub async fn edit(&self) -> Result<()> {
    let tmp_dir = "/tmp/jucenit";
    fs::create_dir_all(tmp_dir).await.into_diagnostic()?;
    let path = "/tmp/jucenit/jucenit.config.tmp.toml".to_owned();

    // Retrieve config
    let toml = ConfigFile::from(self).to_toml()?;
    // Create and write to file
    let mut file = fs::File::create(path.clone()).await.into_diagnostic()?;
    let bytes = toml.as_bytes();
    file.write_all(bytes).await.into_diagnostic()?;

    // Modify file with editor
    let editor = env::var("EDITOR").into_diagnostic()?;
    let child = Command::new(editor)
        .arg(path.clone())
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Couldn't spawn a detached subprocess");
    let output = child.wait_with_output().into_diagnostic()?;

    // Try Update nginx-unit config
    let tmp_config = Config::from(&ConfigFile::load(&path)?);
    Config::set(&tmp_config).await?;

    // Clean up tmp files before exit
    fs::remove_file(path).await.into_diagnostic()?;
    Ok(())
}

/**
 * Returns every hosts in configuration.
 */
pub async fn get_hosts() -> Result<Vec<String>> {
    let main = Self::pull().await?;
    let hosts: Vec<String> = main
        .units
        .into_keys()
        .filter(|e| e.host.is_some())
        .map(|e| e.host.unwrap())
        .collect();
    // println!("{:?}", hosts);
    Ok(hosts)
}

/**
 * Merge a configuration chunk to the main configuration.
 */
// fn merge(main: &mut Config, chunk: &Config) -> Result<Config> {
// main.units.extend(chunk.units.clone());
// Ok(main.to_owned())
// }

// Methods below are for convernient serializing.

async fn serialize(config: &Config) -> Result<()> {
    // Create/Ensure directory
    let data_dir = "/var/spool/jucenit";
    let message = format!("Couldn't create data directory at: {:?}", data_dir);
    let res = fs::create_dir_all(data_dir)
        .await
        .into_diagnostic()
        .wrap_err(message)?;

    let file_path = format!("{}/config.rs", &data_dir);

    // Write configuration to lock file
    // let serialized = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())
    //     .into_diagnostic()?;
    let serialized = ron::ser::to_string(config).into_diagnostic()?;
    let bytes = serialized.as_bytes();

    let message = format!("Couldn't write to data file at: {:?}", file_path);
    let mut file = fs::File::create(&file_path)
        .await
        .into_diagnostic()
        .wrap_err(message)?;

    file.write_all(bytes).await.into_diagnostic()?;

    // Try to set main configuration file permissions (loose)
    // and fail silently
    let metadata = file.metadata().await.into_diagnostic()?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o774);
    fs::set_permissions(&file_path, perms)
        .await
        .into_diagnostic()
        .ok();

    Ok(())
}
async fn deserialize() -> Result<Config> {
    let data_dir = "/var/spool/jucenit";
    let file_path = format!("{}/config.rs", &data_dir);

    // Read lock file
    let message = format!("Couldn't read data file at: {:?}", &file_path);

    let mut file = fs::File::open(&file_path)
        .await
        .into_diagnostic()
        .wrap_err(message)?;

    let mut content = String::new();
    file.read_to_string(&mut content).await.into_diagnostic()?;
    let config: Config = ron::from_str(&content).into_diagnostic()?;

    Ok(config)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: Option<String>,
    pub action: Option<Action>,
    pub listeners: Vec<String>,
    pub kind: UnitKind,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitKind {
    #[default]
    Managed,
    HttpChallenge,
    TlsAlpnChallenge,
    Unmanaged,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Match {
//     pub host: Option<String>,
//     pub uri: Option<String>,
//     pub source: Option<Vec<String>>,
//     pub listener: String,
// }

#[cfg(test)]
mod tests {
    use super::Config as JuceConfig;
    use crate::cast::Config as ConfigFile;
    use miette::Result;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[tokio::test]
    async fn deserialize_config() -> Result<()> {
        let res = JuceConfig::pull().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn set_global_config() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config_file = ConfigFile::load(path.to_str().unwrap())?;

        let res = JuceConfig::from(&config_file);
        JuceConfig::set(&res).await?;
        Ok(())
    }
    #[tokio::test]
    async fn push_config_chunk() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config_file = ConfigFile::load(path.to_str().unwrap())?;
        let config = JuceConfig::from(&config_file);
        let res = JuceConfig::push(&config).await?;
        Ok(())
    }
}
