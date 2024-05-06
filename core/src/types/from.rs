use std::collections::HashMap;

use super::config::Config as ConfigFile;
use super::unit::{Config as ConfigUnit, ListenerOpts, Route as UnitRoute};

impl From<&ConfigFile> for ConfigUnit {
    fn from(config_file: &ConfigFile) -> Self {
        let mut unit_config = ConfigUnit::default();

        let mut listeners = HashMap::new();
        let mut routes: Vec<UnitRoute> = vec![];

        for e in config_file.unit.clone().unwrap() {
            let route = UnitRoute {
                action: e.action,
                match_: e.match_,
            };
            routes.push(route);
            for listener in e.listeners {
                listeners.insert(
                    listener,
                    ListenerOpts {
                        pass: "routes/jucenit".to_owned(),
                        tls: None,
                    },
                );
            }
        }

        unit_config.listeners = listeners;
        unit_config.routes.insert("jucenit".to_owned(), routes);
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
}
