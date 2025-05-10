//! Dependency management library for `wgso` crate.
//!
//! See [`retrieve_dependencies`] to retrieve dependencies from a configuration file.

mod config;
mod dependencies;
mod errors;

pub use dependencies::*;
pub use errors::*;
