use acme2::gen_rsa_private_key;
use acme2::{
    Account, AccountBuilder, AuthorizationStatus, Challenge, ChallengeStatus, Csr, Directory,
    DirectoryBuilder, OrderBuilder, OrderStatus,
};
use std::time::Duration;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// use acme2::Error;
// Global vars
use crate::nginx::SETTINGS;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// File manipulation
use std::fs;
use std::io::Write;
use uuid::Uuid;
// Crate structs
use super::pebble::{pebble_account, pebble_http_client, PEBBLE_CERT_URL};
use crate::cast::{Action, Config as ConfigFile, Match, Unit as ConfigFileUnit};
use crate::nginx::Config as NginxConfig;
use openssl::x509::X509;

// Production
// const LETS_ENCRYPT_URL: &'static str = "https://acme-v02.api.letsencrypt.org/directory";
// Stagging
pub static LETS_ENCRYPT_URL: Lazy<Arc<Mutex<String>>> = Lazy::new(|| {
    Arc::new(Mutex::new(
        "https://acme-staging-v02.api.letsencrypt.org/directory".to_owned(),
    ))
});
/**
* Create an ACME account to use for the order. For production
* purposes, you should keep the account (and private key), so
* you can renew your certificate easily.
*/
async fn letsencrypt_account() -> Result<Arc<Account>> {
    // Create a new ACMEv2 directory for Let's Encrypt.
    let dir = DirectoryBuilder::new(LETS_ENCRYPT_URL.lock().unwrap().clone())
        .build()
        .await
        .into_diagnostic()?;

    let mut builder = AccountBuilder::new(dir.clone());
    let account = builder
        .contact(vec!["mailto:areskul@areskul.com".to_string()])
        .terms_of_service_agreed(true)
        .build()
        .await
        .into_diagnostic()?;
    Ok(account)
}

/**
* Create an nginx-unit match to serve the challenge key file.
* A file at `https://example.com/.well-known/${challenge.token}`
* with the content of `challenge.key_authorization()??`.
*/
fn make_challenge_config(dns: &str, challenge: &Challenge) -> Result<ConfigFileUnit> {
    // Update nginx-unit config
    let unit = ConfigFileUnit {
        listeners: vec!["*:80".to_owned(), "*:443".to_owned()],
        match_: Match {
            uri: Some(format!("/.well-known/{}", challenge.token.clone().unwrap())),
            host: Some(dns.to_owned()),
            ..Match::default()
        },
        action: Some(Action {
            share: Some(vec![format!("/tmp/jucenit/challenge_{}.txt", dns)]),
            ..Action::default()
        }),
    };
    Ok(unit)
    // println!("{:?}", res);
}

/**
* Create tmp challenge files and nginx-unit routes
*/
async fn set_challenge_key(dns: &str, challenge: &Challenge) -> Result<()> {
    // Write challenge key to temporary file
    let data = challenge.key_authorization().into_diagnostic()?.unwrap();

    // Create and write to file
    let tmp_dir = "/tmp/jucenit";
    fs::create_dir_all(tmp_dir).into_diagnostic()?;
    let path = format!("/tmp/jucenit/challenge_{}.txt", dns);
    let mut file = fs::File::create(path.clone()).into_diagnostic()?;
    let bytes = data.as_bytes();
    file.write_all(bytes).into_diagnostic()?;

    // Update nginx conf
    let unit = make_challenge_config(dns, challenge)?;
    let config = NginxConfig::from(&unit);
    NginxConfig::update(&config).await?;
    Ok(())
}
/**
* Delete tmp challenge files and nginx-unit routes
*/
async fn del_challenge_key(dns: &str, challenge: &Challenge) -> Result<()> {
    let tmp_dir = "/tmp/jucenit";
    let path = format!("{}/challenge_{}.txt", tmp_dir, dns);
    fs::remove_file(path).into_diagnostic()?;

    // Update nginx conf
    let unit = make_challenge_config(dns, challenge)?;
    let config = NginxConfig::from(&unit);
    NginxConfig::delete(&config).await?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct Letsencrypt;

impl Letsencrypt {
    pub async fn get(dns: &str, account: &Account) -> Result<String> {
        let account = pebble_account().await?;
        // let account = letsencrypt_account().await?;

        // Create a new order for a specific domain name.
        let mut builder = OrderBuilder::new(account);
        builder.add_dns_identifier(dns.to_owned());
        let order = builder.build().await.into_diagnostic()?;

        // Get the list of needed authorizations for this order.
        let authorizations = order.authorizations().await.into_diagnostic()?;
        for auth in authorizations {
            // Get an http-01 challenge for this authorization
            if let Some(challenge) = auth.get_challenge("http-01") {
                set_challenge_key(dns, &challenge).await?;
                let challenge = challenge.validate().await.into_diagnostic()?;
                let challenge = challenge
                    .wait_done(Duration::from_secs(5), 3)
                    .await
                    .into_diagnostic()?;
                assert_eq!(challenge.status, ChallengeStatus::Valid);
                del_challenge_key(dns, &challenge).await?;

                let authorization = auth
                    .wait_done(Duration::from_secs(5), 3)
                    .await
                    .into_diagnostic()?;
                assert_eq!(authorization.status, AuthorizationStatus::Valid);
            }
        }

        // Poll the order every 5 seconds until it is in either the
        // `ready` or `invalid` state. Ready means that it is now ready
        // for finalization (certificate creation).
        let order = order
            .wait_ready(Duration::from_secs(5), 3)
            .await
            .into_diagnostic()?;

        assert_eq!(order.status, OrderStatus::Ready);

        // Generate an RSA private key for the certificate.
        let pkey = gen_rsa_private_key(4096).into_diagnostic()?;
        let private_key = pkey.private_key_to_pem_pkcs8().into_diagnostic()?;
        let private_key = String::from_utf8(private_key).into_diagnostic()?;

        // Create a certificate signing request for the order, and request
        // the certificate.
        let order = order
            .finalize(Csr::Automatic(pkey))
            .await
            .into_diagnostic()?;

        // Poll the order every 5 seconds until it is in either the
        // `valid` or `invalid` state. Valid means that the certificate
        // has been provisioned, and is now ready for download.
        let order = order
            .wait_done(Duration::from_secs(5), 3)
            .await
            .into_diagnostic()?;

        assert_eq!(order.status, OrderStatus::Valid);

        // Download the certificate, and panic if it doesn't exist.
        let certificates = order.certificate().await.into_diagnostic()?.unwrap();
        assert!(certificates.len() > 1);

        let mut bundle: String = "".to_owned();
        for cert in certificates.clone() {
            let cert = cert.to_pem().into_diagnostic()?;
            let cert = String::from_utf8(cert).into_diagnostic()?;
            bundle += &cert;
        }
        bundle += &private_key;

        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::Letsencrypt;
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn set_local_creds() -> Result<()> {
        let res = pebble_account().await?;
        Ok(())
    }
    // #[tokio::test]
    async fn set_remote_creds() -> Result<()> {
        let res = letsencrypt_account().await?;
        // println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn get_pebble_cert() -> Result<()> {
        let account = letsencrypt_account().await?.clone();
        let res = Letsencrypt::get("crocuda.com", &account).await?;
        println!("{:#?}", res);
        Ok(())
    }
    // #[tokio::test]
    async fn get_letsencrypt_cert() -> Result<()> {
        let account = pebble_account().await?.clone();
        let res = Letsencrypt::get("crocuda.com", &account).await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn set_challenge() -> Result<()> {
        // set_challenge_key("crocuda.com", challenge).await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
