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
use crate::cast::{Action, Config as ConfigFile, Match};

use http::uri::Uri;
use std::env;

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

        // Ensure usual web routes exist
        let mut routes = HashMap::new();
        routes.insert("jucenit_[*:80]".to_owned(), vec![]);
        routes.insert("jucenit_[*:443]".to_owned(), vec![]);

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

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub action: Option<Action>,
    #[serde(rename = "match")]
    pub match_: Match,
}

#[cfg(test)]
mod tests {

    use crate::cast::Config as ConfigFile;
    use crate::juce::Config as JuceConfig;
    use crate::nginx::{Config as NginxConfig, Nginx};
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
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::set(&NginxConfig::from(&JuceConfig::from(&config_file))).await?;
        println!("{:#?}", res);
        Ok(())
    }
}
