use serde::{Deserialize, Serialize};
use std::default::Default;
use std::{collections::HashMap, env::temp_dir};
use tokio::task::spawn_local;
// Global vars
use crate::nginx::SETTINGS;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
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
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use uuid::Uuid;

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
    pub async fn set(config: &Config) -> Result<serde_json::Value> {
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

    async fn clean() -> Result<serde_json::Value> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/config")
            .body(serde_json::to_string(&Config::default()).into_diagnostic()?)
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;
        Ok(res)
    }
    pub async fn update(new: &Config) -> Result<()> {
        let old = Config::get().await?;
        let res = Config::merge(&old, &new)?;
        Config::set(&res).await?;
        Ok(())
    }
    pub async fn delete(new: &Config) -> Result<()> {
        let old = Config::get().await?;
        let res = Config::unmerge(&old, &new)?;
        Config::set(&res).await?;
        Ok(())
    }
    pub async fn get_hosts() -> Result<Vec<String>> {
        let config = Config::get().await?;
        let mut hosts = vec![];

        for (name, routes) in config.routes.into_iter() {
            for route in routes {
                if let Some(host) = route.match_.host {
                    hosts.push(host);
                }
            }
        }

        hosts.sort();
        hosts.dedup();
        Ok(hosts)
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
        Self::set(&Self::from(&ConfigFile::load(&path)?)).await?;

        // spawn_local(async move {
        //     CertificateStore::hydrate().await.unwrap();
        // });
        CertificateStore::hydrate().await.unwrap();

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
    use crate::nginx::{Config, Nginx};
    // Error handling
    use miette::Result;

    #[tokio::test]
    async fn clean_config() -> Result<()> {
        let res = Config::clean().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn get_config() -> Result<()> {
        let res = Config::get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_config() -> Result<()> {
        let res = Config::set(&Config::default()).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn set_from_file() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = Config::set(&Config::from(&config_file)).await?;
        println!("{:#?}", res);
        Ok(())
    }
}
