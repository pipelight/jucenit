// Structs
use crate::types::Config;
// Filesystem - read file
// Filesystem
use std::fs;
use std::path::Path;
use utils::files::FileType;
// Error Handling
use crate::error::{TomlError, YamlError};
use miette::{Error, IntoDiagnostic, Result};

impl Config {
    pub fn ensure_listener(config: Config) -> Result<()> {
        Ok(())
    }
    /**
     * Flatten the configuration structure to mutliple unit queries
     */
    pub fn to_unit(config: Config) -> Result<()> {
        Ok(())
    }

    pub fn to_json(config: Config) -> Result<()> {
        Ok(())
    }
}
