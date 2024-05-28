use std::collections::HashMap;

use super::{Config as NginxConfig, ListenerOpts, Route as NginxRoute};
use crate::cast::{Config as ConfigFile, Unit as ConfigFileUnit};
use crate::juce::{Config as JuceConfig, Unit as JuceUnit};

impl From<&JuceConfig> for NginxConfig {
    fn from(e: &JuceConfig) -> NginxConfig {
        // Create jucenit managed nginx-unit routes(steps)
        let mut routes: HashMap<String, Vec<NginxRoute>> = HashMap::new();

        // Create jucenit managed nginx-unit listeners
        let mut listeners: HashMap<String, ListenerOpts> = HashMap::new();

        for unit in e.units.values() {
            for listener in &unit.listeners {
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

        NginxConfig {
            listeners,
            routes,
            ..NginxConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigFile, JuceConfig, NginxConfig};
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn from_jucenit_to_nginx() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file));
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn from_jucenit_to_nginx_json() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file));
        let res = serde_json::to_string_pretty(&res).into_diagnostic()?;
        println!("{}", res);
        Ok(())
    }
}
