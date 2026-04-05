//! Chambers CLI — research instrument for Phase 0.
//!
//! Not a product. A tool for researchers to:
//! - Create chambers, submit primitives, inspect views, trigger burn.
//! - Run scripted decision tasks via the symbolic orchestrator.
//! - Inspect post-burn residue.

use chambers_orchestrator::*;
use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::*;
use chambers_types::world::{LifecyclePhase, TerminationMode, WorldId};
use std::io::{self, BufRead, Write};

fn main() {
    let mut runtime = Runtime::new();
    runtime
        .load_grammar(decision_chamber_grammar())
        .expect("failed to load Decision Chamber grammar");

    println!("=== Chambers Phase 0 — Research CLI ===");
    println!("Type 'help' for commands.\n");

    let mut active_world: Option<WorldId> = None;
    let principal = Principal::new("researcher");

    let stdin = io::stdin();
    loop {
        if let Some(wid) = active_world {
            match runtime.world_engine.get_world(wid) {
                Ok(w) => print!("[{} {:?} e{}] > ", &wid.to_string()[..8], w.lifecycle_phase, w.epoch),
                Err(_) => {
                    active_world = None;
                    print!("[no world] > ");
                }
            }
        } else {
            print!("[no world] > ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "help" => print_help(),
            "quit" | "exit" => break,

            "create" => {
                let objective = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    "Interactive decision session".to_string()
                };
                match runtime.create_world("decision_chamber_v1", &objective) {
                    Ok(wid) => {
                        println!("World created: {}", wid);
                        // Auto-issue Active capabilities
                        let _ = runtime.issue_capabilities(
                            wid,
                            principal.clone(),
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
                        );
                        active_world = Some(wid);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }

            "add" if parts.len() >= 3 => {
                if let Some(wid) = active_world {
                    let obj_type = parts[1];
                    let content = parts[2..].join(" ");
                    let payload = serde_json::json!({"statement": content, "description": content});
                    let preservable = obj_type == "decision_summary";
                    let lc = if preservable {
                        LifecycleClass::Preservable
                    } else if obj_type == "alternative" {
                        LifecycleClass::Intermediate
                    } else {
                        LifecycleClass::Temporary
                    };
                    match runtime.submit(&TransitionRequest {
                        world_id: wid,
                        principal: principal.clone(),
                        operation: TransitionOperation::CreateObject {
                            object_type: obj_type.to_string(),
                            payload,
                            lifecycle_class: lc,
                            preservable,
                        },
                    }) {
                        Ok(chambers_operation::OperationResult::ObjectCreated(id)) => {
                            println!("Created {} [{}]", obj_type, &id.to_string()[..8]);
                        }
                        Ok(_) => println!("OK"),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("No active world. Use 'create' first.");
                }
            }

            "view" => {
                if let Some(wid) = active_world {
                    let view_type = parts.get(1).copied().unwrap_or("summary");
                    match view_type {
                        "summary" => match runtime.view_engine.summary_view(wid) {
                            Ok(v) => {
                                println!("Objects: {}, Links: {}", v.object_count, v.link_count);
                                for (t, c) in &v.type_counts {
                                    println!("  {}: {}", t, c);
                                }
                                if v.has_unresolved_challenges {
                                    println!("  [!] Unresolved challenges");
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        },
                        "graph" => match runtime.view_engine.graph_view(wid) {
                            Ok(v) => {
                                println!("Nodes:");
                                for n in &v.nodes {
                                    println!("  [{}] {} ({}{})", &n.id[..8], n.object_type, n.lifecycle_class,
                                        if n.preservable { ", preservable" } else { "" });
                                }
                                println!("Edges:");
                                for e in &v.edges {
                                    println!("  {} --{}-> {}", &e.source[..8], e.link_type, &e.target[..8]);
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        },
                        "conversation" => match runtime.view_engine.conversation_view(wid) {
                            Ok(v) => {
                                for entry in &v.entries {
                                    println!("  [{}] {}: {}", &entry.object_id[..8], entry.object_type, entry.payload_summary);
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        },
                        "burn" => {
                            let v = runtime.view_engine.burn_view(wid);
                            println!("Artifacts preserved: {}", v.artifacts_preserved);
                            println!("Artifact classes: {:?}", v.artifact_classes);
                            println!("World state destroyed: {}", v.world_state_destroyed);
                        }
                        _ => println!("Unknown view: {}. Use: summary, graph, conversation, burn", view_type),
                    }
                } else {
                    println!("No active world.");
                }
            }

            "advance" if parts.len() >= 2 => {
                if let Some(wid) = active_world {
                    let phase = match parts[1] {
                        "convergence" => LifecyclePhase::ConvergenceReview,
                        "finalization" => LifecyclePhase::Finalization,
                        "active" => LifecyclePhase::Active,
                        _ => {
                            println!("Unknown phase. Use: active, convergence, finalization");
                            continue;
                        }
                    };
                    match runtime.advance_phase(wid, phase) {
                        Ok(()) => {
                            println!("Advanced to {:?}", phase);
                            // Re-issue capabilities for new phase
                            let prims = match phase {
                                LifecyclePhase::ConvergenceReview => vec![
                                    Primitive::ChallengeObject, Primitive::CondenseObject,
                                    Primitive::LinkObjects, Primitive::TriggerBurn,
                                ],
                                LifecyclePhase::Finalization => vec![
                                    Primitive::SealArtifact, Primitive::CondenseObject, Primitive::TriggerBurn,
                                ],
                                _ => vec![
                                    Primitive::CreateObject, Primitive::LinkObjects,
                                    Primitive::ChallengeObject, Primitive::TriggerBurn,
                                ],
                            };
                            let _ = runtime.issue_capabilities(wid, principal.clone(), &prims);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("No active world.");
                }
            }

            "seal" if parts.len() >= 2 => {
                if let Some(wid) = active_world {
                    let obj_id_prefix = parts[1];
                    // Find object by prefix (decrypt all to search)
                    match runtime.state_engine.all_objects_decrypted(wid) {
                        Ok(objects) => {
                            let found = objects.iter().find(|o| o.object_id.to_string().starts_with(obj_id_prefix));
                            if let Some(obj) = found {
                                let oid = obj.object_id;
                                match runtime.submit(&TransitionRequest {
                                    world_id: wid,
                                    principal: principal.clone(),
                                    operation: TransitionOperation::SealArtifact {
                                        target_id: oid,
                                        authorization: SealAuthorization::HumanConfirmed {
                                            confirmer: "researcher".to_string(),
                                        },
                                    },
                                }) {
                                    Ok(_) => println!("Artifact sealed."),
                                    Err(e) => println!("Error: {}", e),
                                }
                            } else {
                                println!("No object found with prefix '{}'", obj_id_prefix);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("No active world.");
                }
            }

            "burn" => {
                if let Some(wid) = active_world {
                    let mode = match parts.get(1).copied() {
                        Some("abort") => TerminationMode::AbortBurn,
                        Some("preserve") => TerminationMode::ConvergedPreserving,
                        Some("total") => TerminationMode::ConvergedTotalBurn,
                        _ => {
                            println!("Usage: burn <abort|preserve|total>");
                            continue;
                        }
                    };
                    match runtime.submit(&TransitionRequest {
                        world_id: wid,
                        principal: principal.clone(),
                        operation: TransitionOperation::TriggerBurn { mode },
                    }) {
                        Ok(_) => {
                            println!("Burn complete ({:?}).", mode);
                            // Show residue
                            let residue = runtime.burn_engine.measure_residue(wid);
                            println!("  Residue score: {}", residue.residue_score);
                            println!("  State engine has world: {}", residue.state_engine_has_world);
                            println!("  Crypto key destroyed: {}", residue.crypto_key_destroyed);
                            println!("  Audit events: {}", residue.substrate_event_count);
                            // Show burn view
                            let bv = runtime.view_engine.burn_view(wid);
                            println!("  Artifacts preserved: {}", bv.artifacts_preserved);
                            active_world = None;
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("No active world.");
                }
            }

            "run" => {
                let mode = parts.get(1).copied().unwrap_or("preserve");
                let orchestrator = SymbolicOrchestrator::new(&runtime, principal.clone());
                let task = sample_task();
                match mode {
                    "preserve" => {
                        match orchestrator.run_preserve(&task, "Select AWS with HIPAA BAA.", "Best compliance fit.") {
                            Ok(r) => {
                                println!("Orchestrator completed ({:?}).", r.mode);
                                println!("  Objects: {}, Links: {}, Artifact: {}", r.objects_created, r.links_created, r.artifact_preserved);
                                let residue = runtime.burn_engine.measure_residue(r.world_id);
                                println!("  Residue score: {}", residue.residue_score);
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    "abort" => {
                        match orchestrator.run_abort(&task) {
                            Ok(r) => {
                                println!("Orchestrator completed ({:?}).", r.mode);
                                println!("  Objects: {}, Artifact: {}", r.objects_created, r.artifact_preserved);
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    _ => println!("Usage: run <preserve|abort>"),
                }
            }

            "audit" => {
                if let Some(wid) = active_world {
                    let events = runtime.audit.events_for_world(wid);
                    for e in &events {
                        println!("  [{}] {:?}", e.timestamp.format("%H:%M:%S"), e.event_type);
                    }
                } else {
                    println!("No active world. Showing all events:");
                    let events = runtime.audit.all_events();
                    for e in events.iter().take(20) {
                        println!("  [{}] {} {:?}", e.timestamp.format("%H:%M:%S"), &e.world_id.to_string()[..8], e.event_type);
                    }
                    if events.len() > 20 {
                        println!("  ... {} more events", events.len() - 20);
                    }
                }
            }

            "vault" => {
                let artifacts = runtime.vault.all_artifacts();
                if artifacts.is_empty() {
                    println!("Vault is empty.");
                } else {
                    for a in &artifacts {
                        println!("  [{}] class={} from_world={} sealed={}",
                            &a.artifact_id.0.to_string()[..8],
                            a.artifact_class,
                            &a.source_world_id.to_string()[..8],
                            a.sealed_at.format("%Y-%m-%d %H:%M:%S"));
                    }
                }
            }

            _ => println!("Unknown command. Type 'help'."),
        }
    }
}

fn print_help() {
    println!("Commands:");
    println!("  create [objective]           Create a new Decision Chamber world");
    println!("  add <type> <content>         Create an object (premise, constraint, risk, upside, alternative, decision_summary)");
    println!("  view [summary|graph|conversation|burn]  Inspect world state");
    println!("  advance <convergence|finalization>      Advance lifecycle phase");
    println!("  seal <object-id-prefix>      Seal a preservable object as artifact");
    println!("  burn <abort|preserve|total>  Trigger burn");
    println!("  run <preserve|abort>         Run scripted decision task via orchestrator");
    println!("  audit                        Show audit log");
    println!("  vault                        Show vault contents");
    println!("  help                         This message");
    println!("  quit                         Exit");
}

fn sample_task() -> DecisionTask {
    DecisionTask {
        question: "Which cloud provider for HIPAA workloads?".into(),
        premises: vec![
            PremiseInput { statement: "We process PHI.".into(), source: Some("compliance".into()) },
            PremiseInput { statement: "Current infra is aging.".into(), source: Some("infra team".into()) },
        ],
        constraints: vec![
            ConstraintInput { description: "HIPAA BAA required.".into(), severity: "hard".into() },
            ConstraintInput { description: "Under $50k/month.".into(), severity: "hard".into() },
        ],
        alternatives: vec![
            AlternativeInput { description: "AWS".into(), pros: "Mature".into(), cons: "Expensive".into() },
            AlternativeInput { description: "Azure".into(), pros: "Enterprise".into(), cons: "Less mature".into() },
            AlternativeInput { description: "GCP".into(), pros: "Analytics".into(), cons: "Smaller ecosystem".into() },
        ],
        risks: vec![
            RiskInput { description: "Vendor lock-in".into(), likelihood: "high".into(), impact: "medium".into() },
        ],
        upsides: vec![
            UpsideInput { description: "40% less ops overhead".into(), magnitude: "high".into() },
        ],
    }
}
