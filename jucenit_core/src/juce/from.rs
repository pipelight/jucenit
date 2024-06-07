use super::{Config as JuceConfig, Unit as JuceUnit, UnitKind};
use crate::cast::{Config as ConfigFile, Match, Unit as ConfigFileUnit};
use crate::mapping::{ListenerOpts, Route};
use indexmap::IndexMap;

impl From<&ConfigFile> for JuceConfig {
    fn from(config_file: &ConfigFile) -> JuceConfig {
        let mut jucenit_config = JuceConfig::default();

        // Iterate over config file [[unit]] steps
        for e in config_file.unit.clone() {
            // Fill the IndexMap
            if let Some(hosts) = e.match_.hosts {
                for host in hosts {
                    jucenit_config.units.insert(
                        Match {
                            host: Some(host),
                            uri: e.match_.uri.clone(),
                            source: e.match_.source.clone(),
                        },
                        JuceUnit {
                            id: e.id.clone(),
                            action: e.action.clone(),
                            listeners: e.listeners.clone(),
                            kind: UnitKind::Managed,
                        },
                    );
                }
            }
        }
        jucenit_config
    }
}

#[cfg(test)]
mod tests {
    use super::JuceConfig;
    use crate::cast::Config as ConfigFile;
    use miette::Result;

    #[test]
    fn convert_config_file() -> Result<()> {
        let config_file = ConfigFile::load("../examples/jucenit.toml")?;
        let res = JuceConfig::from(&config_file);
        println!("{:#?}", res);
        Ok(())
    }
}
