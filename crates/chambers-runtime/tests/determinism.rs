//! Determinism harness (Issue 42).
//!
//! Property: given the same TransitionRequest and the same world-state,
//! the interpreter produces the same substrate-level result.
//!
//! Approach: record a sequence of operations, replay on a fresh runtime,
//! compare state snapshots.

use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::*;
use chambers_types::world::LifecyclePhase;

fn researcher() -> Principal {
    Principal::new("researcher")
}

fn setup_and_load() -> Runtime {
    let mut runtime = Runtime::new();
    runtime
        .load_grammar(decision_chamber_grammar())
        .expect("grammar load");
    runtime
}

/// A recorded operation for replay.
#[derive(Clone)]
struct RecordedOp {
    operation: TransitionOperation,
}

/// Execute a sequence of operations and return the final state snapshot.
fn execute_sequence(ops: &[RecordedOp]) -> StateSnapshot {
    let runtime = setup_and_load();
    let world_id = runtime
        .create_world("decision_chamber_v1", "determinism test")
        .unwrap();

    // Issue broad capabilities
    let all_prims = &[
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
        .issue_capabilities(world_id, researcher(), all_prims)
        .unwrap();

    let mut created_ids = Vec::new();

    for op in ops {
        let request = TransitionRequest {
            world_id,
            principal: researcher(),
            operation: op.operation.clone(),
        };
        match runtime.submit(&request) {
            Ok(result) => {
                if let chambers_operation::OperationResult::ObjectCreated(id) = result {
                    created_ids.push(id);
                }
            }
            Err(e) => {
                // Record the error as part of the snapshot — determinism includes rejections
                return StateSnapshot {
                    object_count: runtime
                        .state_engine
                        .object_count(world_id)
                        .unwrap_or(0),
                    link_count: runtime
                        .state_engine
                        .link_count(world_id)
                        .unwrap_or(0),
                    object_types: get_sorted_types(&runtime, world_id),
                    error: Some(format!("{}", e)),
                };
            }
        }
    }

    StateSnapshot {
        object_count: runtime.state_engine.object_count(world_id).unwrap_or(0),
        link_count: runtime.state_engine.link_count(world_id).unwrap_or(0),
        object_types: get_sorted_types(&runtime, world_id),
        error: None,
    }
}

fn get_sorted_types(runtime: &Runtime, world_id: chambers_types::world::WorldId) -> Vec<String> {
    let objects = runtime
        .state_engine
        .all_objects_decrypted(world_id)
        .unwrap_or_default();
    let mut types: Vec<String> = objects.iter().map(|o| o.object_type.clone()).collect();
    types.sort();
    types
}

#[derive(Debug, PartialEq)]
struct StateSnapshot {
    object_count: usize,
    link_count: usize,
    object_types: Vec<String>,
    error: Option<String>,
}

#[test]
fn test_determinism_create_objects() {
    let ops = vec![
        RecordedOp {
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({"statement": "Premise A"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        },
        RecordedOp {
            operation: TransitionOperation::CreateObject {
                object_type: "constraint".to_string(),
                payload: serde_json::json!({"description": "Budget limit"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        },
        RecordedOp {
            operation: TransitionOperation::CreateObject {
                object_type: "risk".to_string(),
                payload: serde_json::json!({"description": "Vendor lock-in"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        },
    ];

    // Execute the same sequence 10 times
    let snapshots: Vec<StateSnapshot> = (0..10).map(|_| execute_sequence(&ops)).collect();

    // All snapshots must be identical
    for (i, snap) in snapshots.iter().enumerate().skip(1) {
        assert_eq!(
            snapshots[0], *snap,
            "Run 0 and run {} produced different results",
            i
        );
    }
}

#[test]
fn test_determinism_invalid_operation_rejection() {
    // Attempting to create an unknown type should be deterministically rejected
    let ops = vec![RecordedOp {
        operation: TransitionOperation::CreateObject {
            object_type: "nonexistent_type".to_string(),
            payload: serde_json::json!({"foo": "bar"}),
            lifecycle_class: LifecycleClass::Temporary,
            preservable: false,
        },
    }];

    let snapshots: Vec<StateSnapshot> = (0..5).map(|_| execute_sequence(&ops)).collect();

    for (i, snap) in snapshots.iter().enumerate().skip(1) {
        assert_eq!(
            snapshots[0], *snap,
            "Rejection must be deterministic: run 0 vs run {}",
            i
        );
    }

    // Should have an error
    assert!(
        snapshots[0].error.is_some(),
        "unknown type should produce error"
    );
}

#[test]
fn test_determinism_mixed_valid_and_invalid() {
    let ops = vec![
        RecordedOp {
            operation: TransitionOperation::CreateObject {
                object_type: "premise".to_string(),
                payload: serde_json::json!({"statement": "Valid premise"}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        },
        // This will fail — trying to seal in Active phase
        RecordedOp {
            operation: TransitionOperation::CreateObject {
                object_type: "INVALID_TYPE".to_string(),
                payload: serde_json::json!({}),
                lifecycle_class: LifecycleClass::Temporary,
                preservable: false,
            },
        },
    ];

    let snapshots: Vec<StateSnapshot> = (0..5).map(|_| execute_sequence(&ops)).collect();

    for (i, snap) in snapshots.iter().enumerate().skip(1) {
        assert_eq!(snapshots[0], *snap, "Determinism failed: run 0 vs run {}", i);
    }

    // First op succeeded (1 object), second failed
    assert_eq!(snapshots[0].object_count, 1);
    assert!(snapshots[0].error.is_some());
}

#[test]
fn test_determinism_phase_transitions() {
    // Verify that phase transition legality is deterministic
    let runtime1 = setup_and_load();
    let runtime2 = setup_and_load();

    let w1 = runtime1
        .create_world("decision_chamber_v1", "test1")
        .unwrap();
    let w2 = runtime2
        .create_world("decision_chamber_v1", "test2")
        .unwrap();

    // Valid transition
    let r1 = runtime1.advance_phase(w1, LifecyclePhase::ConvergenceReview);
    let r2 = runtime2.advance_phase(w2, LifecyclePhase::ConvergenceReview);
    assert_eq!(r1.is_ok(), r2.is_ok(), "Phase transition determinism");

    // Invalid transition (skip to Terminated from ConvergenceReview should work as abort)
    let r1 = runtime1.advance_phase(w1, LifecyclePhase::Terminated);
    let r2 = runtime2.advance_phase(w2, LifecyclePhase::Terminated);
    assert_eq!(r1.is_ok(), r2.is_ok(), "Abort transition determinism");
}
