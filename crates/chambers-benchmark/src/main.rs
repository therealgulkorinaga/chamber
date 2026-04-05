//! Chambers Phase 0 benchmark runner (Issue 90).
//!
//! Runs the canonical decision task N times in each condition,
//! collects metrics, tests H1–H3, and produces the falsification report.

use chambers_benchmark::chambers_runner;
use chambers_benchmark::hypothesis::FalsificationReport;
use chambers_benchmark::metrics::{BenchmarkComparison, ResidueMetrics};
use chambers_benchmark::microvm_baseline;
use chambers_benchmark::task::canonical_task;
use chambers_benchmark::vm_baseline;
use std::fs;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    println!("╔══════════════════════════════════════════════════╗");
    println!("║   CHAMBERS PHASE 0 — BENCHMARK HARNESS          ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("Runs per condition: {}", n);
    println!();

    let task = canonical_task();
    let mut all_metrics: Vec<ResidueMetrics> = Vec::new();

    // === Chambers condition ===
    println!("Running Chambers condition ({} runs)...", n);
    let start = Instant::now();
    for i in 0..n {
        let run_id = format!("chambers-{}", i);
        let metrics = chambers_runner::run_chambers(&task, &run_id);
        println!(
            "  [{}] obj={:.4} edge={:.4} meta={} recon={}",
            run_id,
            metrics.recoverable_object_fraction,
            metrics.recoverable_edge_fraction,
            metrics.surviving_metadata_count,
            if metrics.reconstruction_time_secs.is_infinite() {
                "∞".to_string()
            } else {
                format!("{:.0}s", metrics.reconstruction_time_secs)
            }
        );
        all_metrics.push(metrics);
    }
    println!("  Completed in {:.2}s\n", start.elapsed().as_secs_f64());

    // === Disposable VM condition ===
    println!("Running Disposable VM condition ({} runs)...", n);
    let start = Instant::now();
    for i in 0..n {
        let run_id = format!("vm-{}", i);
        let metrics = vm_baseline::run_disposable_vm(&task, &run_id);
        println!(
            "  [{}] obj={:.4} edge={:.4} meta={} recon={:.0}s",
            run_id,
            metrics.recoverable_object_fraction,
            metrics.recoverable_edge_fraction,
            metrics.surviving_metadata_count,
            metrics.reconstruction_time_secs,
        );
        all_metrics.push(metrics);
    }
    println!("  Completed in {:.2}s\n", start.elapsed().as_secs_f64());

    // === Constrained microVM condition ===
    println!("Running Constrained microVM condition ({} runs)...", n);
    let start = Instant::now();
    for i in 0..n {
        let run_id = format!("microvm-{}", i);
        let metrics = microvm_baseline::run_constrained_microvm(&task, &run_id);
        println!(
            "  [{}] obj={:.4} edge={:.4} meta={} recon={:.0}s",
            run_id,
            metrics.recoverable_object_fraction,
            metrics.recoverable_edge_fraction,
            metrics.surviving_metadata_count,
            metrics.reconstruction_time_secs,
        );
        all_metrics.push(metrics);
    }
    println!("  Completed in {:.2}s\n", start.elapsed().as_secs_f64());

    // === Compute comparison ===
    let comparison = BenchmarkComparison::from_runs(&task.task_id, &all_metrics);

    // === Generate falsification report ===
    // H2 requires user study data — mark as inconclusive for now
    let report = FalsificationReport::generate(comparison, &[], &[]);

    println!();
    report.print();

    // === Save results ===
    let results_dir = "benchmarks/results";
    fs::create_dir_all(results_dir).ok();

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let raw_path = format!("{}/raw_{}.jsonl", results_dir, timestamp);
    let report_path = format!("{}/report_{}.json", results_dir, timestamp);

    // Save raw metrics as JSONL
    let raw_data: String = all_metrics
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&raw_path, &raw_data).ok();

    // Save report as JSON
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    fs::write(&report_path, &report_json).ok();

    println!("\nResults saved:");
    println!("  Raw: {}", raw_path);
    println!("  Report: {}", report_path);
}
