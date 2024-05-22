// Rules
// #![allow(unused_variables)]
// #![allow(unused_must_use)]

#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

pub mod globals;
// Internal Imports
pub mod dates;
pub mod error;
pub mod files;
pub mod git;
pub mod logger;
pub mod signal;
pub mod teleport;
