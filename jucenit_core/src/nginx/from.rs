use miette::Result;
use std::collections::HashMap;

use super::{CertificateStore, Config as NginxConfig};
use crate::cast::{Config as ConfigFile, Unit as ConfigFileUnit};
use crate::juce::{Config as JuceConfig, Unit as JuceUnit};
use crate::mapping::{ListenerOpts, Route as NginxRoute, Tls};

// impl From<&JuceConfig> for NginxConfig {
impl NginxConfig {
    pub async fn from(e: &JuceConfig) -> Result<NginxConfig> {
        // Create jucenit managed nginx-unit routes(steps)
        let mut routes: HashMap<String, Vec<NginxRoute>> = HashMap::new();

        // Create jucenit managed nginx-unit listeners
        let mut listeners: HashMap<String, ListenerOpts> = HashMap::new();
        let mut hosts_by_listeners: HashMap<String, Vec<String>> = HashMap::new();

        for (match_, unit) in e.units.iter() {
            for listener in &unit.listeners {
                // Add certificates to their corresponding listeners
                if let Some(dns) = match_.host.clone() {
                    if CertificateStore::get(&dns).await.is_ok() {
                        match hosts_by_listeners.get_mut(listener) {
                            None => {
                                hosts_by_listeners.insert(listener.to_owned(), vec![dns]);
                            }
                            Some(val) => {
                                val.push(dns);
                            }
                        };
                    }
                }

                let route_name = format!("jucenit_[{}]", listener);
                // Provision routes with keys
                routes.insert(route_name.clone(), vec![]);
                listeners.insert(
                    listener.to_owned(),
                    ListenerOpts {
                        pass: format!("routes/{}", &route_name),
                        tls: None,
                    },
                );
            }
        }

        // Add provisionned tls option to listeners
        for (k, v) in hosts_by_listeners {
            if let Some(opts) = listeners.get_mut(&k) {
                opts.tls = Some(Tls {
                    certificate: v.to_owned(),
                });
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
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file)).await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn from_jucenit_to_nginx_json() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file)).await?;
        let res = serde_json::to_string_pretty(&res).into_diagnostic()?;
        println!("{}", res);
        Ok(())
    }
}
