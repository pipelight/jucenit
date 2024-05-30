// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Ssl utils
use crate::ssl;
use crate::ssl::Letsencrypt as LetsencryptCertificate;
use std::collections::HashMap;

// Globals
use crate::juce::Config as JuceConfig;
use crate::mapping::Tls;
use crate::nginx::Config as NginxConfig;
use crate::nginx::SETTINGS;

// Struct
use super::CertificateInfo;

#[derive(Debug, Clone, Default)]
pub struct CertificateStore;
impl CertificateStore {
    /**
     * Poll the configuration for hosts and seek through certificate store
     * for matching valid certificates or generate them.
     * Update the configuration with fresh ssl.
     */
    pub async fn hydrate() -> Result<()> {
        let account = ssl::pebble_account().await?.clone();
        // let account = ssl::letsencrypt_account().await?.clone();
        for host in JuceConfig::get_hosts().await? {
            let dns = host;
            // For ACME limitation rate reason
            // Check if a certificate already exists
            let cert = CertificateStore::get(&dns).await;
            match cert {
                Ok(res) => {
                    if res.validity.should_renew()? {
                        let bundle =
                            LetsencryptCertificate::get_cert_bundle(&dns, &account).await?;
                        CertificateStore::update(&dns, &bundle).await?;
                    }
                }
                Err(_) => {
                    let bundle = LetsencryptCertificate::get_cert_bundle(&dns, &account).await?;
                    CertificateStore::update(&dns, &bundle).await?;
                }
            };
        }
        JuceConfig::push(&JuceConfig::pull().await?).await?;
        Ok(())
    }
    /**
     * Upload a certificate bundle:
     *  - a .pem file
     *   with intermediate certs and private key)
     * to nginx-unit certificate store
     */
    async fn add(dns: &str, bundle: &str) -> Result<serde_json::Value> {
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
     * Remove a certificate from nginx-unit certificate store.
     */
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
    /**
     * Remove every certificate from nginx-unit certificate store.
     */
    pub async fn clean() -> Result<()> {
        // Get list of every certificates in nginx-unit certificate store.
        let certificates = CertificateStore::get_all().await?;
        for (key, _) in certificates {
            CertificateStore::remove(&key).await?;
        }
        Ok(())
    }
    /**
     * Replace a certificate bundle:
     *  - a .pem file
     *   with intermediate certs and private key)
     * to nginx-unit certificate store
     */
    pub async fn update(dns: &str, bundle: &str) -> Result<serde_json::Value> {
        // Remove preceding certificate if it exists
        let _ = CertificateStore::remove(dns).await;
        let res = CertificateStore::add(dns, bundle).await?;
        Ok(res)
    }
    // /**
    //  * Bulk update listeners with every certificates in the store
    //  */
    // pub async fn update_listeners() -> Result<()> {
    //     let certificates = JuceConfig::get_hosts().await?;
    //     // let certificates = CertificateStore::get_all().await?;
    //     // let dns_list: Vec<String> = certificates.into_keys().collect();
    //
    //     let mut config = NginxConfig::get().await?;
    //     for (_, val) in config.listeners.iter_mut() {
    //         val.tls = Some(Tls {
    //             // certificate: dns_list.clone(),
    //             certificate: certificates.clone(),
    //         });
    //     }
    //     println!("{:#?}", &config);
    //     let res = NginxConfig::set(&config).await?;
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {

    use super::CertificateStore;
    use crate::ssl;
    use crate::ssl::Fake as FakeCertificate;
    use crate::ssl::Letsencrypt as LetsencryptCertificate;
    use miette::Result;

    use crate::ConfigFile;
    use crate::JuceConfig;
    use crate::NginxConfig;

    /**
     * Set a fresh testing environment
     */
    async fn set_testing_config() -> Result<()> {
        // Clean config and certificate store
        CertificateStore::clean().await?;
        JuceConfig::set(&JuceConfig::default()).await?;

        // Set new configuration
        let config_file = ConfigFile::from_toml("../examples/jucenit.toml")?;
        let juce_config = JuceConfig::from(&config_file);
        JuceConfig::set(&juce_config).await?;

        Ok(())
    }

    // #[tokio::test]
    async fn clean_cert_store() -> Result<()> {
        let res = CertificateStore::clean().await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn remove_cert() -> Result<()> {
        let dns = "example.com";
        let res = CertificateStore::remove(dns).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn add_fake_cert() -> Result<()> {
        let dns = "example.com";
        let bundle = FakeCertificate::get(dns)?;
        let res = CertificateStore::add(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn update_fake_cert() -> Result<()> {
        let dns = "example.com";
        let bundle = FakeCertificate::get(dns)?;
        let res = CertificateStore::update(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn update_cert_letsencrypt() -> Result<()> {
        let dns = "example.com";
        let account = ssl::pebble::pebble_account().await?.clone();
        let bundle = LetsencryptCertificate::get_cert_bundle(dns, &account).await?;
        let res = CertificateStore::update(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn hydrate_cert_store() -> Result<()> {
        set_testing_config().await?;

        let res = CertificateStore::hydrate().await?;

        let certificates = CertificateStore::get_all().await?;
        let mut dns_list: Vec<String> = certificates.into_keys().collect();
        dns_list.sort();
        let mut expected = vec!["example.com".to_owned(), "test.com".to_owned()];
        expected.sort();

        assert_eq!(expected, dns_list);
        Ok(())
    }
}
