//! End-to-end integration tests for the Decision Chamber.
//!
//! Tests both paths:
//! 1. Create → Explore → Converge → Finalize → Preserve + Burn
//! 2. Create → Explore → Abort Burn

use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::*;
use chambers_types::world::{LifecyclePhase, TerminationMode};

fn setup_runtime() -> Runtime {
    let mut runtime = Runtime::new();
    let grammar = decision_chamber_grammar();
    runtime.load_grammar(grammar).expect("grammar should load");
    runtime
}

fn researcher() -> Principal {
    Principal::new("researcher")
}

/// Issue all Active-phase capabilities for the researcher.
fn issue_active_capabilities(runtime: &Runtime, world_id: chambers_types::world::WorldId) {
    let active_prims = &[
        Primitive::CreateObject,
        Primitive::LinkObjects,
        Primitive::ChallengeObject,
        Primitive::GenerateAlternative,
        Primitive::RankSet,
        Primitive::SynthesizeSet,
        Primitive::CondenseObject,
        Primitive::TriggerBurn,
    ];
    runtime
        .issue_capabilities(world_id, researcher(), active_prims)
        .expect("capability issuance should succeed");
}

fn issue_convergence_capabilities(runtime: &Runtime, world_id: chambers_types::world::WorldId) {
    let prims = &[
        Primitive::ChallengeObject,
        Primitive::CondenseObject,
        Primitive::LinkObjects,
        Primitive::TriggerBurn,
    ];
    runtime
        .issue_capabilities(world_id, researcher(), prims)
        .expect("capability issuance should succeed");
}

fn issue_finalization_capabilities(runtime: &Runtime, world_id: chambers_types::world::WorldId) {
    let prims = &[
        Primitive::SealArtifact,
        Primitive::CondenseObject,
        Primitive::TriggerBurn,
    ];
    runtime
        .issue_capabilities(world_id, researcher(), prims)
        .expect("capability issuance should succeed");
}

