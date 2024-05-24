use super::CertificateStore;
use serde::{Deserialize, Serialize};
use std::default::Default;
// Globals
use crate::nginx::SETTINGS;
// Error Handling
use crate::error::JsonError;
use miette::{Error, IntoDiagnostic, Result};

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawCertificate {
    key: String,
    chain: Vec<CertificateInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CertificateInfo {
    subject: Identity,
    issuer: Identity,
    pub validity: Validity,
}
impl CertificateInfo {
    pub fn should_renew(self: &Self) -> Result<()> {
        println!("{:?}", self.validity);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Identity {
    common_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Validity {
    since: String,
    until: String,
}
impl Validity {
    pub fn remaining_time(&self) -> Result<Duration> {
        ComputeValidity::from(self).remaining_time()
    }
    pub fn should_renew(&self) -> Result<bool> {
        ComputeValidity::from(self).should_renew()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComputeValidity {
    since: DateTime<Utc>,
    until: DateTime<Utc>,
}
impl From<&Validity> for ComputeValidity {
    fn from(e: &Validity) -> ComputeValidity {
        ComputeValidity {
            since: NaiveDateTime::parse_from_str(&e.since, "%b %e %T %Y %Z")
                .unwrap()
                .and_utc(),
            until: NaiveDateTime::parse_from_str(&e.until, "%b %e %T %Y %Z")
                .unwrap()
                .and_utc(),
        }
    }
}
impl ComputeValidity {
    pub fn remaining_time(&self) -> Result<Duration> {
        let rest = self.until - Utc::now();
        Ok(rest)
    }
    pub fn should_renew(&self) -> Result<bool> {
        Ok(self.remaining_time()? <= Duration::weeks(3))
    }
}

impl CertificateStore {
    /**
     * Get a certificate from nginx-unit certificate store.
     */
    pub async fn get(dns: &str) -> Result<CertificateInfo> {
        let settings = SETTINGS.lock().unwrap().clone();

        let cert = reqwest::get(settings.get_url() + "/certificates/" + dns + "/chain")
            .await
            .into_diagnostic()?
            .json::<Vec<CertificateInfo>>()
            .await
            .into_diagnostic()?;

        Ok(cert.first().unwrap().to_owned())
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
    use crate::nginx::CertificateStore;
    use crate::ssl;
    use crate::ssl::Fake as FakeCertificate;
    use crate::ssl::Letsencrypt as LetsencryptCertificate;
    use crate::NginxConfig;

    // Error Handling
    use miette::{Error, IntoDiagnostic, Result};

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
