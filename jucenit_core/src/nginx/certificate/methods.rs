use futures::future::join_all;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Ssl utils
use crate::ssl;
use crate::ssl::Letsencrypt as LetsencryptCertificate;
use crate::ConfigFile;
use rayon::prelude::*;
use std::collections::HashMap;

// Globals
use crate::nginx::Config as NginxConfig;
use crate::nginx::SETTINGS;

use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Loop
use std::thread::sleep;
use std::time::{Duration, *};

// Struct
use super::CertificateInfo;

#[derive(Debug, Clone, Default)]
pub struct CertificateStore;
impl CertificateStore {
    /**
     * Poll the configuration for hosts and seek through certificate store
     * for matching valid certificates or generate them,
     * and update nginx-unit configuration with fresh ssl.
     */
    pub async fn hydrate() -> Result<()> {
        let db = connect_db().await?;
        let hosts = Host::find().all(&db).await.into_diagnostic()?;
        let domain: Vec<String> = hosts.iter().map(|x| x.domain.clone()).collect();

        let parallel = domain.iter().map(Self::hydrate_one);

        join_all(parallel).await;

        // Update listeners tls option with fresh certs
        // By updating whole config
        let config = ConfigFile::pull().await?;
        config.push().await?;

        Ok(())
    }

    async fn hydrate_one(host: &String) -> Result<()> {
        #[cfg(debug_assertions)]
        let account = ssl::pebble_account().await?.clone();
        #[cfg(not(debug_assertions))]
        let account = ssl::letsencrypt_account().await?.clone();

        let dns = host.to_owned();
        // For ACME limitation rate reason
        // Check if a certificate already exists
        let cert = CertificateStore::get(&dns).await;
        match cert {
            Ok(res) => {
                if res.validity.should_renew()? {
                    let bundle = LetsencryptCertificate::get_cert_bundle(&dns, &account).await?;
                    CertificateStore::update(&dns, &bundle).await.unwrap();
                }
            }
            Err(_) => {
                let bundle = LetsencryptCertificate::get_cert_bundle(&dns, &account)
                    .await
                    .unwrap();
                CertificateStore::update(&dns, &bundle).await?;
            }
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
    /**
     * Poll certificate store and declared hosts every minutes for changes.
     */
    pub async fn watch() -> Result<()> {
        loop {
            CertificateStore::hydrate().await?;
            sleep(Duration::from_secs(60));
        }
    }
    /**
     * Remove every certificate from nginx-unit certificate store.
     * and update nginx-unit configuration
     */
    pub async fn clean() -> Result<()> {
        // Get list of every certificates in nginx-unit certificate store.
        let certificates = CertificateStore::get_all().await?;
        for (key, _) in certificates {
            CertificateStore::remove(&key).await?;
        }
        // Update routes
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
    async fn remove(dns: &str) -> Result<serde_json::Value> {
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
}

#[cfg(test)]
mod tests {

    use super::CertificateStore;
    use crate::ssl;
    use crate::ssl::Fake as FakeCertificate;
    use crate::ssl::Letsencrypt as LetsencryptCertificate;
    use std::path::PathBuf;

    use miette::Result;

    use crate::ConfigFile;
    use crate::NginxConfig;

    use serial_test::serial;

    /**
     * Set a fresh testing environment:
     * - clean certificate store
     * - set minimal nginx configuration
     */
    async fn set_testing_config() -> Result<()> {
        CertificateStore::clean().await?;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config = ConfigFile::load(path.to_str().unwrap())?;
        config.set().await?;

        Ok(())
    }

    // #[tokio::test]
    // #[serial]
    async fn clean_cert_store() -> Result<()> {
        let res = CertificateStore::clean().await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    // #[serial]
    async fn remove_cert() -> Result<()> {
        let dns = "example.com";
        let res = CertificateStore::remove(dns).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    // #[serial]
    async fn add_fake_cert() -> Result<()> {
        set_testing_config().await?;
        let dns = "example.com";
        let bundle = FakeCertificate::get(dns)?;
        let res = CertificateStore::add(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    // #[serial]
    async fn update_fake_cert() -> Result<()> {
        set_testing_config().await?;
        let dns = "example.com";
        let bundle = FakeCertificate::get(dns)?;
        let res = CertificateStore::update(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    // #[serial]
    async fn update_cert_letsencrypt() -> Result<()> {
        set_testing_config().await?;
        let dns = "example.com";
        let account = ssl::pebble::pebble_account().await?.clone();
        let bundle = LetsencryptCertificate::get_cert_bundle(dns, &account).await?;
        let res = CertificateStore::update(dns, &bundle).await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn hydrate_cert_store() -> Result<()> {
        set_testing_config().await?;

        let res = CertificateStore::hydrate().await?;

        let certificates = CertificateStore::get_all().await?;
        let mut dns_list: Vec<String> = certificates.into_keys().collect();
        dns_list.sort();
        let mut expected = vec![
            "api.example.com".to_owned(),
            "example.com".to_owned(),
            "test.com".to_owned(),
        ];
        expected.sort();

        assert_eq!(expected, dns_list);
        Ok(())
    }
}
