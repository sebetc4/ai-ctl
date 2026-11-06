//! ai-ctl: diagnose, build, serve and benchmark local AI backends on a single GPU.
//!
//! The crate is split into four layers with a strict dependency rule; see
//! `docs/architecture.md`.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
