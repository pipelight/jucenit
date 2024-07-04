use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::env::temp_dir;
use tokio::task::spawn_local;
// Global vars
use crate::nginx::SETTINGS;
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
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub raw_params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub raw_params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ListenerOpts {
    pub pass: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Tls>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
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
    pub listeners: IndexMap<String, ListenerOpts>,
    pub routes: IndexMap<String, Vec<Route>>,
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

        let listeners = IndexMap::new();

        // Ensure routes is an array
        // Avoid Json inconsistency
        let routes = IndexMap::new();
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
    pub async fn set(&self) -> Result<Config> {
        let settings = SETTINGS.lock().await.clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/config")
            .body(serde_json::to_string(&self).into_diagnostic()?)
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
        return Ok(self.clone());
    }

    /**
     * Get the nginx-unit configuration as a rust struct.
     */
    pub async fn get() -> Result<Config> {
        let settings = SETTINGS.lock().await.clone();
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
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn get_config() -> Result<()> {
        let res = NginxConfig::get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_default_config() -> Result<()> {
        let res = NginxConfig::set(&NginxConfig::default()).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_config_from_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config = ConfigFile::load(path.to_str().unwrap())?;
        config.set().await?;

        let nginx = NginxConfig::pull().await?;

        let json = serde_json::to_string_pretty(&nginx).into_diagnostic()?;
        println!("{}", json);

        let res = nginx.set().await?;
        println!("{:#?}", res);

        Ok(())
    }
}
