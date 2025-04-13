//! WGSO is a WebGPU Shader Orchestrator used to create GPU-native applications.
#![allow(clippy::result_large_err)]

mod directive;
mod error;
mod file;
mod program;
mod runner;
mod storage;
mod wgsl_module;
mod wgsl_parsing;

pub use error::*;
pub use program::*;
pub use runner::*;
