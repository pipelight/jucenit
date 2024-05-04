// Struct
use cast::types::{Action, Match};
use cast::unit::UnitConfig;

use std::collections::HashMap;
use std::future::Future;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

async fn get() -> Result<()> {
    let resp = reqwest::get("http://127.0.0.1:8080/")
        .await
        .into_diagnostic()?
        .json::<UnitConfig>()
        .await
        .into_diagnostic()?;

    println!("{resp:#?}");

    Ok(())
}

#[cfg(test)]
/**
 * Test loading a file from a given path
 */
mod make_request {
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn read_toml() -> Result<()> {
        get().await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
