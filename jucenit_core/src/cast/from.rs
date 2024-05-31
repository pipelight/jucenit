use super::Config as ConfigFile;
use crate::juce::Config as JuceConfig;
use crate::mapping::{ListenerOpts, Route};

impl From<&JuceConfig> for ConfigFile {
    fn from(juce_config: &JuceConfig) -> ConfigFile {
        let juce_config = juce_config.clone();

        let config_file = ConfigFile::default();
        juce_config.units;
        config_file
    }
}
