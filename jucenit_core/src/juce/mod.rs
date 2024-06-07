//!
//! The main/core struct to be manipulated
//!
//! Jucenit uses a kind of main store struct that eases the generation of
//! an nginx-unit Json configuration.
//!
//! This is a powerful intermediate
//! that is, in the end, lossy converted to a nginx-unit configuration.
//!
mod crud;
mod database;
mod from;

// Reexports
pub use crud::*;
