use acme2::{gen_rsa_private_key, Authorization};
use acme2::{
    Account, AccountBuilder, AuthorizationStatus, Challenge, ChallengeStatus, Csr, Directory,
    DirectoryBuilder, OrderBuilder, OrderStatus,
};
use serde_json::json;
use std::time::Duration;
// Error Handling
use miette::{ensure, Context, Error, IntoDiagnostic, Result};
// use acme2::Error;
// Global vars
use crate::nginx::SETTINGS;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

// File manipulation
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::path::{Path, PathBuf};
use toml::toml;
use uuid::Uuid;
// Crate structs
use super::pebble::*;
use crate::{Action, ConfigUnit, Match};
use openssl::{pkey::PKey, x509::X509};

// Production url
pub static LETS_ENCRYPT_URL: Lazy<Arc<Mutex<String>>> = Lazy::new(|| {
    Arc::new(Mutex::new(
        "https://acme-v02.api.letsencrypt.org/directory".to_owned(),
    ))
});
// Stagging url
// pub static LETS_ENCRYPT_URL: Lazy<Arc<Mutex<String>>> = Lazy::new(|| {
//     Arc::new(Mutex::new(
//         "https://acme-staging-v02.api.letsencrypt.org/directory".to_owned(),
//     ))
// });
static HTTP_PORT: i32 = 80;
static TLS_PORT: i32 = 443;

/**
* Create a new ACMEv2 directory for Let's Encrypt.
*/
async fn letsencrypt_directory() -> Result<Arc<Directory>> {
    let dir = DirectoryBuilder::new(LETS_ENCRYPT_URL.lock().await.clone())
        .build()
        .await
        .into_diagnostic()?;
    Ok(dir)
}
/**
* Create an ACME account to use for the order. For production
* purposes, you should keep the account (and private key), so
* you can renew your certificate easily.
*/
pub async fn set_account() -> Result<Arc<Account>> {
    // Set a Private key path
    let file_path = "/var/spool/jucenit/ssl_account_private_key.pem".to_owned();
    let path = Path::new(&file_path);

    #[cfg(debug_assertions)]
    let dir = pebble_directory().await?;
    #[cfg(not(debug_assertions))]
    let dir = letsencrypt_directory().await?;

    // Retrieve previous account
    if path.exists() {
        let message = format!("Couldn't open file at: {:?}", file_path);
        let data = fs::read(file_path.clone()).await.into_diagnostic()?;
        let pkey = PKey::private_key_from_pem(&data).into_diagnostic()?;

        let mut builder = AccountBuilder::new(dir.clone());
        let account = builder
            .private_key(pkey)
            .contact(vec!["mailto:areskul@areskul.com".to_string()])
            .terms_of_service_agreed(true)
            .only_return_existing(true)
            .build()
            .await
            .into_diagnostic()?;

        Ok(account)

    // Create new account
    } else {
        let mut builder = AccountBuilder::new(dir.clone());
        let account = builder
            .contact(vec!["mailto:areskul@areskul.com".to_string()])
            .terms_of_service_agreed(true)
            .build()
            .await
            .into_diagnostic()?;

        let pkey = account.private_key();
        let private_key = pkey.private_key_to_pem_pkcs8().into_diagnostic()?;

        //  Write private key to file
        let tmp_dir = "/var/spool/jucenit";
        let message = format!("Couldn't create dir: {:?}", tmp_dir);
        fs::create_dir_all(tmp_dir)
            .await
            .into_diagnostic()
            .wrap_err(message)?;

        let message = format!("Couldn't create file at: {:?}", file_path);
        let mut file = fs::File::create(file_path.clone())
            .await
            .into_diagnostic()
            .wrap_err(message)?;
        file.write_all(&private_key).await.into_diagnostic()?;

        Ok(account)
    }
}

/**
* Create a self-signed certificate to serve domain and resolve challenge.
*/
fn make_jucenit_tls_alpn_challenge_config(dns: &str, challenge: &Challenge) -> Result<String> {
    // Challenge ports
    let toml = format!(
        "
        uuid = '{}'
        listeners = ['*:{}']
        [match]
        hosts = ['{}']
        uri = '/.well-known/acme-challenge/{}'
        [action]
        share = ['/tmp/jucenit/challenge_{}.txt']
        ",
        Uuid::new_v4(),
        TLS_PORT,
        dns,
        challenge.token.clone().unwrap(),
        dns
    );
    let unit = ConfigUnit::from_toml_str(&toml)?;
    let cert = String::new();

    // todo!();

    Ok(cert)
}
/**
* Create an nginx-unit match to serve the challenge key file.
* A file at `https://example.com/.well-known/${challenge.token}`
* with the content of `challenge.key_authorization()??`.
*/
async fn make_jucenit_http_challenge_config(
    dns: &str,
    challenge: &Challenge,
) -> Result<ConfigUnit> {
    // Challenge ports
    let http_port = 80;
    let tls_port = 443;

    // Update nginx-unit config
    let toml = format!(
        "
        uuid = '{}'
        listeners = ['*:{}']
        [match]
        hosts = ['{}']
        uri = ['/.well-known/acme-challenge/{}']
        [action]
        share = ['/tmp/jucenit/challenge_{}.txt']
        ",
        Uuid::new_v4(),
        HTTP_PORT,
        dns,
        challenge.token.clone().unwrap(),
        dns
    );
    let unit = ConfigUnit::from_toml_str(&toml)?;

    Ok(unit)
}

