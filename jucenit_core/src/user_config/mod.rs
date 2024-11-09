use config::{Config, ConfigBuilder, File, FileFormat};
use std::path::PathBuf;
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

fn get_user_config() -> Result<()> {
    #[cfg(debug_assertions)]
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../config.toml");
    let path = path.display().to_string();

    #[cfg(not(debug_assertions))]
    let path = "/etc/jucenit/config.toml";

    let mut builder = Config::builder()
        .set_default("default", "1")
        .into_diagnostic()?
        .add_source(File::new(&path, FileFormat::Toml))
        //  .add_async_source(...)
        .set_override("override", "1")
        .into_diagnostic()?;

    match builder.build() {
        Ok(config) => {
            println!("{:#?}", config);
            // use your config
        }
        Err(e) => {
            println!("{:#?}", e);
            // something went wrong
        }
    }
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default() -> Result<()> {
        get_user_config()?;
        Ok(())
    }
}
