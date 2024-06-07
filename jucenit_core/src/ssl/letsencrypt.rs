use acme2::{gen_rsa_private_key, Authorization};
use acme2::{
    Account, AccountBuilder, AuthorizationStatus, Challenge, ChallengeStatus, Csr, Directory,
    DirectoryBuilder, OrderBuilder, OrderStatus,
};
use std::time::Duration;
// Error Handling
use miette::{Context, Error, IntoDiagnostic, Result};
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
use crate::cast::{Action, Match};
use crate::juce::{Config as JuceConfig, Unit as JuceUnit, UnitKind};
use openssl::x509::X509;

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

/**
* Create an ACME account to use for the order. For production
* purposes, you should keep the account (and private key), so
* you can renew your certificate easily.
*/
pub async fn letsencrypt_account() -> Result<Arc<Account>> {
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
* Create a self-signed certificate to serve domain and resolve challenge.
*/
fn make_jucenit_tls_alpn_challenge_config(dns: &str, challenge: &Challenge) -> Result<String> {
    // Challenge ports
    let http_port = 80;
    let tls_port = 443;
    // Update nginx-unit config
    let match_ = Match {
        uri: Some(format!(
            "/.well-known/acme-challenge/{}",
            challenge.token.clone().unwrap()
        )),
        host: Some(dns.to_owned()),
        ..Match::default()
    };
    let unit = JuceUnit {
        id: Some(format!("challenge_{}", dns)),
        kind: UnitKind::HttpChallenge,
        listeners: vec![format!("*:{}", http_port)],
        action: Some(Action {
            share: Some(vec![format!("/tmp/jucenit/challenge_{}.txt", dns)]),
            ..Action::default()
        }),
    };
    let cert = String::new();
    Ok(cert)
}
/**
* Create an nginx-unit match to serve the challenge key file.
* A file at `https://example.com/.well-known/${challenge.token}`
* with the content of `challenge.key_authorization()??`.
*/
fn make_jucenit_http_challenge_config(
    dns: &str,
    challenge: &Challenge,
) -> Result<(Match, JuceUnit)> {
    // Challenge ports
    let http_port = 80;
    let tls_port = 443;
    // Update nginx-unit config
    let match_ = Match {
        uri: Some(format!(
            "/.well-known/acme-challenge/{}",
            challenge.token.clone().unwrap()
        )),
        host: Some(dns.to_owned()),
        ..Match::default()
    };
    let unit = JuceUnit {
        id: Some(format!("challenge_{}", dns)),
        kind: UnitKind::HttpChallenge,
        listeners: vec![format!("*:{}", http_port)],
        action: Some(Action {
            share: Some(vec![format!("/tmp/jucenit/challenge_{}.txt", dns)]),
            ..Action::default()
        }),
    };
    Ok((match_, unit))
}

/**
* Create tmp challenge files and nginx-unit routes
*/
fn set_challenge_key_file(dns: &str, challenge: &Challenge) -> Result<()> {
    // Write challenge key to temporary file
    let data = challenge.key_authorization().into_diagnostic()?.unwrap();

    // Create and write to file
    let tmp_dir = "/tmp/jucenit";
    let message = format!("Couldn't create dir: {:?}", tmp_dir);
    fs::create_dir_all(tmp_dir)
        .into_diagnostic()
        .wrap_err(message)?;

    let file_path = format!("/tmp/jucenit/challenge_{}.txt", dns);
    let message = format!("Couldn't create file at: {:?}", file_path);
    let mut file = fs::File::create(file_path.clone())
        .into_diagnostic()
        .wrap_err(message)?;
    let bytes = data.as_bytes();
    file.write_all(bytes).into_diagnostic()?;

    Ok(())
}

/**
* Delete tmp challenge files and nginx-unit routes
*/
fn del_challenge_key_file(dns: &str, challenge: &Challenge) -> Result<()> {
    let tmp_dir = "/tmp/jucenit";
    let path = format!("{}/challenge_{}.txt", tmp_dir, dns);
    fs::remove_file(path).into_diagnostic()?;
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

        let order = order
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        assert_eq!(order.status, OrderStatus::Valid);

        // Download the certificate, and panic if it doesn't exist.
        let certificates = order.certificate().await.into_diagnostic()?.unwrap();
        assert!(certificates.len() > 1);

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
        set_challenge_key_file(dns, &challenge)?;
        let (match_, unit) = make_jucenit_http_challenge_config(dns, &challenge)?;
        JuceConfig::add_unit((match_.clone(), unit)).await?;

        let challenge = challenge.validate().await.into_diagnostic()?;
        let challenge = challenge
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        assert_eq!(challenge.status, ChallengeStatus::Valid);

        // Delete route to challenge key file
        del_challenge_key_file(dns, &challenge)?;
        JuceConfig::del_unit(match_.clone()).await?;

        let authorization = auth
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        assert_eq!(authorization.status, AuthorizationStatus::Valid);
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
        set_challenge_key_file(dns, &challenge)?;

        let bundle = make_jucenit_tls_alpn_challenge_config(dns, &challenge)?;
        // JuceConfig::add_unit((match_.clone(), unit)).await?;

        let challenge = challenge.validate().await.into_diagnostic()?;
        let challenge = challenge
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        assert_eq!(challenge.status, ChallengeStatus::Valid);

        // Delete route to challenge key file
        //
        // del_challenge_key_file(dns, &challenge)?;
        // JuceConfig::del_unit(match_.clone()).await?;

        let authorization = auth
            .wait_done(Duration::from_secs(5), 10)
            .await
            .into_diagnostic()?;
        assert_eq!(authorization.status, AuthorizationStatus::Valid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Letsencrypt;
    use super::*;
    use crate::nginx::CertificateStore;
    use crate::ConfigFile;
    use miette::Result;

    /**
     * Set a fresh testing environment
     */
    async fn set_testing_config() -> Result<()> {
        // Clean config and certificate store
        CertificateStore::clean().await?;
        JuceConfig::set(&JuceConfig::default()).await?;

        // Set new configuration
        let config_file = ConfigFile::load("../examples/jucenit.toml")?;
        let juce_config = JuceConfig::from(&config_file);
        JuceConfig::set(&juce_config).await?;

        Ok(())
    }

    #[tokio::test]
    async fn set_local_creds() -> Result<()> {
        let res = pebble_account().await?;
        Ok(())
    }
    // #[tokio::test]
    async fn set_remote_creds() -> Result<()> {
        let res = letsencrypt_account().await?;
        println!("{:#?}", res);
        Ok(())
    }

    // #[tokio::test]
    async fn get_letsencrypt_cert() -> Result<()> {
        set_testing_config().await?;

        let dns = "example.com";
        let account = letsencrypt_account().await?.clone();
        let res = Letsencrypt::get_cert_bundle(dns, &account).await?;
        // println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn get_pebble_cert() -> Result<()> {
        set_testing_config().await?;

        let dns = "example.com";
        let account = pebble_account().await?.clone();
        let res = Letsencrypt::get_cert_bundle(dns, &account).await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
