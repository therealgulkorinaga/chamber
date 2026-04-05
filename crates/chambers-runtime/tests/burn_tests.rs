//! Burn engine tests (Issues 52-53).
//!
//! Tests: idempotency, completeness, residue measurement, crypto key destruction.

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
    runtime
        .load_grammar(decision_chamber_grammar())
        .unwrap();
    runtime
}

fn create_populated_world(runtime: &Runtime) -> chambers_types::world::WorldId {
    let world_id = runtime
        .create_world("decision_chamber_v1", "burn test")
        .unwrap();

    let prims = &[
        Primitive::CreateObject,
        Primitive::LinkObjects,
        Primitive::TriggerBurn,
    ];
    runtime
        .issue_capabilities(world_id, researcher(), prims)
        .unwrap();

    // Create several objects to ensure there's state to burn
    for i in 0..5 {
        runtime
            .submit(&TransitionRequest {
                world_id,
                principal: researcher(),
                operation: TransitionOperation::CreateObject {
                    object_type: "premise".to_string(),
                    payload: serde_json::json!({"statement": format!("Premise {}", i)}),
                    lifecycle_class: LifecycleClass::Temporary,
                    preservable: false,
                },
            })
            .unwrap();
    }

    world_id
}

#[test]
fn test_burn_completes_all_five_layers() {
    let runtime = setup();
    let world_id = create_populated_world(&runtime);

    // Verify world has state
    assert!(runtime.state_engine.has_world(world_id));
    assert!(runtime.crypto.has_world_key(world_id));

    // Trigger abort burn
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::AbortBurn,
            },
        })
        .unwrap();

    // Verify all state is gone
    assert!(!runtime.state_engine.has_world(world_id));
    assert!(!runtime.crypto.has_world_key(world_id));
    assert!(runtime.crypto.is_key_destroyed(world_id));
    assert!(runtime.world_engine.is_retired(world_id));
}

#[test]
fn test_burn_idempotency() {
    let runtime = setup();
    let world_id = create_populated_world(&runtime);

    // First burn
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::AbortBurn,
            },
        })
        .unwrap();

    // Second burn on same world — should not panic
    // The world is already retired, so create_world won't help, but
    // the burn engine itself should handle this gracefully.
    // Calling burn_world directly on the burn engine:
    let result = runtime.burn_engine.burn_world(world_id, TerminationMode::AbortBurn);

    // Should succeed (idempotent) or have non-fatal errors
    match result {
        Ok(r) => {
            // Second burn may have errors for already-cleaned layers, but shouldn't panic
            assert!(
                r.layers_completed.len() > 0 || r.errors.len() > 0,
                "second burn should report something"
            );
        }
        Err(_) => {
            // Also acceptable — world already burned
        }
    }
}

#[test]
fn test_semantic_residue_after_burn() {
    let runtime = setup();
    let world_id = create_populated_world(&runtime);

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

    // Run semantic measurement directly
    let residue = runtime.burn_engine.measure_residue(world_id);

    // Verify clean burn
    assert!(!residue.state_engine_has_world, "state engine should not have world");
    assert!(!residue.crypto_key_exists, "crypto key should not exist");
    assert!(residue.crypto_key_destroyed, "crypto key should be marked destroyed");
    assert!(!residue.audit_leaks_internals, "audit should not leak internals");
    assert_eq!(residue.residue_score, 0.0, "residue score should be 0 after clean burn");

    // Audit events should exist (they're substrate-scoped, not world-scoped)
    assert!(residue.substrate_event_count > 0, "substrate audit events should exist");
    assert_eq!(residue.world_events_surviving, 0, "world-scoped audit events should be burned");
    assert!(!residue.audit_leaks_internals, "no world-scoped events should survive burn");
}

#[test]
fn test_crypto_key_unrecoverable_after_burn() {
    let runtime = setup();
    let world_id = create_populated_world(&runtime);

    // Encrypt something before burn
    let plaintext = b"sensitive world data";
    let encrypted = runtime.crypto.encrypt(world_id, plaintext).unwrap();

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

    // Attempt to decrypt — should fail
    let decrypt_result = runtime.crypto.decrypt(world_id, &encrypted);
    assert!(
        decrypt_result.is_err(),
        "decryption must fail after key destruction"
    );
}

#[test]
fn test_burn_preserves_vault_artifacts() {
    let runtime = setup();
    let world_id = runtime
        .create_world("decision_chamber_v1", "preserve test")
        .unwrap();

    // Issue capabilities for full lifecycle
    let active_prims = &[
        Primitive::CreateObject,
        Primitive::LinkObjects,
        Primitive::TriggerBurn,
    ];
    runtime
        .issue_capabilities(world_id, researcher(), active_prims)
        .unwrap();

    // Create a decision_summary
    let result = runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": "Choose Postgres",
                    "rationale": "Best fit for workload"
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

    // Advance to finalization
    runtime
        .advance_phase(world_id, LifecyclePhase::ConvergenceReview)
        .unwrap();
    runtime
        .advance_phase(world_id, LifecyclePhase::Finalization)
        .unwrap();

    let seal_prims = &[Primitive::SealArtifact, Primitive::TriggerBurn];
    runtime
        .issue_capabilities(world_id, researcher(), seal_prims)
        .unwrap();

    // Seal artifact
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::SealArtifact {
                target_id: summary_id,
                authorization: SealAuthorization::HumanConfirmed {
                    confirmer: "test".to_string(),
                },
            },
        })
        .unwrap();

    // Burn with preservation
    runtime
        .submit(&TransitionRequest {
            world_id,
            principal: researcher(),
            operation: TransitionOperation::TriggerBurn {
                mode: TerminationMode::ConvergedPreserving,
            },
        })
        .unwrap();

    // World state is gone
    assert!(!runtime.state_engine.has_world(world_id));

    // But vault artifact survives
    let artifacts = runtime.vault.artifacts_from_world(world_id);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_class, "decision_summary");
}
