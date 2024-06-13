use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env::temp_dir};
use tokio::task::spawn_local;
// Global vars
use crate::nginx::SETTINGS;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};
// exec
use crate::nginx::certificate::CertificateStore;
use crate::ssl::Fake as FakeCertificate;
use crate::ssl::Letsencrypt as LetsencryptCertificate;
use crate::{ssl, Nginx};
// Async
use futures::executor::block_on;

// Config file
use crate::cast::Config as ConfigFile;

use http::uri::Uri;
use std::env;

// Common structs to file config and unit config
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Action {
    // Reverse proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    // Public folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot: Option<String>,
    // Error
    #[serde(rename = "return")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_number: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<Action>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub hosts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub raw_params: Option<serde_json::Value>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Config {
//     pub listeners: HashMap<String, ListenerOpts>,
//     pub routes: HashMap<String, Vec<Route>>,
// }

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ListenerOpts {
    pub pass: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Tls {
    pub certificate: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Route {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub listeners: HashMap<String, ListenerOpts>,
    pub routes: HashMap<String, Vec<Route>>,
    pub settings: Option<serde_json::Value>,
}
impl Default for Config {
    fn default() -> Self {
        let settings = Some(
            serde_json::from_str(
                "
          {
            \"http\": {
              \"log_route\": true
            }
          }
            ",
            )
            .into_diagnostic()
            .unwrap(),
        );

        let listeners = HashMap::new();

        // Ensure routes is an array
        // Avoid Json inconsistency
        let routes = HashMap::new();

        Config {
            routes,
            listeners,
            settings,
        }
    }
}
impl Config {
    /**
     * Replace the in place configuration.
     */
    pub async fn set(config: &Config) -> Result<Config> {
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

        // Response conversion from Json to Rust type.
        match res {
            serde_json::Value::Object(res) => {
                if let Some(success) = res.get("success") {
                    println!("nginx-server: {}", success);
                } else if let Some(error) = res.get("error") {
                    return Err(Error::msg(error.to_string()));
                } else {
                    let message = format!(
                        "Unexpected error returned from nginx-server:\n 
                            {:#?}",
                        res
                    );
                    return Err(Error::msg(message));
                }
            }
            _ => {
                let message = format!(
                    "Unexpected value returned from nginx-server:\n
                    {}",
                    res
                );
                return Err(Error::msg(message));
            }
        };
        return Ok(config.clone());
    }

    /**
     * Get the nginx-unit configuration as a rust struct.
     */
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
}

#[cfg(test)]
mod tests {

    use crate::cast::Config as ConfigFile;
    use crate::nginx::{Config as NginxConfig, Nginx};
    use std::path::PathBuf;
    // Error handling
    use miette::Result;

    #[tokio::test]
    async fn get_config() -> Result<()> {
        let res = NginxConfig::get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn set_default_config() -> Result<()> {
        let res = NginxConfig::set(&NginxConfig::default()).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_config_from_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config_file = ConfigFile::load(path.to_str().unwrap())?;

        // let nginx_config = NginxConfig::from(&JuceConfig::from(&config_file)).await?;
        // let res = NginxConfig::set(&nginx_config).await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
