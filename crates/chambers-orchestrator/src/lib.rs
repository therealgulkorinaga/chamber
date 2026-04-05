//! Symbolic orchestrator for Chambers.
//!
//! Rules-based planner that drives Decision Chamber lifecycle
//! without any LLM dependency. Maps structured task input to
//! substrate primitives.
//!
//! All orchestrator state is either world-scoped (submitted through
//! primitives) or stateless (pure functions). No hidden scratch state.

use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::*;
use chambers_types::world::{LifecyclePhase, TerminationMode, WorldId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("substrate error: {0}")]
    Substrate(#[from] chambers_types::error::SubstrateError),
    #[error("orchestrator error: {0}")]
    Logic(String),
}

/// A structured decision task — input to the orchestrator.
#[derive(Debug, Clone)]
pub struct DecisionTask {
    pub question: String,
    pub premises: Vec<PremiseInput>,
    pub constraints: Vec<ConstraintInput>,
    pub alternatives: Vec<AlternativeInput>,
    pub risks: Vec<RiskInput>,
    pub upsides: Vec<UpsideInput>,
}

#[derive(Debug, Clone)]
pub struct PremiseInput {
    pub statement: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConstraintInput {
    pub description: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct AlternativeInput {
    pub description: String,
    pub pros: String,
    pub cons: String,
}

#[derive(Debug, Clone)]
pub struct RiskInput {
    pub description: String,
    pub likelihood: String,
    pub impact: String,
}

#[derive(Debug, Clone)]
pub struct UpsideInput {
    pub description: String,
    pub magnitude: String,
}

/// Result of a complete orchestrated run.
#[derive(Debug)]
pub struct OrchestratorResult {
    pub world_id: WorldId,
    pub mode: TerminationMode,
    pub objects_created: usize,
    pub links_created: usize,
    pub artifact_preserved: bool,
}

/// The symbolic orchestrator — drives chamber lifecycle through rules.
pub struct SymbolicOrchestrator<'a> {
    runtime: &'a Runtime,
    principal: Principal,
}

impl<'a> SymbolicOrchestrator<'a> {
    pub fn new(runtime: &'a Runtime, principal: Principal) -> Self {
        Self { runtime, principal }
    }

    /// Run a full decision chamber: create → explore → converge → finalize → preserve+burn.
    pub fn run_preserve(
        &self,
        task: &DecisionTask,
        decision: &str,
        rationale: &str,
    ) -> Result<OrchestratorResult, OrchestratorError> {
        let world_id = self.create_and_activate(task)?;
        let mut objects_created = 0;
        let mut links_created = 0;

        // === EXPLORATION PHASE ===
        self.issue_active_caps(world_id)?;

        // Create premises
        let mut premise_ids = Vec::new();
        for p in &task.premises {
            let id = self.create_object(world_id, "premise", serde_json::json!({
                "statement": p.statement,
                "source": p.source.as_deref().unwrap_or("unspecified")
            }))?;
            premise_ids.push(id);
            objects_created += 1;
        }

        // Create constraints
        let mut constraint_ids = Vec::new();
        for c in &task.constraints {
            let id = self.create_object(world_id, "constraint", serde_json::json!({
                "description": c.description,
                "severity": c.severity
            }))?;
            constraint_ids.push(id);
            objects_created += 1;
        }

        // Create alternatives
        let mut alt_ids = Vec::new();
        for a in &task.alternatives {
            let id = self.create_object(world_id, "alternative", serde_json::json!({
                "description": a.description,
                "pros": a.pros,
                "cons": a.cons
            }))?;
            alt_ids.push(id);
            objects_created += 1;
        }

        // Create risks and link to alternatives
        for (i, r) in task.risks.iter().enumerate() {
            let rid = self.create_object(world_id, "risk", serde_json::json!({
                "description": r.description,
                "likelihood": r.likelihood,
                "impact": r.impact
            }))?;
            objects_created += 1;

            // Link risk to the alternative it applies to (round-robin if more risks than alts)
            if !alt_ids.is_empty() {
                let target = alt_ids[i % alt_ids.len()];
                self.link_objects(world_id, rid, target, "risks")?;
                links_created += 1;
            }
        }

        // Create upsides and link to alternatives
        for (i, u) in task.upsides.iter().enumerate() {
            let uid = self.create_object(world_id, "upside", serde_json::json!({
                "description": u.description,
                "magnitude": u.magnitude
            }))?;
            objects_created += 1;

            if !alt_ids.is_empty() {
                let target = alt_ids[i % alt_ids.len()];
                self.link_objects(world_id, uid, target, "benefits")?;
                links_created += 1;
            }
        }

        // Synthesize a recommendation from all inputs
        let mut synth_sources = Vec::new();
        synth_sources.extend(premise_ids.iter().copied());
        synth_sources.extend(alt_ids.iter().copied());

        if !synth_sources.is_empty() {
            let rec_id = self.synthesize(
                world_id,
                &synth_sources,
                "recommendation",
                serde_json::json!({
                    "summary": decision,
                    "rationale": rationale,
                    "confidence": "high"
                }),
            )?;
            objects_created += 1;
            links_created += synth_sources.len();

            // Create decision_summary
            let summary_id = self.create_preservable_summary(
                world_id, decision, rationale, task.alternatives.len(),
            )?;
            objects_created += 1;

            // Link summary to recommendation
            self.link_objects(world_id, summary_id, rec_id, "based_on")?;
            links_created += 1;

            // === CONVERGENCE ===
            self.runtime.advance_phase(world_id, LifecyclePhase::ConvergenceReview)?;
            self.issue_convergence_caps(world_id)?;

            // === FINALIZATION ===
            self.runtime.advance_phase(world_id, LifecyclePhase::Finalization)?;
            self.issue_finalization_caps(world_id)?;

            // Seal
            self.seal_artifact(world_id, summary_id)?;

            // Burn
            self.trigger_burn(world_id, TerminationMode::ConvergedPreserving)?;

            Ok(OrchestratorResult {
                world_id,
                mode: TerminationMode::ConvergedPreserving,
                objects_created,
                links_created,
                artifact_preserved: true,
            })
        } else {
            Err(OrchestratorError::Logic("no inputs to synthesize".into()))
        }
    }

