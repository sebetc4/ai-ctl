//! Domain layer: the entities and rules of the inference stack.
//!
//! Pure Rust with no I/O. Depends on nothing else in the crate; every other
//! layer may depend on it.

pub mod bench;
pub mod build;
pub mod diagnostics;
pub mod gateway;
pub mod gateway_config;
pub mod inventory;
pub mod machine;
pub mod prompt;
pub mod recipe;
pub mod safetensors;
pub mod stats;
pub mod text;
pub mod timestamp;
