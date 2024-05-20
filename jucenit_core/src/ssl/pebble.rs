// Global vars
use crate::nginx::SETTINGS;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Acme
use acme2::gen_rsa_private_key;
use acme2::{
    Account, AccountBuilder, AuthorizationStatus, Challenge, ChallengeStatus, Csr, Directory,
    DirectoryBuilder, OrderBuilder, OrderStatus,
};
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// use acme2::Error;

// Testing local ACME server
pub static PEBBLE_URL: Lazy<Arc<Mutex<String>>> =
    Lazy::new(|| Arc::new(Mutex::new("https://localhost:14000/dir".to_owned())));
pub static PEBBLE_CERT_URL: Lazy<Arc<Mutex<String>>> =
    Lazy::new(|| Arc::new(Mutex::new("https://localhost:15000/roots/0".to_owned())));

pub async fn pebble_http_client() -> reqwest::Client {
    let raw = tokio::fs::read("/etc/pebble/test/certs/pebble.minica.pem")
        .await
        .unwrap();
    let cert = reqwest::Certificate::from_pem(&raw).unwrap();
    reqwest::Client::builder()
        .add_root_certificate(cert)
        .build()
        .unwrap()
}
async fn pebble_directory() -> Arc<Directory> {
    let http_client = pebble_http_client().await;
    DirectoryBuilder::new(PEBBLE_URL.lock().unwrap().clone())
        .http_client(http_client)
        .build()
        .await
        .unwrap()
}
/**
* Create an ACME account to use for the order
* against letsencrypt/pebble acme local testing server
*/
pub async fn pebble_account() -> Result<Arc<Account>> {
    let dir = pebble_directory().await;

    let mut builder = AccountBuilder::new(dir);
    let account = builder
        .contact(vec!["mailto:hello@lcas.dev".to_string()])
        .terms_of_service_agreed(true)
        .build()
        .await
        .unwrap();

    Ok(account)
}
