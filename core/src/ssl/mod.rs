use acme2::gen_rsa_private_key;
use acme2::Account;
use acme2::AccountBuilder;
use acme2::AuthorizationStatus;
use acme2::ChallengeStatus;
use acme2::Csr;
use acme2::DirectoryBuilder;
use acme2::OrderBuilder;
use acme2::OrderStatus;
use std::time::Duration;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// use acme2::Error;
// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

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
async fn set_letsencrypt_credentials() -> Result<Arc<Account>> {
    // Create a new ACMEv2 directory for Let's Encrypt.
    let dir = DirectoryBuilder::new(LETS_ENCRYPT_URL.lock().unwrap().clone())
        .build()
        .await
        .into_diagnostic()?;

    let mut builder = AccountBuilder::new(dir.clone());
    builder.contact(vec!["mailto:hello@lcas.dev".to_string()]);
    builder.terms_of_service_agreed(true);

    let account = builder.build().await.into_diagnostic()?;
    Ok(account)
}

async fn get_certificate(dns: &str) -> Result<()> {
    let letsencrypt_credentials = set_letsencrypt_credentials().await?;

    // Create a new order for a specific domain name.
    let mut builder = OrderBuilder::new(letsencrypt_credentials);

    builder.add_dns_identifier(dns.to_owned());
    let order = builder.build().await.into_diagnostic()?;

    // Get the list of needed authorizations for this order.
    let authorizations = order.authorizations().await.into_diagnostic()?;
    for auth in authorizations {
        // Get an http-01 challenge for this authorization (or panic
        // if it doesn't exist).
        let challenge = auth.get_challenge("http-01").unwrap();

        // At this point in time, you must configure your webserver to serve
        // a file at `https://example.com/.well-known/${challenge.token}`
        // with the content of `challenge.key_authorization()??`.

        // Start the validation of the challenge.
        let challenge = challenge.validate().await.into_diagnostic()?;

        // Poll the challenge every 5 seconds until it is in either the
        // `valid` or `invalid` state.
        let challenge = challenge
            .wait_done(Duration::from_secs(5), 3)
            .await
            .into_diagnostic()?;

        assert_eq!(challenge.status, ChallengeStatus::Valid);

        // You can now remove the challenge file hosted on your webserver.

        // Poll the authorization every 5 seconds until it is in either the
        // `valid` or `invalid` state.
        let authorization = auth
            .wait_done(Duration::from_secs(5), 3)
            .await
            .into_diagnostic()?;
        assert_eq!(authorization.status, AuthorizationStatus::Valid)
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
    let cert = order.certificate().await.into_diagnostic()?.unwrap();
    assert!(cert.len() > 1);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // get_cert().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::get_certificate;
    use miette::Result;

    // #[tokio::test]
    async fn get_dummy_cert() -> Result<()> {
        get_certificate("crocuda.com").await?;
        // println!("{:#?}", res);
        Ok(())
    }
}
