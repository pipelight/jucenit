mod convert;
mod error;
pub mod types;
pub mod unit;

// Structs
use crate::types::Config;
// Filesystem
use std::fs;
use std::path::Path;
use utils::files::FileType;
// Error Handling
use crate::error::{TomlError, YamlError};
use miette::{Error, IntoDiagnostic, Result};

impl Config {
    /**
    Choose the appropriated method to load the config file
    according to the file extension(.ts, .toml, .yml...).

    Arguments:
      - file_path is the config file path
      - args are only to be used with scripting language (typescript) to pass args to the underlying script.

    Languages coming next after v1.0.0:
      - Rust, Hcl, Kcl, Python...
    */
    pub fn load(file_path: &str, args: Option<Vec<String>>) -> Result<Config> {
        let extension = &Path::new(file_path)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let file_type = FileType::from(extension);
        let mut config = match file_type {
            FileType::Toml | FileType::Tml => Config::from_toml(file_path)?,
            FileType::Yaml | FileType::Yml => Config::from_yaml(file_path)?,
            _ => {
                let msg = format!("File type is unknown");
                return Err(Error::msg(msg));
            }
        };
        Ok(config)
    }
    /**
    Returns a Config struct from a provided toml file path.
    */
    pub fn from_toml(file_path: &str) -> Result<Config> {
        let tml = fs::read_to_string(file_path).into_diagnostic()?;
        let res = toml::from_str::<Config>(&tml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = TomlError::new(e, &tml);
                Err(err.into())
            }
        }
    }
    /**
     * Returns a Config struct from a provided yaml file path.
     */
    pub fn from_yaml(file_path: &str) -> Result<Config> {
        let yml = fs::read_to_string(file_path).into_diagnostic()?;
        let res = serde_yaml::from_str::<Config>(&yml);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                let err = YamlError::new(e, &yml);
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
/**
 * Test loading a file from a given path
 */
mod read_config_file {
    use miette::Result;

    use crate::types::Config;

    #[test]
    fn read_toml() -> Result<()> {
        let res = Config::from_toml("../examples/unit.toml")?;
        println!("{:#?}", res);
        Ok(())
    }
}