#[test]
fn test_full_preserve_and_burn_path() {
    let runtime = setup_runtime();

    // === PHASE: Create World ===
    let world_id = runtime
        .create_world("decision_chamber_v1", "Choose a cloud provider")
        .expect("world creation should succeed");

    // Verify world is Active
    let world = runtime.world_engine.get_world(world_id).unwrap();
    assert_eq!(world.lifecycle_phase, LifecyclePhase::Active);

    // Issue Active-phase capabilities
    issue_active_capabilities(&runtime, world_id);

    // === PHASE: Exploration ===

    // Create objects: premise
    let premise_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({
                    "statement": "We need a cloud provider that supports HIPAA compliance.",
                    "source": "legal team"
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .expect("create premise should succeed");

    let premise_id = match premise_result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Create constraint
    let constraint_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "constraint".to_string(),
                payload: serde_json::json!({
                    "description": "Budget must not exceed $50k/month.",
                    "severity": "hard"
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .expect("create constraint should succeed");

    let _constraint_id = match constraint_result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Create alternative
    let alt_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "alternative".to_string(),
                payload: serde_json::json!({
                    "description": "AWS with HIPAA BAA",
                    "pros": "Mature, broad service catalog",
                    "cons": "Expensive, vendor lock-in"
                }),
                lifecycle_class: LifecycleClass::Intermediate,
                preservable: false,
            },
        })
        .expect("create alternative should succeed");

    let alt_id = match alt_result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Create risk
    let risk_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "risk".to_string(),
                payload: serde_json::json!({
                    "description": "Vendor lock-in increases switching costs by 3x after year 2.",
                    "likelihood": "high",
                    "impact": "medium"
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .expect("create risk should succeed");

    let risk_id = match risk_result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Link risk to alternative
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::LinkObjects {
                source_id: risk_id,
                target_id: alt_id,
                link_type: "risks".to_string(),
            },
        })
        .expect("link should succeed");

    // Create recommendation via synthesis
    let rec_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::SynthesizeSet {
                source_ids: vec![premise_id, alt_id],
                synthesis_type: "recommendation".to_string(),
                synthesis_payload: serde_json::json!({
                    "summary": "AWS with HIPAA BAA is recommended given compliance needs.",
                    "rationale": "Only provider meeting all hard constraints.",
                    "confidence": "high"
                }),
            },
        })
        .expect("synthesize should succeed");

    let rec_id = match rec_result {
        chambers_operation::OperationResult::SetSynthesized(id) => id,
        _ => panic!("expected SetSynthesized"),
    };

    // Create decision_summary (preservable)
    let summary_result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": "Select AWS with HIPAA BAA.",
                    "rationale": "Meets compliance, within budget, acceptable risk profile.",
                    "alternatives_considered": 3,
                    "risks_accepted": "Vendor lock-in mitigated by multi-year exit strategy.",
                    "constraints_satisfied": "HIPAA compliance, $50k/month budget"
                }),
                lifecycle_class: LifecycleClass::Preservable,
                preservable: true,
            },
        })
        .expect("create decision_summary should succeed");

    let summary_id = match summary_result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Link summary to recommendation
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::LinkObjects {
                source_id: summary_id,
                target_id: rec_id,
                link_type: "based_on".to_string(),
            },
        })
        .expect("link summary->recommendation should succeed");

    // Verify summary view
    let summary_view = runtime.view_engine.summary_view(world_id).unwrap();
    assert!(summary_view.object_count >= 5);
    assert!(summary_view.link_count >= 2);

    // === PHASE: Convergence Review ===
    runtime
        .advance_phase(world_id, LifecyclePhase::ConvergenceReview)
        .expect("advance to convergence should succeed");

    issue_convergence_capabilities(&runtime, world_id);

    // === PHASE: Finalization ===
    runtime
        .advance_phase(world_id, LifecyclePhase::Finalization)
        .expect("advance to finalization should succeed");

    issue_finalization_capabilities(&runtime, world_id);

    // Seal the decision_summary
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::SealArtifact {
                target_id: summary_id,
                authorization: SealAuthorization::HumanConfirmed {
                    confirmer: "lead_researcher".to_string(),
                },
            },
        })
        .expect("seal artifact should succeed");

    // Verify artifact is in vault
    assert_eq!(runtime.vault.artifact_count_for_world(world_id), 1);

    // === PHASE: Burn ===
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::ConvergedPreserving,
            },
        })
        .expect("burn should succeed");

    // === POST-BURN VERIFICATION ===

    // World is gone from state engine
    assert!(!runtime.state_engine.has_world(world_id));

    // World ID is retired
    assert!(runtime.world_engine.is_retired(world_id));

    // Artifact survives in vault
    let artifacts = runtime.vault.artifacts_from_world(world_id);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_class, "decision_summary");

    // Burn view confirms destruction
    let burn_view = runtime.view_engine.burn_view(world_id);
    assert!(burn_view.world_state_destroyed);
    assert_eq!(burn_view.artifacts_preserved, 1);

    // Audit: only Tier 1 events survive burn (WorldCreated + BurnCompleted = 2)
    let audit_events = runtime.audit.events_for_world(world_id);
    assert_eq!(audit_events.len(), 2, "only 2 substrate-scoped events should survive burn");
    // Verify the surviving events are substrate-scoped
    assert!(audit_events.iter().all(|e| e.event_type.is_substrate_scoped()),
        "all post-burn events should be substrate-scoped");

    // Crypto key is destroyed
    assert!(!runtime.crypto.has_world_key(world_id));
    assert!(runtime.crypto.is_key_destroyed(world_id));

    println!("=== PRESERVE+BURN PATH: PASSED ===");
    println!("  Objects created: 5+");
    println!("  Links created: 2+");
    println!("  Artifact preserved: 1 (decision_summary)");
    println!("  World state destroyed: true");
    println!("  Crypto key destroyed: true");
    println!("  Audit events: {}", audit_events.len());
}

#[test]
fn test_abort_burn_path() {
    let runtime = setup_runtime();

    // Create world
    let world_id = runtime
        .create_world("decision_chamber_v1", "Choose a database")
        .expect("world creation should succeed");

    issue_active_capabilities(&runtime, world_id);

    // Create some objects
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({
                    "statement": "We need a database that scales to 10M rows.",
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .expect("create premise should succeed");

    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "alternative".to_string(),
                payload: serde_json::json!({
                    "description": "PostgreSQL",
                }),
                lifecycle_class: LifecycleClass::Intermediate,
                preservable: false,
            },
        })
        .expect("create alternative should succeed");

    // Verify objects exist
    assert!(runtime.state_engine.has_world(world_id));

    // Abort burn — no convergence, no artifact
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::AbortBurn,
            },
        })
        .expect("abort burn should succeed");

    // === POST-BURN VERIFICATION ===

    // No artifact in vault
    assert_eq!(runtime.vault.artifact_count_for_world(world_id), 0);

    // World state destroyed
    assert!(!runtime.state_engine.has_world(world_id));

    // World ID retired
    assert!(runtime.world_engine.is_retired(world_id));

    // Crypto key destroyed
    assert!(runtime.crypto.is_key_destroyed(world_id));

    println!("=== ABORT BURN PATH: PASSED ===");
    println!("  Artifacts preserved: 0");
    println!("  World state destroyed: true");
    println!("  Crypto key destroyed: true");
}

