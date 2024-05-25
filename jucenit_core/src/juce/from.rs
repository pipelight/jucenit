use super::{Config as JuceConfig, Unit as JuceUnit, UnitKind};
use crate::cast::{Config as ConfigFile, Unit as ConfigFileUnit};
use crate::nginx::{Config as NginxConfig, ListenerOpts, Route};
use indexmap::IndexMap;

impl From<&ConfigFile> for JuceConfig {
    fn from(config_file: &ConfigFile) -> JuceConfig {
        let mut jucenit_config = JuceConfig::default();

        // Iterate over config file [[unit]] steps
        for e in config_file.unit.clone() {
            // Fill the IndexMap
            jucenit_config.units.insert(
                e.match_,
                JuceUnit {
                    id: e.id,
                    action: e.action,
                    listeners: e.listeners,
                    kind: UnitKind::Managed,
                },
            );
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
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let res = JuceConfig::from(&config_file);
        println!("{:#?}", res);
        Ok(())
    }
}
