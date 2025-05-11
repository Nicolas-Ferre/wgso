//! WGSO is a WebGPU Shader Orchestrator used to create GPU-native applications.
#![allow(clippy::result_large_err)]

mod cli;
mod directives;
mod error;
mod program;
mod runner;

pub use cli::*;
pub use error::*;
pub use program::*;
pub use runner::*;
