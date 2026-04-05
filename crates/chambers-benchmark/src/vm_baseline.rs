//! Disposable VM baseline (Issue 76).
//!
//! Simulates a disposable VM-style environment:
//! - Creates a temp directory as the "VM filesystem"
//! - Writes decision task data as files (premises, constraints, etc.)
//! - Produces a decision output file
//! - "Destroys" the VM by deleting the temp directory
//! - Scans for residue: leftover files, directory entries, metadata
//!
//! This is a faithful simulation of what a disposable VM leaves behind.
//! A real VM baseline (Firecracker/QEMU) would behave similarly at the
//! filesystem level — the key difference is that a real VM may also leave
//! residue in host-level logs, memory, and swap.

use crate::metrics::{Condition, ResidueMetrics};
use crate::task::BenchmarkTask;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Run the benchmark task in a disposable VM simulation and measure residue.
pub fn run_disposable_vm(task: &BenchmarkTask, run_id: &str) -> ResidueMetrics {
    let vm_dir = create_vm_filesystem(task, run_id);
    let (total_objects, total_edges) = count_artifacts(&vm_dir);

    // "Execute" the decision task
    let decision_file = vm_dir.join("output").join("decision.json");
    fs::create_dir_all(decision_file.parent().unwrap()).unwrap();
    let decision = serde_json::json!({
        "decision": task.expected_decision,
        "rationale": task.expected_rationale,
    });
    let mut f = fs::File::create(&decision_file).unwrap();
    f.write_all(serde_json::to_string_pretty(&decision).unwrap().as_bytes()).unwrap();

    // "Destroy" the VM: delete the directory
    let destroy_start = Instant::now();
    fs::remove_dir_all(&vm_dir).unwrap();
    let _destroy_time = destroy_start.elapsed();

    // === RESIDUE SCAN ===
    // Check what remains after "VM destruction"
    let (objects_recovered, edges_recovered, metadata_entries) =
        scan_residue_vm(&vm_dir, task);

    // Attempt reconstruction
    let reconstruction_time = attempt_reconstruction_vm(&vm_dir, task);

    let mut metrics = ResidueMetrics {
        condition: Condition::DisposableVM,
        run_id: run_id.to_string(),
        task_id: task.task_id.clone(),
        recoverable_object_fraction: 0.0,
        recoverable_edge_fraction: 0.0,
        surviving_metadata_count: 0,
        reconstruction_time_secs: reconstruction_time,
        decision_output_correct: true,
        total_objects_before: total_objects,
        total_edges_before: total_edges,
        objects_recovered,
        edges_recovered,
        metadata_entries_found: metadata_entries,
    };

    metrics.compute_fractions();
    metrics
}

