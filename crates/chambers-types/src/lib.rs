//! Core types for the Chambers Phase 0 substrate runtime.
//!
//! This crate defines the shared data model used across all engines:
//! World, Object, CapabilityToken, Artifact, lifecycle enums,
//! primitive operations, and transition requests.

pub mod world;
pub mod object;
pub mod capability;
pub mod artifact;
pub mod primitive;
pub mod error;
pub mod grammar;
