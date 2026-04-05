//! Grammar semantics tests (Issue 66) and vault audit leakage review (Issue 62).
//!
//! Tests: contradiction blocking, preservation conditions, mandatory object resolution,
//! vault residue boundary, audit log content.

use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::*;
use chambers_types::world::{LifecyclePhase, TerminationMode};

fn researcher() -> Principal {
    Principal::new("researcher")
}

fn setup() -> Runtime {
    let mut runtime = Runtime::new();
    runtime.load_grammar(decision_chamber_grammar()).unwrap();
    runtime
}

fn create_world_with_caps(runtime: &Runtime) -> chambers_types::world::WorldId {
    let world_id = runtime
        .create_world("decision_chamber_v1", "grammar test")
        .unwrap();
    runtime
        .issue_capabilities(
            world_id,
            researcher(),
            &[
                Primitive::CreateObject,
                Primitive::LinkObjects,
                Primitive::ChallengeObject,
                Primitive::GenerateAlternative,
                Primitive::RankSet,
                Primitive::SynthesizeSet,
                Primitive::CondenseObject,
                Primitive::TriggerBurn,
            ],
        )
        .unwrap();
    world_id
}

// --- Contradiction blocking ---

#[test]
fn test_unresolved_contradictions_detected() {
    let runtime = setup();
    let world_id = create_world_with_caps(&runtime);

    // Create a contradiction (unresolved by default)
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "contradiction".to_string(),
                payload: serde_json::json!({
                    "description": "Premise A contradicts Premise B",
                    "resolved": false
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .unwrap();

    // Refresh convergence state
    runtime
        .state_engine
        .refresh_convergence(world_id, &["decision_summary".to_string()], true, true)
        .unwrap();

    // Check for unresolved challenges/contradictions
    let conv = runtime
        .state_engine
        .with_convergence(world_id, |c| c.clone())
        .unwrap();

    assert!(
        !conv.unresolved_contradictions.is_empty(),
        "should detect unresolved contradiction"
    );
    assert_eq!(
        conv.convergence_validated,
        Some(false),
        "convergence should fail with unresolved contradiction"
    );
    assert!(
        conv.validation_failure_reason
            .as_ref()
            .unwrap()
            .contains("contradiction"),
        "failure reason should mention contradictions"
    );
}

#[test]
fn test_resolved_contradictions_dont_block() {
    let runtime = setup();
    let world_id = create_world_with_caps(&runtime);

    // Create a resolved contradiction
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "contradiction".to_string(),
                payload: serde_json::json!({
                    "description": "Was contradictory but resolved",
                    "resolved": true
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .unwrap();

    // Also need a decision_summary for mandatory type check
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": "Test",
                    "rationale": "Test rationale"
                }),
                lifecycle_class: LifecycleClass::Preservable,
                preservable: true,
            },
        })
        .unwrap();

    runtime
        .state_engine
        .refresh_convergence(world_id, &["decision_summary".to_string()], true, true)
        .unwrap();

    let conv = runtime
        .state_engine
        .with_convergence(world_id, |c| c.clone())
        .unwrap();

    assert!(
        conv.unresolved_contradictions.is_empty(),
        "resolved contradiction should not block"
    );
    assert_eq!(conv.convergence_validated, Some(true));
}

// --- Mandatory object resolution ---

#[test]
fn test_missing_mandatory_type_blocks_convergence() {
    let runtime = setup();
    let world_id = create_world_with_caps(&runtime);

    // Create only premises, no decision_summary
    runtime
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

    runtime
        .state_engine
        .refresh_convergence(world_id, &["decision_summary".to_string()], true, true)
        .unwrap();

    let conv = runtime
        .state_engine
        .with_convergence(world_id, |c| c.clone())
        .unwrap();

    assert_eq!(conv.convergence_validated, Some(false));
    assert!(conv
        .validation_failure_reason
        .as_ref()
        .unwrap()
        .contains("missing required"));
}

// --- Preservation law ---

#[test]
fn test_only_decision_summary_is_preservable() {
    let runtime = setup();

    // Check every type in the grammar
    let non_preservable = [
        "decision_objective", "premise", "support_statement", "constraint",
        "risk", "upside", "contradiction", "alternative", "recommendation",
    ];

    for t in &non_preservable {
        let can_preserve = runtime
            .policy_engine
            .can_preserve_object("decision_chamber_v1", t)
            .unwrap();
        assert!(!can_preserve, "{} should NOT be preservable", t);
    }

    let can_preserve = runtime
        .policy_engine
        .can_preserve_object("decision_chamber_v1", "decision_summary")
        .unwrap();
    assert!(can_preserve, "decision_summary MUST be preservable");
}