/**
* Create tmp challenge files and nginx-unit routes
*/
async fn set_challenge_key_file(dns: &str, challenge: &Challenge) -> Result<()> {
    // Write challenge key to temporary file
    let data = challenge.key_authorization().into_diagnostic()?.unwrap();

    // Create and write to file
    let tmp_dir = "/tmp/jucenit";
    let message = format!("Couldn't create dir: {:?}", tmp_dir);
    fs::create_dir_all(tmp_dir)
        .await
        .into_diagnostic()
        .wrap_err(message)?;

    let file_path = format!("/tmp/jucenit/challenge_{}.txt", dns);
    let message = format!("Couldn't create file at: {:?}", file_path);
    let mut file = fs::File::create(file_path.clone())
        .await
        .into_diagnostic()
        .wrap_err(message)?;
    let bytes = data.as_bytes();
    file.write_all(bytes).await.into_diagnostic()?;

    Ok(())
}

/**
* Delete tmp challenge files and nginx-unit routes
*/
async fn del_challenge_key_file(dns: &str, challenge: &Challenge) -> Result<()> {
    let tmp_dir = "/tmp/jucenit";
    let path = format!("{}/challenge_{}.txt", tmp_dir, dns);
    fs::remove_file(path).await.into_diagnostic()?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct Letsencrypt;

impl Letsencrypt {
    pub async fn get_cert_bundle(dns: &str, account: &Arc<Account>) -> Result<String> {
        // Create a new order for a specific domain name.
        let mut builder = OrderBuilder::new(account.to_owned());
        builder.add_dns_identifier(dns.to_owned());
        let order = builder.build().await.into_diagnostic()?;

        // Get the list of needed authorizations for this order.
        let authorizations = order.authorizations().await.into_diagnostic()?;
        for auth in authorizations {
            // Get an tls-alpn-01 challenge
            // if let Some(challenge) = auth.get_challenge("tls-alpn-01") {
            //     Self::tls_alpn_challenge(dns, auth, &challenge).await?;
            // }
            // Get an http-01 challenge
            if let Some(challenge) = auth.get_challenge("http-01") {
                Self::http_challenge(dns, auth, &challenge).await?;
            }
        }

        let order = order
            .wait_ready(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(OrderStatus::Ready == order.status, "Order no Ready");

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

        let order = order
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(OrderStatus::Valid == order.status, "Order not Valid");

        // Download the certificate, and panic if it doesn't exist.
        let certificates = order.certificate().await.into_diagnostic()?.unwrap();
        ensure!(certificates.len() > 1, "No certificate returned");

        let mut bundle = String::new();
        for cert in certificates.clone() {
            let cert = cert.to_pem().into_diagnostic()?;
            let cert = String::from_utf8(cert).into_diagnostic()?;
            bundle += &cert;
        }
        bundle += &private_key;
        Ok(bundle)
    }
    async fn http_challenge(dns: &str, auth: Authorization, challenge: &Challenge) -> Result<()> {
        // Create route to challenge key file
        set_challenge_key_file(dns, &challenge).await?;
        let unit = make_jucenit_http_challenge_config(dns, &challenge).await?;
        unit.push().await?;

        let challenge = challenge.validate().await.into_diagnostic()?;
        let challenge = challenge
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(
            ChallengeStatus::Valid == challenge.status,
            "Http Challenge not Valid"
        );

        // Delete route to challenge key file
        del_challenge_key_file(dns, &challenge).await?;
        unit.remove().await?;

        let authorization = auth
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(
            AuthorizationStatus::Valid == authorization.status,
            "Authorization not Valid"
        );
        Ok(())
    }
    /**
     * Warning: Online ressources are relatively poor on how to implement this challenge.
     * Refer direcly to the standard at:
     * https://datatracker.ietf.org/doc/html/rfc8737
     * and read the comments
     */
    async fn tls_alpn_challenge(
        dns: &str,
        auth: Authorization,
        challenge: &Challenge,
    ) -> Result<()> {
        // Create tls cert with challenge info
        set_challenge_key_file(dns, &challenge).await?;

        let bundle = make_jucenit_tls_alpn_challenge_config(dns, &challenge)?;
        // JuceConfig::add_unit((match_.clone(), unit)).await?;

        let challenge = challenge.validate().await.into_diagnostic()?;
        let challenge = challenge
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(
            challenge.status == ChallengeStatus::Valid,
            "Tls Challenge not Valid"
        );

        // Delete route to challenge key file
        del_challenge_key_file(dns, &challenge).await?;
        // JuceConfig::del_unit(match_.clone()).await?;

        let authorization = auth
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        ensure!(
            authorization.status == AuthorizationStatus::Valid,
            "Authorization not Valid"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Letsencrypt;
    use super::*;
    use crate::database::{connect_db, fresh_db};
    use crate::nginx::CertificateStore;
    use crate::ConfigFile;
    use std::path::PathBuf;

    use miette::Result;

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

    #[tokio::test]
    async fn set_creds() -> Result<()> {
        let res = set_account().await?;
        Ok(())
    }
    #[tokio::test]
    async fn get_cert() -> Result<()> {
        set_testing_config().await?;

        let dns = "example.com";
        let account = set_account().await?.clone();
        let res = Letsencrypt::get_cert_bundle(dns, &account).await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
