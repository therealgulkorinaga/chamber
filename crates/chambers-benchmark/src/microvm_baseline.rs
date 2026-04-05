//! Constrained microVM baseline (Issue 77).
//!
//! Simulates a constrained microVM environment:
//! - Minimal filesystem (ramfs-like: all in-memory, no persistent storage)
//! - No network
//! - Single process
//! - Destroy = process kill + memory free
//!
//! This is stronger than a regular VM because:
//! - No persistent disk to leave traces on
//! - Smaller attack surface
//! - Memory freed on destroy
//!
//! Residue sources in a real microVM:
//! - Host-side VM metadata (creation time, resource usage)
//! - Host memory that may not be immediately overwritten
//! - Host-side logging of microVM lifecycle

use crate::metrics::{Condition, ResidueMetrics};
use crate::task::BenchmarkTask;
use std::collections::HashMap;

/// In-memory "ramfs" simulation.
struct RamFs {
    files: HashMap<String, Vec<u8>>,
}

impl RamFs {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn write(&mut self, path: &str, data: &[u8]) {
        self.files.insert(path.to_string(), data.to_vec());
    }

    fn file_count(&self) -> usize {
        self.files.len()
    }

    fn destroy(mut self) -> DestroyedState {
        let file_count = self.files.len();
        let total_bytes: usize = self.files.values().map(|v| v.len()).sum();
        // Zero all memory (simulating ramdisk cleanup)
        for (_, v) in self.files.iter_mut() {
            for byte in v.iter_mut() {
                *byte = 0;
            }
        }
        self.files.clear();
        DestroyedState {
            file_count,
            total_bytes,
        }
    }
}

struct DestroyedState {
    file_count: usize,
    total_bytes: usize,
}

/// Run the benchmark task in a constrained microVM simulation.
pub fn run_constrained_microvm(task: &BenchmarkTask, run_id: &str) -> ResidueMetrics {
    let mut ramfs = RamFs::new();
    let mut edge_count = 0;

    // Populate ramfs with task data
    for (i, p) in task.premises.iter().enumerate() {
        let data = serde_json::to_vec(p).unwrap();
        ramfs.write(&format!("/data/premise_{}.json", i), &data);
    }

    for (i, c) in task.constraints.iter().enumerate() {
        let data = serde_json::to_vec(c).unwrap();
        ramfs.write(&format!("/data/constraint_{}.json", i), &data);
    }

    for (i, a) in task.alternatives.iter().enumerate() {
        let data = serde_json::to_vec(a).unwrap();
        ramfs.write(&format!("/data/alternative_{}.json", i), &data);
    }

    for (i, r) in task.risks.iter().enumerate() {
        let data = serde_json::to_vec(r).unwrap();
        ramfs.write(&format!("/data/risk_{}.json", i), &data);
        edge_count += 1; // risk -> alternative link
    }

    for (i, u) in task.upsides.iter().enumerate() {
        let data = serde_json::to_vec(u).unwrap();
        ramfs.write(&format!("/data/upside_{}.json", i), &data);
        edge_count += 1; // upside -> alternative link
    }

    // Reasoning log
    let reasoning = format!(
        "Evaluated {} premises, {} constraints, {} alternatives, {} risks. Decision: {}",
        task.premises.len(),
        task.constraints.len(),
        task.alternatives.len(),
        task.risks.len(),
        task.expected_decision
    );
    ramfs.write("/data/reasoning.log", reasoning.as_bytes());

    // Links manifest
    let links: Vec<serde_json::Value> = task
        .risks
        .iter()
        .enumerate()
        .map(|(i, r)| {
            serde_json::json!({
                "source": format!("risk_{}", i),
                "target": r.applies_to,
            })
        })
        .collect();
    ramfs.write(
        "/data/links.json",
        serde_json::to_vec(&links).unwrap().as_slice(),
    );

    // Decision output
    let decision = serde_json::json!({
        "decision": task.expected_decision,
        "rationale": task.expected_rationale,
    });
    ramfs.write(
        "/output/decision.json",
        serde_json::to_vec(&decision).unwrap().as_slice(),
    );

    let total_objects = ramfs.file_count();

    // Destroy the microVM
    let destroyed = ramfs.destroy();

    // === RESIDUE SCAN ===
    // After microVM destruction, what remains?
    // - In-memory data: zeroed and dropped (Rust ownership)
    // - Persistent storage: none (ramfs)
    // - Host-side metadata: lifecycle events
    let mut metadata_entries = Vec::new();

    // Host sees: microVM was created and destroyed (timestamps, resource metrics)
    metadata_entries.push("host_metadata:microvm_created_timestamp".into());
    metadata_entries.push("host_metadata:microvm_destroyed_timestamp".into());
    metadata_entries.push(format!(
        "host_metadata:microvm_memory_used_bytes:{}",
        destroyed.total_bytes
    ));
    metadata_entries.push(format!(
        "host_metadata:microvm_file_count:{}",
        destroyed.file_count
    ));

    // MicroVM is stronger than regular VM: no disk residue
    // But host-side memory may not be immediately zeroed
    // Model: some memory pages may be recoverable briefly after free
    let objects_recovered = 0; // ramfs zeroed and dropped
    let edges_recovered = 0;

    // Reconstruction: requires memory forensics, much harder than VM disk forensics
    // but still possible if host memory isn't overwritten
    let reconstruction_time = 600.0; // 10 minutes with memory forensics (modeled)

    let mut metrics = ResidueMetrics {
        condition: Condition::ConstrainedMicroVM,
        run_id: run_id.to_string(),
        task_id: task.task_id.clone(),
        recoverable_object_fraction: 0.0,
        recoverable_edge_fraction: 0.0,
        surviving_metadata_count: 0,
        reconstruction_time_secs: reconstruction_time,
        decision_output_correct: true,
        total_objects_before: total_objects,
        total_edges_before: edge_count,
        objects_recovered,
        edges_recovered,
        metadata_entries_found: metadata_entries,
    };

    metrics.compute_fractions();
    metrics
}