// --- Epoch capability narrowing ---

#[test]
fn test_create_object_blocked_in_finalization() {
    let runtime = setup();
    let world_id = create_world_with_caps(&runtime);

    // Advance to finalization
    runtime
        .advance_phase(world_id, LifecyclePhase::ConvergenceReview)
        .unwrap();
    runtime
        .advance_phase(world_id, LifecyclePhase::Finalization)
        .unwrap();

    // Issue finalization-only caps
    runtime
        .issue_capabilities(
            world_id,
            researcher(),
            &[Primitive::SealArtifact, Primitive::CondenseObject, Primitive::TriggerBurn],
        )
        .unwrap();

    // Try to create an object in Finalization — should fail (not in phase_primitives)
    let result = runtime.submit(&TransitionRequest {
        world_id,
        principal: researcher(),
        operation: TransitionOperation::CreateObject {
            object_type: "premise".to_string(),
            payload: serde_json::json!({"statement": "Late premise"}),
            lifecycle_class: LifecycleClass::Temporary,
            preservable: false,
        },
    });

    assert!(result.is_err(), "create_object should be blocked in Finalization");
}

// --- Vault audit leakage review (Issue 62) ---

#[test]
fn test_vault_contains_no_world_internals() {
    let runtime = setup();
    let world_id = runtime
        .create_world("decision_chamber_v1", "vault leakage test")
        .unwrap();

    runtime
        .issue_capabilities(
            world_id,
            researcher(),
            &[Primitive::CreateObject, Primitive::TriggerBurn],
        )
        .unwrap();

    // Create many objects — these are world internals
    for i in 0..10 {
        runtime
            .submit(&TransitionRequest {
                world_id,
                principal: researcher(),
                operation: TransitionOperation::CreateObject {
                    object_type: "premise".to_string(),
                    payload: serde_json::json!({"statement": format!("Internal premise {}", i)}),
                    lifecycle_class: LifecycleClass::Temporary,
                    preservable: false,
                },
            })
            .unwrap();
    }

    // Create and seal a decision_summary
    let result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": "Final decision",
                    "rationale": "Rationale"
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

    runtime
        .advance_phase(world_id, LifecyclePhase::ConvergenceReview)
        .unwrap();
    runtime
        .advance_phase(world_id, LifecyclePhase::Finalization)
        .unwrap();

    runtime
        .issue_capabilities(
            world_id,
            researcher(),
            &[Primitive::SealArtifact, Primitive::TriggerBurn],
        )
        .unwrap();

    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::SealArtifact {
                target_id: summary_id,
                authorization: SealAuthorization::HumanConfirmed {
                    confirmer: "reviewer".to_string(),
                },
            },
        })
        .unwrap();

    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::ConvergedPreserving,
            },
        })
        .unwrap();

    // === VAULT LEAKAGE REVIEW ===
    let artifacts = runtime.vault.artifacts_from_world(world_id);
    assert_eq!(artifacts.len(), 1, "exactly one artifact should survive");

    let artifact = &artifacts[0];

    // Artifact should contain ONLY the decision_summary payload
    assert_eq!(artifact.artifact_class, "decision_summary");

    // Provenance metadata should be minimal — no world internals
    let prov = &artifact.provenance_metadata;
    // No internal object references, no link structure, no intermediate reasoning
    assert!(
        !serde_json::to_string(prov).unwrap().contains("premise"),
        "provenance should not reference internal objects"
    );
    assert!(
        !serde_json::to_string(prov).unwrap().contains("Internal premise"),
        "provenance should not contain internal payloads"
    );
}

#[test]
fn test_audit_log_contains_no_payloads() {
    let runtime = setup();
    let world_id = create_world_with_caps(&runtime);

    // Create objects with distinctive payload content
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({
                    "statement": "SUPER_SECRET_PAYLOAD_CONTENT_12345"
                }),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        })
        .unwrap();

    // Burn
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::AbortBurn,
            },
        })
        .unwrap();

    // Check audit log for payload leakage
    let events = runtime.audit.events_for_world(world_id);
    let serialized = serde_json::to_string(&events).unwrap();

    assert!(
        !serialized.contains("SUPER_SECRET_PAYLOAD_CONTENT_12345"),
        "audit log must not contain object payloads"
    );
    assert!(
        !serialized.contains("SUPER_SECRET"),
        "audit log must not leak any world-internal content"
    );

    // After burn, only Tier 1 events survive (WorldCreated + BurnCompleted)
    assert_eq!(events.len(), 2, "only 2 substrate-scoped events should survive burn");
}
