use std::collections::HashMap;

use super::config::{Config as ConfigFile, Unit as Unity};
use super::unit::{Config as ConfigUnit, ListenerOpts, Route as UnitRoute};

impl From<&ConfigUnit> for ConfigFile {
    fn from(unit_config: &ConfigUnit) -> Self {
        let mut config_file = ConfigFile::default();

        for (key, value) in unit_config.listeners.iter() {
            let route_name = format!("jucenit_[{}]", key);
            if let Some(route_vec) = unit_config.routes.get(&route_name) {
                for route in route_vec {
                    let unit = Unity {
                        listeners: vec![key.to_owned()],
                        action: route.action.clone(),
                        match_: route.match_.clone(),
                    };
                    config_file.unit.push(unit)
                }
            }
        }

        // let mut listeners = HashMap::new();
        // for listener in unit_config.listeners {}

        return config_file;
    }
}

impl From<&Unity> for ConfigUnit {
    fn from(e: &Unity) -> Self {
        let mut unit_config = ConfigUnit::default();

        let mut route_vec: Vec<UnitRoute> = vec![];

        let mut listeners = HashMap::new();
        let mut routes: HashMap<String, Vec<UnitRoute>> = HashMap::new();

        for listener in e.listeners.clone() {
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
            let route = UnitRoute {
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
        return unit_config;
    }
}

impl From<&ConfigFile> for ConfigUnit {
    fn from(config_file: &ConfigFile) -> Self {
        let mut unit_config = ConfigUnit::default();

        let mut listeners = HashMap::new();
        let mut routes: HashMap<String, Vec<UnitRoute>> = HashMap::new();

        for e in config_file.unit.clone() {
            for listener in e.listeners {
                let mut route_vec: Vec<UnitRoute> = vec![];
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
                let route = UnitRoute {
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

    use super::ConfigFile;
    use super::ConfigUnit;

    use miette::Result;

    #[test]
    fn get_config() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = ConfigUnit::from(&config_file);
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn adapt_file() -> Result<()> {
        ConfigFile::from_toml("../examples/jucenit.toml")?.adapt()?;
        Ok(())
    }

    #[tokio::test]
    async fn get_unit_config_to_toml() -> Result<()> {
        let res = ConfigFile::from(&(ConfigUnit::get().await?));
        println!("{:#?}", res);
        Ok(())
    }
}
