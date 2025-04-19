//! WGSO is a WebGPU Shader Orchestrator used to create GPU-native applications.
#![allow(clippy::result_large_err)]

mod directive;
mod error;
mod fields;
mod file;
mod module;
mod program;
mod resource;
mod runner;
mod type_;

pub use error::*;
pub use program::*;
pub use runner::*;
