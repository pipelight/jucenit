use super::{
    common::{Action, Match},
    config::Config as ConfigFile,
};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::future::Future;
use std::{collections::HashMap, env::temp_dir};
// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// exec
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use uuid::Uuid;

pub static SETTINGS: Lazy<Arc<Mutex<Settings>>> =
    Lazy::new(|| Arc::new(Mutex::new(Settings::default())));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub url: Option<String>,
    pub socket: Option<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            url: Some("http://127.0.0.1:8080".to_string()),
            socket: None,
        }
    }
}
impl Settings {
    fn get_url(&self) -> String {
        if let Some(url) = &self.url {
            return url.to_owned();
        } else if let Some(socket) = &self.socket {
            return socket.to_owned();
        } else {
            return Settings::default().url.unwrap();
        }
    }
}

// Unit identical structs
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub config: Config,
    pub certificates: serde_json::Value,
    pub status: serde_json::Value,
    #[serde(skip)]
    pub settings: Settings,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub listeners: HashMap<String, ListenerOpts>,
    pub routes: HashMap<String, Vec<Route>>,
}
impl Default for Config {
    fn default() -> Self {
        // Ensure usual web routes exist
        let mut routes = HashMap::new();
        routes.insert("jucenit_[*:80]".to_owned(), vec![]);
        routes.insert("jucenit_[*:443]".to_owned(), vec![]);

        let listeners = HashMap::new();
        Config { routes, listeners }
    }
}
impl Config {
    pub async fn get() -> Result<Config> {
        let settings = SETTINGS.lock().unwrap().clone();
        let config = reqwest::get(settings.get_url() + "/config")
            .await
            .into_diagnostic()?
            .json::<Config>()
            .await
            .into_diagnostic()?;
        Ok(config)
    }
    /**
     * You can only PUT object to replace the actual configuration
     */
    async fn set(&self, config: Config) -> Result<serde_json::Value> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/config")
            .body(serde_json::to_string(&config).into_diagnostic()?)
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;

        Ok(res)
    }
    fn merge(old: Config, new: Config) -> Result<Config> {
        let mut merged = old;
        for (key, value) in new.listeners.iter() {
            // merge new listeners to old
            merged.listeners.insert(key.to_owned(), value.to_owned());

            // merge routes based on uniq match
            // let route_name = format!("jucenit_[{}]", key);
            // let mut new_routes = new.routes.get(&route_name).unwrap().clone();
        }
        for (key, value) in new.routes.iter() {
            // If route already exists then fuse and dedup
            if let Some(route) = merged.routes.get_mut(key) {
                route.extend(value.to_owned());
                route.sort_by_key(|p| p.clone().match_);
                route.dedup_by_key(|p| p.clone().match_);
            } else {
                merged.routes.insert(key.to_owned(), value.to_owned());
            }
        }

        Ok(merged)
    }
    pub async fn edit(&self) -> Result<()> {
        // Make temporary file
        let uuid = Uuid::new_v4();
        let tmp_dir = "/tmp/jucenit";
        fs::create_dir_all(tmp_dir).into_diagnostic()?;
        let path = format!("/tmp/jucenit/jucenit.config.tmp.{}.toml", uuid);

        // Retrieve config
        let toml = toml::to_string_pretty(&ConfigFile::from(self)).into_diagnostic()?;
        // Create and write to file
        let mut file = fs::File::create(path.clone()).into_diagnostic()?;
        let bytes = toml.as_bytes();
        file.write_all(bytes).into_diagnostic()?;

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

        // Update nginx-unit config
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/config")
            .body(ConfigFile::load(&path)?.adapt()?)
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;

        // Clean up tmp files before exit
        fs::remove_file(path).into_diagnostic()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ListenerOpts {
    pub pass: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Certificate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub bundle: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
}

#[cfg(test)]
mod tests {

    use super::Config as ConfigUnit;
    use super::ConfigFile;

    use miette::Result;

    #[tokio::test]
    async fn get_config() -> Result<()> {
        let res = ConfigUnit::get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_config() -> Result<()> {
        let mut config = ConfigUnit::default();
        let res = config.set(ConfigUnit::default()).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_from_file() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let mut config = ConfigUnit::default();
        let res = config.set(ConfigUnit::from(&config_file)).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn merge_file_w_duplicates_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.more.toml")?;
        let new = ConfigUnit::from(&config_file);
        let old = ConfigUnit::get().await?;
        let res = ConfigUnit::merge(old, new)?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn merge_file_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.else.toml")?;
        let new = ConfigUnit::from(&config_file);
        let old = ConfigUnit::get().await?;
        let res = ConfigUnit::merge(old, new)?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn merge_config() -> Result<()> {
        let old = ConfigUnit::from(&ConfigFile::from_toml("../examples/jucenit.toml")?);
        let new = ConfigUnit::from(&ConfigFile::from_toml("../examples/jucenit.else.toml")?);
        let res = ConfigUnit::merge(old, new)?;
        println!("{:#?}", res);
        Ok(())
    }
}
