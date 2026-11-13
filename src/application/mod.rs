//! Application layer: the use cases, and the ports they need.
//!
//! A use case orchestrates calls to ports — traits for what lies outside the
//! process, such as the GPU, the file system or the gateway's API. Depends
//! only on `domain`; the adapters behind the ports live in `infrastructure`.

pub mod bench;
pub mod build;
pub mod doctor;
#[cfg(test)]
pub(crate) mod fakes;
pub mod gateway;
pub mod gateway_config;
pub mod init;
pub mod lifecycle;
pub mod ports;
pub mod toolchain;
