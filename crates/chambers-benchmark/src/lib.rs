//! Chambers Phase 0 benchmark harness.
//!
//! Compares Chambers against disposable VM and constrained microVM
//! baselines using the paper's falsifiable evaluation agenda.

pub mod task;
pub mod metrics;
pub mod chambers_runner;
pub mod vm_baseline;
pub mod microvm_baseline;
pub mod hypothesis;
pub mod comprehension;
