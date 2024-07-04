use serde::{Deserialize, Serialize};
use std::default::Default;
// Global vars
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Structs
use super::{CertificateStore, Config};

pub static SETTINGS: Lazy<Arc<Mutex<Settings>>> =
    Lazy::new(|| Arc::new(Mutex::new(Settings::default())));

/*
* A struct to query the good nginx-unit socket or port.
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub url: Option<String>,
    pub socket: Option<String>,
    pub state_dir: Option<String>,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            url: Some("http://127.0.0.1:8080".to_string()),
            socket: None,
            state_dir: Some("/var/spool/unit".to_string()),
        }
    }
}
impl Settings {
    pub fn get_url(&self) -> String {
        if let Some(url) = &self.url {
            return url.to_owned();
        } else if let Some(socket) = &self.socket {
            return socket.to_owned();
        } else {
            return Settings::default().url.unwrap();
        }
    }
}

// Unit identical structs
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Nginx {
    pub config: Config,
    pub certificates: serde_json::Value,
    pub status: serde_json::Value,
    #[serde(skip)]
    pub settings: Settings,
}
