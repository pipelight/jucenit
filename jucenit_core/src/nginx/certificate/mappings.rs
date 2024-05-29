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
    pub chain: Vec<CertificateInfo>,
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
