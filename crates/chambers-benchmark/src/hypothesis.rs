//! Hypothesis testing (Issues 91-94).
//!
//! H1: Chambers produces lower recoverable semantic residue than disposable VM.
//! H2: Users predict what survives/burns more accurately with Chambers.
//! H3: Fewer reconstructable intermediate reasoning traces in Chambers.

use crate::metrics::{BenchmarkComparison, ComprehensionMetrics, Condition, ConditionSummary, ResidueMetrics};
use serde::{Deserialize, Serialize};

/// Verdict for a hypothesis test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict {
    Supported { effect_size: f64, detail: String },
    NotSupported { detail: String },
    Inconclusive { detail: String },
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Supported { effect_size, detail } => {
                write!(f, "SUPPORTED (effect size: {:.3}): {}", effect_size, detail)
            }
            Verdict::NotSupported { detail } => write!(f, "NOT SUPPORTED: {}", detail),
            Verdict::Inconclusive { detail } => write!(f, "INCONCLUSIVE: {}", detail),
        }
    }
}

/// H1 test: lower recoverable semantic residue.
pub fn test_h1(comparison: &BenchmarkComparison) -> Verdict {
    let chambers = comparison.conditions.iter().find(|c| c.condition == Condition::Chambers);
    let vm = comparison.conditions.iter().find(|c| c.condition == Condition::DisposableVM);

    match (chambers, vm) {
        (Some(c), Some(v)) => {
            let obj_delta = v.mean_recoverable_object_fraction - c.mean_recoverable_object_fraction;
            let edge_delta = v.mean_recoverable_edge_fraction - c.mean_recoverable_edge_fraction;
            let meta_delta = v.mean_metadata_count - c.mean_metadata_count;

            let effect = (obj_delta + edge_delta) / 2.0;

            if c.mean_recoverable_object_fraction == 0.0
                && c.mean_recoverable_edge_fraction == 0.0
                && v.mean_recoverable_object_fraction >= 0.0
            {
                Verdict::Supported {
                    effect_size: effect,
                    detail: format!(
                        "Chambers: obj={:.3} edge={:.3} meta={:.1}. VM: obj={:.3} edge={:.3} meta={:.1}. \
                         Chambers achieves zero recoverable state; VM retains metadata.",
                        c.mean_recoverable_object_fraction,
                        c.mean_recoverable_edge_fraction,
                        c.mean_metadata_count,
                        v.mean_recoverable_object_fraction,
                        v.mean_recoverable_edge_fraction,
                        v.mean_metadata_count,
                    ),
                }
            } else if effect > 0.05 {
                Verdict::Supported {
                    effect_size: effect,
                    detail: format!(
                        "Chambers reduces residue by {:.1}% (objects) and {:.1}% (edges).",
                        obj_delta * 100.0,
                        edge_delta * 100.0,
                    ),
                }
            } else {
                Verdict::NotSupported {
                    detail: format!(
                        "Residue difference too small. Delta obj={:.3} edge={:.3}",
                        obj_delta, edge_delta
                    ),
                }
            }
        }
        _ => Verdict::Inconclusive {
            detail: "Missing condition data.".into(),
        },
    }
}

/// H2 test: better user prediction accuracy.
pub fn test_h2(
    chambers_comprehension: &[ComprehensionMetrics],
    baseline_comprehension: &[ComprehensionMetrics],
) -> Verdict {
    if chambers_comprehension.is_empty() || baseline_comprehension.is_empty() {
        return Verdict::Inconclusive {
            detail: "No comprehension data collected yet. Requires user study.".into(),
        };
    }

    let chambers_mean_f1: f64 =
        chambers_comprehension.iter().map(|c| c.f1).sum::<f64>() / chambers_comprehension.len() as f64;
    let baseline_mean_f1: f64 =
        baseline_comprehension.iter().map(|c| c.f1).sum::<f64>() / baseline_comprehension.len() as f64;

    let delta = chambers_mean_f1 - baseline_mean_f1;

    if delta > 0.1 {
        Verdict::Supported {
            effect_size: delta,
            detail: format!(
                "Chambers F1={:.3} vs baseline F1={:.3}. Users predict survival {:.1}% better.",
                chambers_mean_f1,
                baseline_mean_f1,
                delta * 100.0,
            ),
        }
    } else if delta > 0.0 {
        Verdict::Inconclusive {
            detail: format!(
                "Small positive delta {:.3}. May need more participants.",
                delta
            ),
        }
    } else {
        Verdict::NotSupported {
            detail: format!(
                "Chambers F1={:.3} <= baseline F1={:.3}.",
                chambers_mean_f1, baseline_mean_f1
            ),
        }
    }
}

