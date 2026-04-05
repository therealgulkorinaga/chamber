//! Benchmark integration tests (Issues 91-94, 100-102).
//!
//! Verifies the full benchmark pipeline: run all conditions,
//! compute comparison, test hypotheses.

use chambers_benchmark::chambers_runner;
use chambers_benchmark::comprehension::*;
use chambers_benchmark::hypothesis::*;
use chambers_benchmark::metrics::*;
use chambers_benchmark::microvm_baseline;
use chambers_benchmark::task::canonical_task;
use chambers_benchmark::vm_baseline;

#[test]
fn test_chambers_produces_zero_residue() {
    let task = canonical_task();
    let metrics = chambers_runner::run_chambers(&task, "test-0");

    assert_eq!(metrics.recoverable_object_fraction, 0.0);
    assert_eq!(metrics.recoverable_edge_fraction, 0.0);
    assert_eq!(metrics.surviving_metadata_count, 0);
    assert!(metrics.reconstruction_time_secs.is_infinite());
    assert!(metrics.decision_output_correct);
}

#[test]
fn test_vm_retains_metadata() {
    let task = canonical_task();
    let metrics = vm_baseline::run_disposable_vm(&task, "test-vm-0");

    // VM should have zero object/edge recovery (directory deleted)
    assert_eq!(metrics.recoverable_object_fraction, 0.0);
    // But should retain OS-level metadata
    assert!(metrics.surviving_metadata_count > 0);
    // Reconstruction should be possible (finite time)
    assert!(metrics.reconstruction_time_secs.is_finite());
}

#[test]
fn test_microvm_retains_metadata() {
    let task = canonical_task();
    let metrics = microvm_baseline::run_constrained_microvm(&task, "test-microvm-0");

    assert_eq!(metrics.recoverable_object_fraction, 0.0);
    assert!(metrics.surviving_metadata_count > 0);
    assert!(metrics.reconstruction_time_secs.is_finite());
}

#[test]
fn test_h1_passes() {
    let task = canonical_task();
    let mut all: Vec<ResidueMetrics> = Vec::new();

    for i in 0..3 {
        all.push(chambers_runner::run_chambers(&task, &format!("h1-c-{}", i)));
        all.push(vm_baseline::run_disposable_vm(&task, &format!("h1-v-{}", i)));
        all.push(microvm_baseline::run_constrained_microvm(&task, &format!("h1-m-{}", i)));
    }

    let comparison = BenchmarkComparison::from_runs(&task.task_id, &all);
    let verdict = test_h1(&comparison);

    assert!(
        matches!(verdict, Verdict::Supported { .. }),
        "H1 should be supported: {:?}",
        verdict
    );
}

#[test]
fn test_h3_passes() {
    let task = canonical_task();
    let mut all: Vec<ResidueMetrics> = Vec::new();

    for i in 0..3 {
        all.push(chambers_runner::run_chambers(&task, &format!("h3-c-{}", i)));
        all.push(vm_baseline::run_disposable_vm(&task, &format!("h3-v-{}", i)));
        all.push(microvm_baseline::run_constrained_microvm(&task, &format!("h3-m-{}", i)));
    }

    let comparison = BenchmarkComparison::from_runs(&task.task_id, &all);
    let verdict = test_h3(&comparison);

    assert!(
        matches!(verdict, Verdict::Supported { .. }),
        "H3 should be supported: {:?}",
        verdict
    );
}

#[test]
fn test_comprehension_scoring() {
    let task = canonical_task();
    let scenario = chambers_scenario(&task);

    // Perfect prediction: user correctly identifies decision_summary as sole survivor
    let perfect = ParticipantPrediction {
        participant_id: "p1".into(),
        scenario_id: scenario.scenario_id.clone(),
        predicted_survivors: vec!["decision_summary".into()],
        predicted_destroyed: vec![
            "premise".into(),
            "constraint".into(),
            "alternative".into(),
            "risk".into(),
            "upside".into(),
            "recommendation".into(),
        ],
        confidence: 5,
    };

    let metrics = score_prediction(&scenario, &perfect);
    assert_eq!(metrics.precision, 1.0, "perfect prediction should have precision 1.0");
    assert_eq!(metrics.recall, 1.0, "perfect prediction should have recall 1.0");
    assert_eq!(metrics.f1, 1.0, "perfect prediction should have F1 1.0");

    // Wrong prediction: user thinks premises survive
    let wrong = ParticipantPrediction {
        participant_id: "p2".into(),
        scenario_id: scenario.scenario_id.clone(),
        predicted_survivors: vec!["premise".into(), "decision_summary".into()],
        predicted_destroyed: vec!["risk".into()],
        confidence: 2,
    };

    let metrics2 = score_prediction(&scenario, &wrong);
    assert!(metrics2.precision < 1.0, "wrong prediction should have lower precision");
    assert_eq!(metrics2.recall, 1.0, "recall should be 1.0 (predicted the actual survivor)");
}

#[test]
fn test_falsification_report_generation() {
    let task = canonical_task();
    let mut all: Vec<ResidueMetrics> = Vec::new();

    for i in 0..3 {
        all.push(chambers_runner::run_chambers(&task, &format!("rep-c-{}", i)));
        all.push(vm_baseline::run_disposable_vm(&task, &format!("rep-v-{}", i)));
        all.push(microvm_baseline::run_constrained_microvm(&task, &format!("rep-m-{}", i)));
    }

    let comparison = BenchmarkComparison::from_runs(&task.task_id, &all);
    let report = FalsificationReport::generate(comparison, &[], &[]);

    assert!(matches!(report.h1, Verdict::Supported { .. }));
    assert!(matches!(report.h2, Verdict::Inconclusive { .. }));
    assert!(matches!(report.h3, Verdict::Supported { .. }));

    // Report should serialize to JSON
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("SUPPORTED"));
}
