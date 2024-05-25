use indexmap::IndexMap;
use std::collections::HashMap;

use super::Config as JuceConfig;
use crate::cast::{Config as ConfigFile, Unit as ConfigFileUnit};
use crate::nginx::{Config as NginxConfig, ListenerOpts, Route};

impl From<&ConfigFile> for JuceConfig {
    fn from(config_file: &ConfigFile) -> Self {
        let mut jucenit_config = JuceConfig::default();

        let mut listeners = HashMap::new();

        let mut routes: HashMap<String, Vec<Route>> = HashMap::new();

        for e in config_file.unit.clone() {
            for listener in e.listeners {
                let mut route_vec: Vec<Route> = vec![];
                // add listeners to unit
                let route_name = format!("jucenit_[{}]", listener);
                listeners.insert(
                    listener,
                    ListenerOpts {
                        pass: "routes/".to_owned() + &route_name,
                        tls: None,
                    },
                );
                // add named route
                let route = Route {
                    action: e.action.clone(),
                    match_: e.match_.clone(),
                };
                route_vec.push(route);

                // insert or update unit route
                if unit_config.routes.get(&route_name).is_some() {
                    unit_config
                        .routes
                        .get_mut(&route_name)
                        .unwrap()
                        .extend(route_vec.clone());
                } else {
                    unit_config.routes.insert(route_name, route_vec.clone());
                }
            }
        }

        unit_config.listeners = listeners;
        return unit_config;
    }
}
#[cfg(test)]
mod tests {
    use super::JuceConfig;
    use crate::cast::Config as ConfigFile;
    use miette::Result;

    #[test]
    fn convert_config_file() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = JuceConfig::from(&config_file);
        println!("{:#?}", res);
        Ok(())
    }
}
