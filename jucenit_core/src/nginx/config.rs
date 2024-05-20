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
use super::CertificateStore;
use crate::ssl;
use crate::ssl::Fake as FakeCertificate;
use crate::ssl::Letsencrypt as LetsencryptCertificate;

// Config file
use crate::cast::{Action, Config as ConfigFile, Match};

use http::uri::Uri;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{clone, env};
use uuid::Uuid;

pub static SETTINGS: Lazy<Arc<Mutex<Settings>>> =
    Lazy::new(|| Arc::new(Mutex::new(Settings::default())));

/*
* A struct to query the good nginx-unit socket or port.
*/
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
    pub fn get_url(&self) -> String {
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
pub struct Nginx {
    pub config: Config,
    pub certificates: serde_json::Value,
    pub status: serde_json::Value,
    #[serde(skip)]
    pub settings: Settings,
}

impl Nginx {
    /**
     * Generate and upload certificates to unit
     */
    pub async fn set_certificates() -> Result<(serde_json::Value)> {
        let mut config = Config::get().await?;
        for (key, val) in config.routes.iter() {
            for route in val {
                if let Some(host) = &route.match_.host {
                    let dns = host;
                    let account = ssl::pebble::pebble_account().await?.clone();
                    let res = CertificateStore::update(
                        dns,
                        &LetsencryptCertificate::get(dns, &account).await?,
                    )
                    .await?;
                    println!("{:#?}", res);
                    for (key, val) in config.listeners.iter_mut() {
                        if let Some(tls) = &mut val.tls {
                            tls.certificate.push(host.to_owned());
                        } else {
                            val.tls = Some(Tls {
                                certificate: vec![host.to_owned()],
                            });
                        }
                    }
                }
            }
        }
        println!("{:#?}", config);
        let res = Config::set(&config).await?;
        println!("{:#?}", res);
        Ok(res)
    }
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
        let res = CertificateStore::remove_all().await?;
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

    fn merge(old: &Config, new: &Config) -> Result<Config> {
        let mut merged = old.to_owned();
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
    fn unmerge(old: &Config, new: &Config) -> Result<Config> {
        let mut unmerged = old.to_owned();
        for (key, old_route) in unmerged.routes.iter_mut() {
            if let Some(new_route) = new.routes.get(key) {
                for step in new_route.clone() {
                    if let Some(index) = old_route.iter().position(|e| e == &step) {
                        old_route.remove(index);
                    }
                }
            }
        }
        Ok(unmerged)
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

    use super::{Config, Nginx};
    use crate::cast::Config as ConfigFile;
    // Error handling
    use miette::Result;

    // #[tokio::test]
    async fn clean_config() -> Result<()> {
        let res = Config::clean().await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn remove_certs() -> Result<()> {
        // let res = Unit::remove_certificates().await?;
        // println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn get_config() -> Result<()> {
        let res = Config::get().await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn set_config() -> Result<()> {
        let res = Config::set(&Config::default()).await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn set_from_file() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = Config::set(&Config::from(&config_file)).await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn set_from_file_and_tls() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = Config::set(&Config::from(&config_file)).await?;
        Nginx::set_certificates().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn merge_file_w_duplicates_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.merge2.toml")?;
        let new = Config::from(&config_file);
        let old = Config::get().await?;
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn merge_file_to_actual() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.merge1.toml")?;
        let new = Config::from(&config_file);
        let old = Config::get().await?;
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn merge_config_chunks() -> Result<()> {
        let old = Config::from(&ConfigFile::from_toml("../examples/jucenit.toml")?);
        let new = Config::from(&ConfigFile::from_toml("../examples/jucenit.merge1.toml")?);
        let res = Config::merge(&old, &new)?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn unmerge_config_chunks() -> Result<()> {
        let old = Config::from(&ConfigFile::from_toml("../examples/jucenit.merge2.toml")?);
        let new = Config::from(&ConfigFile::from_toml("../examples/jucenit.merge1.toml")?);
        let res = Config::unmerge(&old, &new)?;
        println!("{:#?}", res);
        Ok(())
    }
}
