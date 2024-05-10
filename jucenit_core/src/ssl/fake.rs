// File manipulation
use std::fs;
use std::io::Write;
use uuid::Uuid;
// Crate structs
use crate::{Action, ConfigFile, ConfigUnit, Match, Unity};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

pub fn make_dummy_cert(dns: &str) -> Result<()> {

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::make_dummy_cert;
    use miette::Result;

    // #[tokio::test]
    async fn get_dummy_cert() -> Result<()> {
        make_dummy_cert("crocuda.com")?;
        // println!("{:#?}", res);
        Ok(())
    }
}