    /// Run an abort path: create → explore → abort burn (no convergence, no artifact).
    pub fn run_abort(&self, task: &DecisionTask) -> Result<OrchestratorResult, OrchestratorError> {
        let world_id = self.create_and_activate(task)?;
        let mut objects_created = 0;

        self.issue_active_caps(world_id)?;

        // Create some objects but don't converge
        for p in &task.premises {
            self.create_object(world_id, "premise", serde_json::json!({
                "statement": p.statement
            }))?;
            objects_created += 1;
        }

        // Abort
        self.trigger_burn(world_id, TerminationMode::AbortBurn)?;

        Ok(OrchestratorResult {
            world_id,
            mode: TerminationMode::AbortBurn,
            objects_created,
            links_created: 0,
            artifact_preserved: false,
        })
    }

    // --- Internal helpers ---

    fn create_and_activate(&self, task: &DecisionTask) -> Result<WorldId, OrchestratorError> {
        let world_id = self
            .runtime
            .create_world("decision_chamber_v1", &task.question)?;
        Ok(world_id)
    }

    fn issue_active_caps(&self, world_id: WorldId) -> Result<(), OrchestratorError> {
        self.runtime.issue_capabilities(
            world_id,
            self.principal.clone(),
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
        )?;
        Ok(())
    }

    fn issue_convergence_caps(&self, world_id: WorldId) -> Result<(), OrchestratorError> {
        self.runtime.issue_capabilities(
            world_id,
            self.principal.clone(),
            &[
                Primitive::ChallengeObject,
                Primitive::CondenseObject,
                Primitive::LinkObjects,
                Primitive::TriggerBurn,
            ],
        )?;
        Ok(())
    }

    fn issue_finalization_caps(&self, world_id: WorldId) -> Result<(), OrchestratorError> {
        self.runtime.issue_capabilities(
            world_id,
            self.principal.clone(),
            &[Primitive::SealArtifact, Primitive::CondenseObject, Primitive::TriggerBurn],
        )?;
        Ok(())
    }

    fn create_object(
        &self,
        world_id: WorldId,
        object_type: &str,
        payload: serde_json::Value,
    ) -> Result<chambers_types::object::ObjectId, OrchestratorError> {
        let result = self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::CreateObject {
                object_type: object_type.to_string(),
                payload,
                lifecycle_class: if object_type == "alternative" {
                    LifecycleClass::Intermediate
                } else {
                    LifecycleClass::Temporary
                },
                preservable: false,
            },
        })?;
        match result {
            chambers_operation::OperationResult::ObjectCreated(id) => Ok(id),
            _ => Err(OrchestratorError::Logic("expected ObjectCreated".into())),
        }
    }

    fn create_preservable_summary(
        &self,
        world_id: WorldId,
        decision: &str,
        rationale: &str,
        alternatives_considered: usize,
    ) -> Result<chambers_types::object::ObjectId, OrchestratorError> {
        let result = self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::CreateObject {
                object_type: "decision_summary".to_string(),
                payload: serde_json::json!({
                    "decision": decision,
                    "rationale": rationale,
                    "alternatives_considered": alternatives_considered,
                }),
                lifecycle_class: LifecycleClass::Preservable,
                preservable: true,
            },
        })?;
        match result {
            chambers_operation::OperationResult::ObjectCreated(id) => Ok(id),
            _ => Err(OrchestratorError::Logic("expected ObjectCreated".into())),
        }
    }

    fn link_objects(
        &self,
        world_id: WorldId,
        source: chambers_types::object::ObjectId,
        target: chambers_types::object::ObjectId,
        link_type: &str,
    ) -> Result<(), OrchestratorError> {
        self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::LinkObjects {
                source_id: source,
                target_id: target,
                link_type: link_type.to_string(),
            },
        })?;
        Ok(())
    }

    fn synthesize(
        &self,
        world_id: WorldId,
        source_ids: &[chambers_types::object::ObjectId],
        synthesis_type: &str,
        payload: serde_json::Value,
    ) -> Result<chambers_types::object::ObjectId, OrchestratorError> {
        let result = self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::SynthesizeSet {
                source_ids: source_ids.to_vec(),
                synthesis_type: synthesis_type.to_string(),
                synthesis_payload: payload,
            },
        })?;
        match result {
            chambers_operation::OperationResult::SetSynthesized(id) => Ok(id),
            _ => Err(OrchestratorError::Logic("expected SetSynthesized".into())),
        }
    }

    fn seal_artifact(
        &self,
        world_id: WorldId,
        target_id: chambers_types::object::ObjectId,
    ) -> Result<(), OrchestratorError> {
        self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::SealArtifact {
                target_id,
                authorization: SealAuthorization::HumanConfirmed {
                    confirmer: self.principal.0.clone(),
                },
            },
        })?;
        Ok(())
    }

    fn trigger_burn(
        &self,
        world_id: WorldId,
        mode: TerminationMode,
    ) -> Result<(), OrchestratorError> {
        self.runtime.submit(&TransitionRequest {
            world_id,
            principal: self.principal.clone(),
            operation: TransitionOperation::TriggerBurn { mode },
        })?;
        Ok(())
    }
}
