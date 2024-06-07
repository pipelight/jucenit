//!
//! This is where all the magic happen.
//! The easy to use JuceConfig structs get translated to
//! nginx internals.
//! Beware: Things are a bit complex here. Read the comments.
//!
use miette::Result;
use std::collections::HashMap;

use crate::cast::{Config as ConfigFile, Unit as ConfigFileUnit};
use crate::juce::{Config as JuceConfig, Unit as JuceUnit, UnitKind};
use crate::mapping::{ListenerOpts, Route as NginxRoute, Tls};
use crate::nginx::{CertificateStore, Config as NginxConfig};

// impl From<&JuceConfig> for NginxConfig {
impl NginxConfig {
    pub async fn from(e: &JuceConfig) -> Result<NginxConfig> {
        // Create jucenit managed nginx-unit routes(steps)
        let mut routes: HashMap<String, Vec<NginxRoute>> = HashMap::new();

        // Create jucenit nginx-unit listeners
        let mut listeners: HashMap<String, ListenerOpts> = HashMap::new();
        let mut listeners_to_host_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut listeners_to_kind_map: HashMap<String, UnitKind> = HashMap::new();

        for (match_, unit) in e.units.iter() {
            for listener in &unit.listeners {
                // Create a map that
                // group certificates with their corresponding listeners
                if let Some(dns) = match_.host.clone() {
                    if CertificateStore::get(&dns).await.is_ok() {
                        match listeners_to_host_map.get_mut(listener) {
                            None => {
                                listeners_to_host_map.insert(listener.to_owned(), vec![dns]);
                            }
                            Some(val) => {
                                val.push(dns);
                            }
                        };
                    }
                }
                // Create a map that attribute a UnitKind to listeners
                // If the listener redirects to a route that must serve
                // an http-01 challenge then tls is disabled for this listener
                // (until challenge is removed).
                match listeners_to_kind_map.get_mut(listener) {
                    None => {
                        listeners_to_kind_map.insert(listener.to_owned(), unit.kind.clone());
                    }
                    Some(val) => {
                        if val == &UnitKind::default() {
                            listeners_to_kind_map.insert(listener.to_owned(), unit.kind.clone());
                        }
                    }
                };

                // Set route names with empty (to be provisionned) routing table.
                let route_name = format!("jucenit_[{}]", listener);
                routes.insert(route_name.clone(), vec![]);

                // Set listener names with empty (to be provisionned) routing table.
                listeners.insert(
                    listener.to_owned(),
                    ListenerOpts {
                        pass: format!("routes/{}", &route_name),
                        tls: None,
                    },
                );
            }
        }

        // Provision routes with values
        for (match_, unit) in &e.units {
            for listener in &unit.listeners {
                let route_name = format!("jucenit_[{}]", listener);
                routes.get_mut(&route_name).unwrap().push(NginxRoute {
                    match_: match_.to_owned(),
                    action: unit.action.clone(),
                });
            }
        }

        // Add provisionned tls option to listeners
        for (k, v) in listeners_to_host_map {
            if let Some(kind) = listeners_to_kind_map.get_mut(&k) {
                if kind == &UnitKind::HttpChallenge {
                    break;
                }
            }
            if let Some(opts) = listeners.get_mut(&k) {
                opts.tls = Some(Tls {
                    certificate: v.to_owned(),
                });
            }
        }

        Ok(NginxConfig {
            listeners,
            routes,
            ..NginxConfig::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigFile, JuceConfig, NginxConfig};
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn from_jucenit_to_nginx() -> Result<()> {
        let config_file = ConfigFile::load("../examples/jucenit.full.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file)).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn from_jucenit_to_nginx_json() -> Result<()> {
        let config_file = ConfigFile::load("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file)).await?;
        let res = serde_json::to_string_pretty(&res).into_diagnostic()?;
        println!("{}", res);
        Ok(())
    }
}