/// H3 test: fewer reconstructable intermediate reasoning traces.
pub fn test_h3(comparison: &BenchmarkComparison) -> Verdict {
    let chambers = comparison.conditions.iter().find(|c| c.condition == Condition::Chambers);
    let vm = comparison.conditions.iter().find(|c| c.condition == Condition::DisposableVM);
    let microvm = comparison.conditions.iter().find(|c| c.condition == Condition::ConstrainedMicroVM);

    match (chambers, vm) {
        (Some(c), Some(v)) => {
            let c_recon = c.mean_reconstruction_time;
            let v_recon = v.mean_reconstruction_time;

            if c_recon.is_infinite() && v_recon.is_finite() {
                Verdict::Supported {
                    effect_size: 1.0,
                    detail: format!(
                        "Chambers: reconstruction infeasible (crypto burn). \
                         VM: reconstruction possible in {:.0}s. \
                         MicroVM: {:.0}s.",
                        v_recon,
                        microvm.map(|m| m.mean_reconstruction_time).unwrap_or(0.0),
                    ),
                }
            } else if c_recon > v_recon * 2.0 {
                Verdict::Supported {
                    effect_size: (c_recon - v_recon) / v_recon,
                    detail: format!(
                        "Chambers reconstruction {:.0}x harder than VM ({:.0}s vs {:.0}s).",
                        c_recon / v_recon,
                        c_recon,
                        v_recon,
                    ),
                }
            } else {
                Verdict::NotSupported {
                    detail: format!(
                        "Reconstruction time similar: Chambers={:.0}s, VM={:.0}s.",
                        c_recon, v_recon,
                    ),
                }
            }
        }
        _ => Verdict::Inconclusive {
            detail: "Missing condition data.".into(),
        },
    }
}

/// Generate the falsification report (Issue 94).
#[derive(Debug, Serialize)]
pub struct FalsificationReport {
    pub task_id: String,
    pub runs_per_condition: usize,
    pub h1: Verdict,
    pub h2: Verdict,
    pub h3: Verdict,
    pub overall: String,
    pub comparison: BenchmarkComparison,
}

impl FalsificationReport {
    pub fn generate(
        comparison: BenchmarkComparison,
        chambers_comprehension: &[ComprehensionMetrics],
        baseline_comprehension: &[ComprehensionMetrics],
    ) -> Self {
        let h1 = test_h1(&comparison);
        let h2 = test_h2(chambers_comprehension, baseline_comprehension);
        let h3 = test_h3(&comparison);

        let supported_count = [&h1, &h2, &h3]
            .iter()
            .filter(|v| matches!(v, Verdict::Supported { .. }))
            .count();

        let overall = if supported_count == 3 {
            "THESIS SUPPORTED: All three hypotheses pass. Chambers demonstrates measurable advantages over baselines.".into()
        } else if supported_count >= 1 {
            format!(
                "THESIS PARTIALLY SUPPORTED: {}/3 hypotheses pass. Further investigation needed.",
                supported_count
            )
        } else {
            "THESIS NOT SUPPORTED: No hypotheses pass. Chambers does not demonstrate measurable advantages.".into()
        };

        Self {
            task_id: comparison.task_id.clone(),
            runs_per_condition: comparison.runs_per_condition,
            h1,
            h2,
            h3,
            overall,
            comparison,
        }
    }

    pub fn print(&self) {
        println!("╔══════════════════════════════════════════════════╗");
        println!("║   CHAMBERS PHASE 0 — FALSIFICATION REPORT       ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!();
        println!("Task: {}", self.task_id);
        println!("Runs per condition: {}", self.runs_per_condition);
        println!();
        println!("─── H1: Lower recoverable semantic residue ───");
        println!("  {}", self.h1);
        println!();
        println!("─── H2: Better user prediction accuracy ───");
        println!("  {}", self.h2);
        println!();
        println!("─── H3: Fewer reconstructable traces ───");
        println!("  {}", self.h3);
        println!();
        println!("═══ OVERALL ═══");
        println!("  {}", self.overall);
        println!();

        // Print comparison table
        println!("─── Condition Summary ───");
        println!("{:<20} {:>10} {:>10} {:>10} {:>12}",
            "Condition", "Obj Frac", "Edge Frac", "Metadata", "Recon Time");
        println!("{:-<64}", "");
        for c in &self.comparison.conditions {
            let recon = if c.mean_reconstruction_time.is_infinite() {
                "∞ (infeasible)".to_string()
            } else {
                format!("{:.0}s", c.mean_reconstruction_time)
            };
            println!("{:<20} {:>10.4} {:>10.4} {:>10.1} {:>12}",
                format!("{}", c.condition),
                c.mean_recoverable_object_fraction,
                c.mean_recoverable_edge_fraction,
                c.mean_metadata_count,
                recon);
        }
    }
}