#[test]
fn test_cross_world_isolation() {
    let runtime = setup_runtime();

    let world_a = runtime
        .create_world("decision_chamber_v1", "World A")
        .expect("world A creation should succeed");
    let world_b = runtime
        .create_world("decision_chamber_v1", "World B")
        .expect("world B creation should succeed");

    issue_active_capabilities(&runtime, world_a);
    issue_active_capabilities(&runtime, world_b);

    // Create object in world A
    let result = runtime
        .submit(&TransitionRequest {
            world_id: world_a,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({"statement": "World A premise"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .expect("create in world A should succeed");

    let obj_a_id = match result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Try to reference world A's object from world B — should fail
    let cross_world_result = runtime.submit(&TransitionRequest {
        world_id: world_b,
        principal: researcher(),
        operation: TransitionOperation::ChallengeObject {
            target_id: obj_a_id,
            challenge_text: "Cross-world attack".to_string(),
        },
    });

    assert!(
        cross_world_result.is_err(),
        "cross-world reference should fail"
    );

    println!("=== CROSS-WORLD ISOLATION: PASSED ===");
}

#[test]
fn test_preservation_law_enforcement() {
    let runtime = setup_runtime();

    let world_id = runtime
        .create_world("decision_chamber_v1", "Test preservation")
        .expect("world creation should succeed");

    issue_active_capabilities(&runtime, world_id);

    // Create a non-preservable object
    let result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({"statement": "A premise"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .unwrap();

    let premise_id = match result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Advance to finalization
    runtime
        .advance_phase(world_id, LifecyclePhase::ConvergenceReview)
        .unwrap();
    runtime
        .advance_phase(world_id, LifecyclePhase::Finalization)
        .unwrap();

    issue_finalization_capabilities(&runtime, world_id);

    // Try to seal a non-preservable object — should fail
    let seal_result = runtime.submit(&TransitionRequest {
        world_id,
        principal: researcher(),
        operation: TransitionOperation::SealArtifact {
            target_id: premise_id,
            authorization: SealAuthorization::HumanConfirmed {
                confirmer: "test".to_string(),
            },
        },
    });

    assert!(
        seal_result.is_err(),
        "sealing non-preservable object should fail"
    );

    println!("=== PRESERVATION LAW ENFORCEMENT: PASSED ===");
}

#[test]
fn test_lifecycle_phase_enforcement() {
    let runtime = setup_runtime();

    let world_id = runtime
        .create_world("decision_chamber_v1", "Test lifecycle")
        .expect("world creation should succeed");

    issue_active_capabilities(&runtime, world_id);

    // Create a preservable decision_summary
    let result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": "Test decision",
                    "rationale": "Test rationale"
                }),
                lifecycle_class: LifecycleClass::Preservable,
                preservable: true,
            },
        })
        .unwrap();

    let summary_id = match result {
        chambers_operation::OperationResult::ObjectCreated(id) => id,
        _ => panic!("expected ObjectCreated"),
    };

    // Try to seal in Active phase — should fail (seal only in Finalization)
    let seal_result = runtime.submit(&TransitionRequest {
        world_id,
        principal: researcher(),
        operation: TransitionOperation::SealArtifact {
            target_id: summary_id,
            authorization: SealAuthorization::HumanConfirmed {
                confirmer: "test".to_string(),
            },
        },
    });

    assert!(
        seal_result.is_err(),
        "sealing in Active phase should fail"
    );

    // Invalid phase transition: Active -> Finalization (must go through ConvergenceReview)
    let skip_result = runtime.advance_phase(world_id, LifecyclePhase::Finalization);
    assert!(
        skip_result.is_err(),
        "skipping ConvergenceReview should fail"
    );

    println!("=== LIFECYCLE PHASE ENFORCEMENT: PASSED ===");
}
