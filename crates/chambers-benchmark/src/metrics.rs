//! Residue measurement metrics (Issues 81-85).
//!
//! Unified metrics for comparing semantic residue across
//! Chambers, disposable VM, and constrained microVM.

use serde::{Deserialize, Serialize};

/// Condition being measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    Chambers,
    DisposableVM,
    ConstrainedMicroVM,
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Chambers => write!(f, "Chambers"),
            Condition::DisposableVM => write!(f, "DisposableVM"),
            Condition::ConstrainedMicroVM => write!(f, "ConstrainedMicroVM"),
        }
    }
}

/// Residue measurement for a single run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueMetrics {
    pub condition: Condition,
    pub run_id: String,
    pub task_id: String,

    /// Fraction of non-preserved objects recoverable (0.0–1.0).
    /// 0.0 = no objects recoverable. 1.0 = all objects recoverable.
    pub recoverable_object_fraction: f64,

    /// Fraction of graph edges recoverable (0.0–1.0).
    pub recoverable_edge_fraction: f64,

    /// Count of surviving metadata entries beyond what should remain.
    pub surviving_metadata_count: usize,

    /// Seconds required to reconstruct intermediate reasoning steps.
    /// f64::INFINITY if reconstruction is infeasible.
    pub reconstruction_time_secs: f64,

    /// Whether the final decision output was produced correctly.
    pub decision_output_correct: bool,

    /// Total objects that existed before termination.
    pub total_objects_before: usize,

    /// Total edges that existed before termination.
    pub total_edges_before: usize,

    /// Objects recoverable after termination (non-preserved).
    pub objects_recovered: usize,

    /// Edges recoverable after termination.
    pub edges_recovered: usize,

    /// Metadata entries found beyond expected.
    pub metadata_entries_found: Vec<String>,
}

impl ResidueMetrics {
    pub fn compute_fractions(&mut self) {
        if self.total_objects_before > 0 {
            self.recoverable_object_fraction =
                self.objects_recovered as f64 / self.total_objects_before as f64;
        }
        if self.total_edges_before > 0 {
            self.recoverable_edge_fraction =
                self.edges_recovered as f64 / self.total_edges_before as f64;
        }
        self.surviving_metadata_count = self.metadata_entries_found.len();
    }
}

/// Lifecycle comprehension metrics (Issue 86-88).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensionMetrics {
    pub condition: Condition,
    pub participant_id: String,
    /// Objects the participant predicted would survive.
    pub predicted_survivors: Vec<String>,
    /// Objects that actually survived.
    pub actual_survivors: Vec<String>,
    /// Objects predicted to be destroyed.
    pub predicted_destroyed: Vec<String>,
    /// Objects actually destroyed.
    pub actual_destroyed: Vec<String>,
    /// Precision: of predicted survivors, how many actually survived.
    pub precision: f64,
    /// Recall: of actual survivors, how many were predicted.
    pub recall: f64,
    /// F1 score.
    pub f1: f64,
}

impl ComprehensionMetrics {
    pub fn compute_scores(&mut self) {
        let true_positive = self
            .predicted_survivors
            .iter()
            .filter(|p| self.actual_survivors.contains(p))
            .count() as f64;
        let false_positive = self
            .predicted_survivors
            .iter()
            .filter(|p| !self.actual_survivors.contains(p))
            .count() as f64;
        let false_negative = self
            .actual_survivors
            .iter()
            .filter(|a| !self.predicted_survivors.contains(a))
            .count() as f64;

        self.precision = if true_positive + false_positive > 0.0 {
            true_positive / (true_positive + false_positive)
        } else {
            0.0
        };
        self.recall = if true_positive + false_negative > 0.0 {
            true_positive / (true_positive + false_negative)
        } else {
            0.0
        };
        self.f1 = if self.precision + self.recall > 0.0 {
            2.0 * self.precision * self.recall / (self.precision + self.recall)
        } else {
            0.0
        };
    }
}

/// Aggregated comparison across conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub task_id: String,
    pub runs_per_condition: usize,
    pub conditions: Vec<ConditionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSummary {
    pub condition: Condition,
    pub mean_recoverable_object_fraction: f64,
    pub std_recoverable_object_fraction: f64,
    pub mean_recoverable_edge_fraction: f64,
    pub std_recoverable_edge_fraction: f64,
    pub mean_metadata_count: f64,
    pub mean_reconstruction_time: f64,
}

impl BenchmarkComparison {
    pub fn from_runs(task_id: &str, runs: &[ResidueMetrics]) -> Self {
        let mut conditions = Vec::new();

        for cond in &[Condition::Chambers, Condition::DisposableVM, Condition::ConstrainedMicroVM] {
            let cond_runs: Vec<&ResidueMetrics> =
                runs.iter().filter(|r| r.condition == *cond).collect();

            if cond_runs.is_empty() {
                continue;
            }

            let n = cond_runs.len() as f64;
            let obj_fracs: Vec<f64> = cond_runs.iter().map(|r| r.recoverable_object_fraction).collect();
            let edge_fracs: Vec<f64> = cond_runs.iter().map(|r| r.recoverable_edge_fraction).collect();
            let meta_counts: Vec<f64> = cond_runs.iter().map(|r| r.surviving_metadata_count as f64).collect();
            let recon_times: Vec<f64> = cond_runs.iter().map(|r| r.reconstruction_time_secs).collect();

            conditions.push(ConditionSummary {
                condition: *cond,
                mean_recoverable_object_fraction: mean(&obj_fracs),
                std_recoverable_object_fraction: std_dev(&obj_fracs),
                mean_recoverable_edge_fraction: mean(&edge_fracs),
                std_recoverable_edge_fraction: std_dev(&edge_fracs),
                mean_metadata_count: mean(&meta_counts),
                mean_reconstruction_time: mean(&recon_times),
            });
        }

        Self {
            task_id: task_id.to_string(),
            runs_per_condition: runs.len() / 3.max(1),
            conditions,
        }
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 { return 0.0; }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}
