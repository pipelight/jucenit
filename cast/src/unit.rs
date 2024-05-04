use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::future::Future;
// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub listeners: HashMap<String, ListenerOpts>,
    pub routes: HashMap<String, Vec<Route>>,
}
impl Config {
    async fn get(&self) -> Result<Config> {
        let settings = SETTINGS.lock().unwrap().clone();
        let config = reqwest::get(settings.get_url() + "/config")
            .await
            .into_diagnostic()?
            .json::<Config>()
            .await
            .into_diagnostic()?;
        Ok(config)
    }
    async fn set(&self, config: Config) -> Result<()> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let config = client
            .post(settings.get_url() + "/config")
            .body(serde_json::to_string(&config).into_diagnostic()?)
            .send()
            .await
            .into_diagnostic()?
            .json::<Config>()
            .await
            .into_diagnostic()?;
            Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ListenerOpts {
    pub pass: String,
    pub tls: Option<Certificate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub bundle: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Route {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
}

// Common structs to file config and unit config
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Action {
    // Reverse proxy
    pub proxy: Option<String>,
    // Public folder
    pub share: serde_json::Value,
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
    pub uri: serde_json::Value,
    pub source: Option<Vec<String>>,
}

#[cfg(test)]
mod unit {

    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn get_config() -> Result<()> {
        let mut unit = Unit::default();
        let res = unit.config.get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_config() -> Result<()> {
        let mut unit = Unit::default();
        let res = unit.config.get().await?;
        println!("{:#?}", res);
        Ok(())
    }
}
