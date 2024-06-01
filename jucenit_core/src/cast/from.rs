use indexmap::IndexMap;

use super::{Config as ConfigFile, Match, MultiMatch, Unit as ConfigUnit};
use crate::juce::{Config as JuceConfig, Unit as JuceUnit};
use crate::mapping::{ListenerOpts, Route};

impl From<&JuceConfig> for ConfigFile {
    fn from(juce_config: &JuceConfig) -> ConfigFile {
        let juce_config = juce_config.clone();

        // Loop through JuceUnits and set Unit(actions) as key and Match as value
        // This groups matchs(value) leading to the same action(key).
        let mut units_by_action: IndexMap<JuceUnit, Vec<Match>> = IndexMap::new();
        for (match_, unit) in juce_config.units {
            match units_by_action.get_mut(&unit) {
                None => {
                    units_by_action.insert(unit, vec![match_]);
                }
                Some(val) => {
                    val.push(match_);
                }
            };
        }
        // Flatten and Group match
        let mut config_file = ConfigFile::default();
        for (unit, matches) in units_by_action {
            config_file.unit.push(ConfigUnit {
                id: unit.id,
                action: unit.action,
                match_: MultiMatch::from(&matches),
                listeners: unit.listeners,
            });
        }

        config_file
    }
}

impl From<&Vec<Match>> for MultiMatch {
    fn from(matches: &Vec<Match>) -> MultiMatch {
        let multi_match = MultiMatch {
            hosts: matches.iter().map(|e| e.clone().host).collect(),
            uri: matches.first().unwrap().clone().uri,
            source: matches.first().unwrap().clone().source,
        };
        multi_match
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigFile, JuceConfig};
    use miette::Result;

    // Provide a default config
    async fn set_global_config() -> Result<()> {
        let config_file = ConfigFile::from_toml("../examples/jucenit.test_reverse_cast.toml")?;
        let res = JuceConfig::from(&config_file);
        JuceConfig::set(&res).await?;
        Ok(())
    }

    #[tokio::test]
    async fn from_juce_lock_to_config_file_tml() -> Result<()> {
        set_global_config().await?;
        let lock = JuceConfig::pull().await?;
        let res = ConfigFile::to_toml(&ConfigFile::from(&lock))?;
        println!("{}", res);
        Ok(())
    }
    #[tokio::test]
    async fn from_juce_lock_to_config_file_yml() -> Result<()> {
        set_global_config().await?;
        let lock = JuceConfig::pull().await?;
        let res = ConfigFile::to_yaml(&ConfigFile::from(&lock))?;
        println!("{}", res);
        Ok(())
    }
}