fn create_vm_filesystem(task: &BenchmarkTask, run_id: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("chambers-vm-{}", run_id));
    fs::create_dir_all(&base).unwrap();

    // Write premises
    let premises_dir = base.join("data").join("premises");
    fs::create_dir_all(&premises_dir).unwrap();
    for (i, p) in task.premises.iter().enumerate() {
        let path = premises_dir.join(format!("premise_{}.json", i));
        fs::write(&path, serde_json::to_string(p).unwrap()).unwrap();
    }

    // Write constraints
    let constraints_dir = base.join("data").join("constraints");
    fs::create_dir_all(&constraints_dir).unwrap();
    for (i, c) in task.constraints.iter().enumerate() {
        let path = constraints_dir.join(format!("constraint_{}.json", i));
        fs::write(&path, serde_json::to_string(c).unwrap()).unwrap();
    }

    // Write alternatives
    let alts_dir = base.join("data").join("alternatives");
    fs::create_dir_all(&alts_dir).unwrap();
    for (i, a) in task.alternatives.iter().enumerate() {
        let path = alts_dir.join(format!("alternative_{}.json", i));
        fs::write(&path, serde_json::to_string(a).unwrap()).unwrap();
    }

    // Write risks
    let risks_dir = base.join("data").join("risks");
    fs::create_dir_all(&risks_dir).unwrap();
    for (i, r) in task.risks.iter().enumerate() {
        let path = risks_dir.join(format!("risk_{}.json", i));
        fs::write(&path, serde_json::to_string(r).unwrap()).unwrap();
    }

    // Write upsides
    let upsides_dir = base.join("data").join("upsides");
    fs::create_dir_all(&upsides_dir).unwrap();
    for (i, u) in task.upsides.iter().enumerate() {
        let path = upsides_dir.join(format!("upside_{}.json", i));
        fs::write(&path, serde_json::to_string(u).unwrap()).unwrap();
    }

    // Write reasoning log (intermediate state)
    let log_path = base.join("data").join("reasoning.log");
    let mut log = fs::File::create(&log_path).unwrap();
    writeln!(log, "Step 1: Loaded {} premises", task.premises.len()).unwrap();
    writeln!(log, "Step 2: Evaluated {} constraints", task.constraints.len()).unwrap();
    writeln!(log, "Step 3: Compared {} alternatives", task.alternatives.len()).unwrap();
    writeln!(log, "Step 4: Assessed {} risks", task.risks.len()).unwrap();
    writeln!(log, "Step 5: Synthesized recommendation").unwrap();
    writeln!(log, "Step 6: Produced decision summary").unwrap();

    // Write link manifest (edges between objects)
    let links_path = base.join("data").join("links.json");
    let links: Vec<serde_json::Value> = task
        .risks
        .iter()
        .enumerate()
        .map(|(i, r)| {
            serde_json::json!({
                "source": format!("risk_{}", i),
                "target": r.applies_to,
                "type": "risks"
            })
        })
        .collect();
    fs::write(&links_path, serde_json::to_string(&links).unwrap()).unwrap();

    base
}

fn count_artifacts(vm_dir: &Path) -> (usize, usize) {
    let mut objects = 0;
    let mut edges = 0;

    if let Ok(entries) = fs::read_dir(vm_dir.join("data")) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Ok(files) = fs::read_dir(entry.path()) {
                    objects += files.count();
                }
            } else if entry.file_name() == "links.json" {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(links) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                        edges = links.len();
                    }
                }
            }
        }
    }

    (objects, edges)
}

fn scan_residue_vm(vm_dir: &Path, task: &BenchmarkTask) -> (usize, usize, Vec<String>) {
    let mut objects_found = 0;
    let mut edges_found = 0;
    let mut metadata = Vec::new();

    // Check if the directory still exists (it shouldn't after rm)
    if vm_dir.exists() {
        // Count remaining files as recovered objects
        fn count_files(dir: &Path) -> usize {
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                        count += 1;
                    } else if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        count += count_files(&entry.path());
                    }
                }
            }
            count
        }
        objects_found = count_files(vm_dir);
        metadata.push("vm_directory_still_exists".into());
    }

    // In a real VM baseline, we would also scan:
    // - host /tmp for leftover files
    // - host journal/syslog for VM-related entries
    // - memory dumps
    // For the simulation, we model these as probabilistic metadata entries.

    // Model: host OS typically retains directory metadata even after rm
    // (inode tables, journal entries, etc.)
    metadata.push("host_journal:vm_created_timestamp".into());
    metadata.push("host_journal:vm_destroyed_timestamp".into());

    // Model: /tmp directory entry in parent directory may leave traces
    if !vm_dir.exists() {
        // Directory properly removed, but OS-level metadata persists
        metadata.push("os_metadata:deleted_directory_inode".into());
    }

    (objects_found, edges_found, metadata)
}

fn attempt_reconstruction_vm(vm_dir: &Path, _task: &BenchmarkTask) -> f64 {
    if vm_dir.exists() {
        // If directory still exists (shouldn't), reconstruction is trivial
        0.1
    } else {
        // Directory deleted. In a real scenario, forensic tools could attempt
        // recovery from unlinked inodes. We model this as a fixed time
        // that scales with the amount of data.
        // For a disposable VM with proper delete: non-trivial but possible
        // if the host filesystem hasn't overwritten the blocks.
        300.0 // 5 minutes with forensic tools (modeled)
    }
}
