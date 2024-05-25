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
        e.units.values().map(|unit| {
            unit.listeners.into_iter().map(|listener| {
                let route_name = format!("jucenit_[{}]", listener);
                // Provision routes with keys
                routes.insert(route_name, vec![]);
                listeners.insert(
                    listener,
                    ListenerOpts {
                        pass: "routes/".to_owned() + &route_name,
                        tls: None,
                    },
                )
            })
        });

        // Provision routes with values
        e.units.iter().map(|(match_, unit)| {
            for listener in unit.listeners {
                let route_name = format!("jucenit_[{}]", listener);
                routes.get_mut(&route_name).unwrap().push(NginxRoute {
                    match_: match_.to_owned(),
                    action: unit.action,
                });
            }
        });

        NginxConfig { listeners, routes }
    }
}

#[cfg(test)]
mod tests {

    use super::{ConfigFile, JuceConfig, NginxConfig};

    use miette::Result;

    #[test]
    fn get_config() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = NginxConfig::from(&JuceConfig::from(&config_file));
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn adapt_file() -> Result<()> {
        ConfigFile::from_toml("../examples/jucenit.toml")?.adapt()?;
        Ok(())
    }
}
