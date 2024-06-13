use super::{CertificateInfo, CertificateStore, RawCertificate};
use serde::{Deserialize, Serialize};
use std::default::Default;
// Globals
use crate::nginx::SETTINGS;
// Error Handling
use crate::error::JsonError;
use miette::{Error, IntoDiagnostic, Result};

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use std::collections::HashMap;

impl CertificateStore {
    /**
     * Get a certificate from nginx-unit certificate store.
     */
    pub async fn get(dns: &str) -> Result<CertificateInfo> {
        let settings = SETTINGS.lock().unwrap().clone();

        let mut cert = reqwest::get(settings.get_url() + "/certificates/" + dns + "/chain")
            .await
            .into_diagnostic()?
            .json::<Vec<CertificateInfo>>()
            .await
            .into_diagnostic()?;

        // Get first element
        let message = format!("No certificate in the store for {:?}", dns);
        let err = Error::msg(message);

        cert.reverse();
        cert.pop().ok_or(err)
    }
    /**
     * Get every certificate from nginx-unit certificate store.
     */
    pub async fn get_all() -> Result<HashMap<String, CertificateInfo>> {
        let settings = SETTINGS.lock().unwrap().clone();
        let res = reqwest::get(settings.get_url() + "/certificates")
            .await
            .into_diagnostic()?
            .json::<HashMap<String, RawCertificate>>()
            .await
            .into_diagnostic()?;

        let mut map: HashMap<String, CertificateInfo> = HashMap::new();
        for (k, v) in res.iter() {
            map.insert(k.to_owned(), v.chain.first().unwrap().to_owned());
        }
        Ok(map)
    }
    /**
     * Get every certificate non close to expirity from nginx-unit certificate store.
     */
    pub async fn get_all_valid() -> Result<HashMap<String, CertificateInfo>> {
        let mut res = Self::get_all().await?;
        res.retain(|k, v| !v.validity.should_renew().unwrap());
        Ok(res)
    }
    /**
     * Get every almost expired certificate from nginx-unit certificate store.
     */
    pub async fn get_all_expired() -> Result<HashMap<String, CertificateInfo>> {
        let mut res = Self::get_all().await?;
        res.retain(|k, v| v.validity.should_renew().unwrap());
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::cast::Config as ConfigFile;
    use crate::nginx::CertificateStore;
    use crate::ssl;
    use crate::ssl::Fake as FakeCertificate;
    use crate::ssl::Letsencrypt as LetsencryptCertificate;
    use crate::NginxConfig;
    use std::path::PathBuf;

    // Error Handling
    use miette::{Error, IntoDiagnostic, Result};

    /**
     * Set a fresh testing environment
     */
    async fn set_testing_config() -> Result<()> {
        // Clean config and certificate store
        CertificateStore::clean().await?;
        // JuceConfig::set(&JuceConfig::default()).await?;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");
        let config_file = ConfigFile::load(path.to_str().unwrap())?;

        // let juce_config = JuceConfig::from(&config_file);
        // JuceConfig::set(&juce_config).await?;

        Ok(())
    }

    /**
     * Generate a new certificate and upload it to nginx-unit
     */
    async fn gen_cert_letsencrypt(dns: &str) -> Result<()> {
        // Uncomment to generate a pebble certificate
        // let account = ssl::pebble::pebble_account().await?.clone();
        // let bundle = LetsencryptCertificate::get(dns, &account).await?;
        let bundle = FakeCertificate::get(dns)?;

        let res = CertificateStore::update(dns, &bundle).await?;
        Ok(())
    }

    #[tokio::test]
    async fn get_all_certs() -> Result<()> {
        let res = CertificateStore::get_all().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn get_validity_info() -> Result<()> {
        let dns = "example.com";
        gen_cert_letsencrypt(dns).await?;
        let cert = CertificateStore::get(&dns).await?;

        println!(
            "Certificate remainig vaildity time: {:#?} weeks",
            cert.validity.remaining_time()?.num_weeks()
        );
        let bool = cert.validity.should_renew()?;
        println!("Should be renewed (<=3 weeks)?: {:?}", bool);
        Ok(())
    }
}
