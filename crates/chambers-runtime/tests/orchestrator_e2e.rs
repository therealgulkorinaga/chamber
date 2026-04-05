//! Orchestrator end-to-end tests (Issue 69).
//!
//! Validates: symbolic planner drives full chamber lifecycle without LLM.
//! Both termination paths exercised.

use chambers_orchestrator::*;
use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::world::TerminationMode;

fn setup() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.load_grammar(decision_chamber_grammar()).unwrap();
    runtime
}

fn cloud_provider_task() -> DecisionTask {
    DecisionTask {
        question: "Which cloud provider should we select for HIPAA-compliant workloads?".into(),
        premises: vec![
            PremiseInput {
                statement: "We process protected health information (PHI).".into(),
                source: Some("compliance team".into()),
            },
            PremiseInput {
                statement: "Current infrastructure is on-premise and aging.".into(),
                source: Some("infrastructure team".into()),
            },
        ],
        constraints: vec![
            ConstraintInput {
                description: "Must support HIPAA BAA.".into(),
                severity: "hard".into(),
            },
            ConstraintInput {
                description: "Monthly spend under $50,000.".into(),
                severity: "hard".into(),
            },
        ],
        alternatives: vec![
            AlternativeInput {
                description: "AWS with HIPAA BAA".into(),
                pros: "Mature, broad services, large community".into(),
                cons: "Expensive, vendor lock-in risk".into(),
            },
            AlternativeInput {
                description: "Azure with HIPAA BAA".into(),
                pros: "Good enterprise integration, competitive pricing".into(),
                cons: "Less mature in some areas".into(),
            },
            AlternativeInput {
                description: "GCP with HIPAA BAA".into(),
                pros: "Strong data analytics, competitive pricing".into(),
                cons: "Smaller healthcare ecosystem".into(),
            },
        ],
        risks: vec![
            RiskInput {
                description: "Vendor lock-in increases switching cost 3x after year 2.".into(),
                likelihood: "high".into(),
                impact: "medium".into(),
            },
            RiskInput {
                description: "Compliance audit failure during migration.".into(),
                likelihood: "medium".into(),
                impact: "high".into(),
            },
        ],
        upsides: vec![
            UpsideInput {
                description: "40% reduction in infrastructure management overhead.".into(),
                magnitude: "high".into(),
            },
        ],
    }
}

#[test]
fn test_orchestrator_preserve_path() {
    let runtime = setup();
    let orchestrator = SymbolicOrchestrator::new(&runtime, Principal::new("orchestrator"));

    let result = orchestrator
        .run_preserve(
            &cloud_provider_task(),
            "Select AWS with HIPAA BAA.",
            "Meets all compliance requirements, within budget, acceptable risk with exit strategy.",
        )
        .expect("orchestrator preserve path should succeed");

    assert_eq!(result.mode, TerminationMode::ConvergedPreserving);
    assert!(result.artifact_preserved);
    assert!(result.objects_created >= 10, "should create many objects: {}", result.objects_created);
    assert!(result.links_created >= 3, "should create links: {}", result.links_created);

    // Verify post-burn state
    assert!(!runtime.state_engine.has_world(result.world_id));
    assert!(runtime.world_engine.is_retired(result.world_id));
    assert!(runtime.crypto.is_key_destroyed(result.world_id));

    // Verify artifact
    let artifacts = runtime.vault.artifacts_from_world(result.world_id);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_class, "decision_summary");

    // Verify residue
    let residue = runtime.burn_engine.measure_residue(result.world_id);
    assert_eq!(residue.residue_score, 0.0, "should be clean burn");

    println!("=== ORCHESTRATOR PRESERVE PATH ===");
    println!("  Objects created: {}", result.objects_created);
    println!("  Links created: {}", result.links_created);
    println!("  Artifact: {} preserved", if result.artifact_preserved { "1" } else { "0" });
    println!("  Residue score: {}", residue.residue_score);
}

#[test]
fn test_orchestrator_abort_path() {
    let runtime = setup();
    let orchestrator = SymbolicOrchestrator::new(&runtime, Principal::new("orchestrator"));

    let result = orchestrator
        .run_abort(&cloud_provider_task())
        .expect("orchestrator abort path should succeed");

    assert_eq!(result.mode, TerminationMode::AbortBurn);
    assert!(!result.artifact_preserved);
    assert!(result.objects_created >= 2, "should create some objects");

    // Verify post-burn state
    assert!(!runtime.state_engine.has_world(result.world_id));
    assert!(runtime.world_engine.is_retired(result.world_id));

    // Vault should be empty for this world
    assert_eq!(runtime.vault.artifact_count_for_world(result.world_id), 0);

    println!("=== ORCHESTRATOR ABORT PATH ===");
    println!("  Objects created: {}", result.objects_created);
    println!("  Artifact: none (aborted)");
}

#[test]
fn test_orchestrator_no_hidden_state() {
    // The orchestrator should have no state of its own after a run.
    // All state is either in the runtime (world-scoped, burned) or doesn't exist.
    let runtime = setup();
    let orchestrator = SymbolicOrchestrator::new(&runtime, Principal::new("orchestrator"));

    let result = orchestrator
        .run_preserve(
            &cloud_provider_task(),
            "Test decision",
            "Test rationale",
        )
        .unwrap();

    // After the run, the orchestrator struct has no stored state about the world.
    // It only holds a reference to the runtime and a principal.
    // The world is burned and gone.
    assert!(!runtime.state_engine.has_world(result.world_id));

    // Run another task — completely independent
    let result2 = orchestrator
        .run_abort(&cloud_provider_task())
        .unwrap();

    assert_ne!(
        result.world_id, result2.world_id,
        "each run should use a fresh world"
    );
}
