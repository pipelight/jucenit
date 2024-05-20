// File manipulation
use std::fs;
use std::io::Write;
use uuid::Uuid;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Certificate generation

use rcgen::{generate_simple_self_signed, CertifiedKey};

// Global vars
use crate::nginx::SETTINGS;

#[derive(Debug, Clone, Default)]
pub struct Fake;
impl Fake {
    pub fn get(dns: &str) -> Result<String> {
        let names = vec![dns.to_owned()];
        let CertifiedKey { cert, key_pair } =
            generate_simple_self_signed(names).into_diagnostic()?;
        let bundle = format!("{}{}", cert.pem(), key_pair.serialize_pem());
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::Fake;
    use miette::Result;

    #[test]
    fn get_dummy_bundle() -> Result<()> {
        let res = Fake::get("example.com")?;
        println!("{}", res);
        Ok(())
    }
}
