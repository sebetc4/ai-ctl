//! Infrastructure layer: the adapters that implement the application ports.
//!
//! This is where I/O happens: NVML, external commands, files, HTTP. Depends on
//! `application` (for the port traits) and `domain`; nothing depends on it
//! except the composition root in `main.rs`.

pub mod api_key;
pub mod build_store;
pub mod chat_api;
pub mod cmake;
pub mod command;
pub mod download;
pub mod files;
pub mod gateway_api;
pub mod gateway_process;
pub mod gateway_store;
pub mod git;
pub mod host;
pub mod http;
pub mod inventory_file;
pub mod nvml;
pub mod paths;
pub mod podman;
pub mod run_store;
pub mod server_api;
