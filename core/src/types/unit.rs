use super::{
    common::{Action, Match},
    config::Config as ConfigFile,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::future::Future;
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
use std::process::Command;
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
        // Ensure an empty named route array exists
        let mut routes = HashMap::new();
        routes.insert("jucenit".to_owned(), vec![]);

        let listeners = HashMap::new();
        Config { routes, listeners }
    }
}
impl Config {
    async fn get() -> Result<Config> {
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
    fn merge(mut old: Config, new: Config) -> Result<Config> {
        // merge listeners
        for (key, value) in new.listeners.iter() {
            old.listeners.insert(key.to_owned(), value.to_owned());
        }
        // merge routes based on uniq match

        let mut new_routes = new.routes.get("jucenit").unwrap().clone();
        let mut old_routes = old.routes.get("jucenit").unwrap().clone();
        old_routes.append(&mut new_routes);

        old_routes.sort_by_key(|p| p.clone().match_);
        old_routes.dedup_by_key(|p| p.clone().match_);

        old.routes
            .insert("jucenit".to_owned(), old_routes.to_owned());

        Ok(old)
    }
    pub async fn edit(&self) -> Result<()> {
        let uuid = Uuid::new_v4();
        let path = format!("~/jucenit.{}.tmp.json", uuid);

        // Retrieve config
        let config = Config::get().await?;
        let json = serde_json::to_string_pretty(&config).into_diagnostic()?;

        // Create and write to file
        let mut file = fs::File::create(path.clone()).into_diagnostic()?;
        let bytes = json.as_bytes();
        file.write_all(bytes).into_diagnostic()?;

        // Modify file with editor
        let editor = env::var("EDITOR").into_diagnostic()?;
        let output = Command::new(editor)
            .arg(path)
            .output()
            .expect("failed to execute process");

        // Update config
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/config")
            .body(output.stdout)
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;

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
}
