//! Lifecycle comprehension study tooling (Issues 86-89).
//!
//! Harness for measuring user prediction accuracy:
//! "What will survive? What will be destroyed?"
//!
//! Works for both Chambers and VM baseline conditions.

use crate::metrics::{ComprehensionMetrics, Condition};
use crate::task::BenchmarkTask;
use serde::{Deserialize, Serialize};

/// A comprehension test scenario presented to a participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensionScenario {
    pub scenario_id: String,
    pub condition: Condition,
    pub task_description: String,
    /// Object types that existed during the session.
    pub object_types_present: Vec<ObjectDescription>,
    /// What actually survives after termination.
    pub actual_survivors: Vec<String>,
    /// What is actually destroyed.
    pub actual_destroyed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDescription {
    pub object_type: String,
    pub count: usize,
    pub description: String,
}

/// A participant's predictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPrediction {
    pub participant_id: String,
    pub scenario_id: String,
    /// Object types the participant thinks will survive.
    pub predicted_survivors: Vec<String>,
    /// Object types the participant thinks will be destroyed.
    pub predicted_destroyed: Vec<String>,
    /// Confidence level (1-5 Likert scale).
    pub confidence: u8,
}

/// Generate a Chambers comprehension scenario from the benchmark task.
pub fn chambers_scenario(task: &BenchmarkTask) -> ComprehensionScenario {
    let object_types = vec![
        ObjectDescription {
            object_type: "premise".into(),
            count: task.premises.len(),
            description: "Factual statements grounding the decision.".into(),
        },
        ObjectDescription {
            object_type: "constraint".into(),
            count: task.constraints.len(),
            description: "Hard/soft constraints on the decision.".into(),
        },
        ObjectDescription {
            object_type: "alternative".into(),
            count: task.alternatives.len(),
            description: "Options being evaluated.".into(),
        },
        ObjectDescription {
            object_type: "risk".into(),
            count: task.risks.len(),
            description: "Identified risks for each alternative.".into(),
        },
        ObjectDescription {
            object_type: "upside".into(),
            count: task.upsides.len(),
            description: "Potential benefits of each alternative.".into(),
        },
        ObjectDescription {
            object_type: "recommendation".into(),
            count: 1,
            description: "Synthesized recommendation from analysis.".into(),
        },
        ObjectDescription {
            object_type: "decision_summary".into(),
            count: 1,
            description: "Final sealed decision artifact.".into(),
        },
    ];

    ComprehensionScenario {
        scenario_id: format!("chambers-{}", task.task_id),
        condition: Condition::Chambers,
        task_description: format!(
            "A Decision Chamber was used to answer: \"{}\". \
             The chamber went through Active → Convergence → Finalization → Preserve+Burn. \
             The preservation law states: only 'decision_summary' may survive.",
            task.question
        ),
        object_types_present: object_types,
        actual_survivors: vec!["decision_summary".into()],
        actual_destroyed: vec![
            "premise".into(),
            "constraint".into(),
            "alternative".into(),
            "risk".into(),
            "upside".into(),
            "recommendation".into(),
        ],
    }
}

/// Generate a disposable VM comprehension scenario.
pub fn vm_scenario(task: &BenchmarkTask) -> ComprehensionScenario {
    let object_types = vec![
        ObjectDescription {
            object_type: "premise files".into(),
            count: task.premises.len(),
            description: "JSON files containing premise data.".into(),
        },
        ObjectDescription {
            object_type: "constraint files".into(),
            count: task.constraints.len(),
            description: "JSON files containing constraints.".into(),
        },
        ObjectDescription {
            object_type: "alternative files".into(),
            count: task.alternatives.len(),
            description: "JSON files for each alternative.".into(),
        },
        ObjectDescription {
            object_type: "risk files".into(),
            count: task.risks.len(),
            description: "JSON files for identified risks.".into(),
        },
        ObjectDescription {
            object_type: "reasoning log".into(),
            count: 1,
            description: "Text log of the reasoning process.".into(),
        },
        ObjectDescription {
            object_type: "decision output".into(),
            count: 1,
            description: "Final decision JSON file.".into(),
        },
    ];

    ComprehensionScenario {
        scenario_id: format!("vm-{}", task.task_id),
        condition: Condition::DisposableVM,
        task_description: format!(
            "A disposable VM was used to perform the same decision task: \"{}\". \
             Files were created for each step. The VM was then destroyed (deleted).",
            task.question
        ),
        object_types_present: object_types,
        actual_survivors: vec![], // VM delete removes all files
        actual_destroyed: vec![
            "premise files".into(),
            "constraint files".into(),
            "alternative files".into(),
            "risk files".into(),
            "reasoning log".into(),
            "decision output".into(),
        ],
    }
}

/// Score a participant's predictions against reality.
pub fn score_prediction(
    scenario: &ComprehensionScenario,
    prediction: &ParticipantPrediction,
) -> ComprehensionMetrics {
    let mut metrics = ComprehensionMetrics {
        condition: scenario.condition,
        participant_id: prediction.participant_id.clone(),
        predicted_survivors: prediction.predicted_survivors.clone(),
        actual_survivors: scenario.actual_survivors.clone(),
        predicted_destroyed: prediction.predicted_destroyed.clone(),
        actual_destroyed: scenario.actual_destroyed.clone(),
        precision: 0.0,
        recall: 0.0,
        f1: 0.0,
    };
    metrics.compute_scores();
    metrics
}

/// NASA-TLX style cognitive load questionnaire (Issue 89).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveLoadResponse {
    pub participant_id: String,
    pub scenario_id: String,
    /// Mental demand (1-21 scale, NASA-TLX).
    pub mental_demand: u8,
    /// Temporal demand.
    pub temporal_demand: u8,
    /// Performance (self-assessed).
    pub performance: u8,
    /// Effort.
    pub effort: u8,
    /// Frustration.
    pub frustration: u8,
}

impl CognitiveLoadResponse {
    /// Compute raw TLX score (average of subscales).
    pub fn raw_tlx(&self) -> f64 {
        (self.mental_demand as f64
            + self.temporal_demand as f64
            + self.performance as f64
            + self.effort as f64
            + self.frustration as f64)
            / 5.0
    }
}

/// Print a comprehension scenario for a participant (text-based).
pub fn print_scenario(scenario: &ComprehensionScenario) {
    println!("═══ Comprehension Test: {} ═══", scenario.scenario_id);
    println!();
    println!("Condition: {}", scenario.condition);
    println!();
    println!("Description:");
    println!("  {}", scenario.task_description);
    println!();
    println!("Objects present during the session:");
    for obj in &scenario.object_types_present {
        println!("  - {} ({}x): {}", obj.object_type, obj.count, obj.description);
    }
    println!();
    println!("Questions:");
    println!("  1. Which object types do you think SURVIVE after termination?");
    println!("  2. Which object types do you think are DESTROYED?");
    println!("  3. Confidence (1=very low, 5=very high)?");
}
