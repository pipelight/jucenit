use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utils::teleport::Portal;
// Error Handling
use miette::{Error, IntoDiagnostic, Result};

// Common structs to file config and unit config
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Action {
    // Reverse proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    // Public folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot: Option<String>,
    // Error
    #[serde(rename = "return")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_number: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<Action>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<String>>,
}
