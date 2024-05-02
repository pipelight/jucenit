// Error handling
use miette::{Error, IntoDiagnostic, Result};

pub struct Config;

impl Config {
    pub fn read() -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod config {
    // Error handling
    use miette::{Error, IntoDiagnostic, Result};

    // Colors and Formatting
    use colored::Colorize;

    #[test]
    /**
     * Try to retrieve an ignore file
     */
    fn read_config_file() -> Result<()> {
        println!("read");
        Ok(())
    }
}
