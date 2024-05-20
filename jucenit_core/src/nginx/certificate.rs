// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Ssl utils
use crate::ssl;
use std::collections::HashMap;

//Globals
use super::SETTINGS;
use super::{Config, Tls};

#[derive(Debug, Clone, Default)]

pub struct CertificateStore;
impl CertificateStore {
    // pub async fn update(dns: &str, bundle: &Vec<u8>) -> Result<serde_json::Value> {
    //     Self::remove(dns).await?;
    //     let res = Self::add(dns, bundle).await?;
    //     Ok(res)
    // }
    // pub async fn add(dns: &str, bundle: &Vec<u8>) -> Result<serde_json::Value> {
    //     // Upload certificate to nginx-unit certificate store
    //     let settings = SETTINGS.lock().unwrap().clone();
    //     let client = reqwest::Client::new();
    //     let res = client
    //         .put(settings.get_url() + "/certificates/" + dns)
    //         .body(bundle.to_owned())
    //         .send()
    //         .await
    //         .into_diagnostic()?
    //         .json::<serde_json::Value>()
    //         .await
    //         .into_diagnostic()?;
    //     Ok(res)
    // }
    pub async fn update(dns: &str, bundle: &str) -> Result<serde_json::Value> {
        Self::remove(dns).await?;
        let res = Self::add(dns, bundle).await?;
        Ok(res)
    }
    pub async fn add(dns: &str, bundle: &str) -> Result<serde_json::Value> {
        // Upload certificate to nginx-unit certificate store
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .put(settings.get_url() + "/certificates/" + dns)
            .body(bundle.to_owned())
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;
        Ok(res)
    }
    /**
     * Bulk update listener with every certificates in the store
     */
    async fn update_listeners() -> Result<serde_json::Value> {
        let certificates = Self::get_all().await?;
        let dns_list: Vec<String> = certificates.into_keys().collect();

        let mut config = Config::get().await?;
        for (_, val) in config.listeners.iter_mut() {
            val.tls = Some(Tls {
                certificate: dns_list.clone(),
            });
        }
        let res = Config::set(&config).await?;

        Ok(res)
    }
    // Remove certificates from nginx-unit certificate store
    pub async fn remove(dns: &str) -> Result<serde_json::Value> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = client
            .delete(settings.get_url() + "/certificates/" + &dns)
            .send()
            .await
            .into_diagnostic()?
            .json::<serde_json::Value>()
            .await
            .into_diagnostic()?;
        Ok(res)
    }
    pub async fn get_all() -> Result<HashMap<String, serde_json::Value>> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = reqwest::get(settings.get_url() + "/certificates")
            .await
            .into_diagnostic()?
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .into_diagnostic()?;
        Ok(res)
    }
    pub async fn get() -> Result<HashMap<String, serde_json::Value>> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();
        let res = reqwest::get(settings.get_url() + "/certificates")
            .await
            .into_diagnostic()?
            .json::<HashMap<String, serde_json::Value>>()
            .await
            .into_diagnostic()?;
        Ok(res)
    }
    pub async fn remove_all() -> Result<()> {
        let settings = SETTINGS.lock().unwrap().clone();
        let client = reqwest::Client::new();

        // Get list of certificates in nginx-unit store
        let certificates = Self::get_all().await?;

        for (key, _) in certificates {
            Self::remove(&key).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::CertificateStore;
    use crate::ssl;
    use crate::ssl::Fake as FakeCertificate;
    use crate::ssl::Letsencrypt as LetsencryptCertificate;
    use miette::Result;

    #[tokio::test]
    async fn update_cert_fake() -> Result<()> {
        let dns = "example.com";
        let res = CertificateStore::update(dns, &FakeCertificate::get(dns)?).await?;
        Ok(())
    }
    #[tokio::test]
    async fn update_cert_letsencrypt() -> Result<()> {
        let dns = "crocuda.com";
        let account = ssl::pebble::pebble_account().await?.clone();
        let res = CertificateStore::update(dns, &LetsencryptCertificate::get(dns, &account).await?)
            .await?;
        Ok(())
    }
    #[tokio::test]
    async fn remove_cert() -> Result<()> {
        let res = CertificateStore::remove("example.com").await?;
        println!("{:#?}", res);
        Ok(())
    }
}
