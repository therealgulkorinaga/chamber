//! Chambers runtime — top-level substrate integration.
//!
//! Wires together all engines and provides the public API
//! for creating worlds, submitting operations, and triggering burn.

pub mod grammar_loader;

use chambers_audit::AuditLog;
use chambers_burn::BurnEngine;
use chambers_capability::CapabilitySystem;
use chambers_crypto::CryptoProvider;
use chambers_interpreter::{Interpreter, WorldContext};
use chambers_object::ObjectEngine;
use chambers_operation::{OperationEngine, OperationResult};
use chambers_policy::PolicyEngine;
use chambers_state::StateEngine;
use chambers_types::capability::Principal;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::grammar::ChamberGrammar;
use chambers_types::primitive::*;
use chambers_types::world::*;
use chambers_vault::ArtifactVault;
use chambers_view::ViewEngine;
use std::sync::Arc;

/// The Chambers substrate runtime.
pub struct Runtime {
    pub world_engine: chambers_world::WorldEngine,
    pub object_engine: ObjectEngine,
    pub policy_engine: Arc<PolicyEngine>,
    pub capability_system: Arc<CapabilitySystem>,
    pub state_engine: Arc<StateEngine>,
    pub operation_engine: Arc<OperationEngine>,
    pub interpreter: Interpreter,
    pub burn_engine: BurnEngine,
    pub vault: Arc<ArtifactVault>,
    pub audit: Arc<AuditLog>,
    pub view_engine: ViewEngine,
    pub crypto: Arc<CryptoProvider>,
}

impl Runtime {
    /// Create a new Chambers runtime.
    pub fn new() -> Self {
        let crypto = Arc::new(CryptoProvider::new());
        let audit = Arc::new(AuditLog::new());
        let state = Arc::new(StateEngine::new(crypto.clone()));
        let capability = Arc::new(CapabilitySystem::new());
        let vault = Arc::new(ArtifactVault::new());
        let policy = Arc::new(PolicyEngine::new());

        let operation = Arc::new(OperationEngine::new(
            state.clone(),
            vault.clone(),
            audit.clone(),
        ));

        let interpreter = Interpreter::new(
            policy.clone(),
            capability.clone(),
            state.clone(),
            operation.clone(),
            audit.clone(),
        );

        let burn_engine = BurnEngine::new(
            crypto.clone(),
            state.clone(),
            capability.clone(),
            audit.clone(),
        );

        let world_engine = chambers_world::WorldEngine::new(crypto.clone(), audit.clone());
        let view_engine = ViewEngine::new(state.clone(), vault.clone());

        Self {
            world_engine,
            object_engine: ObjectEngine::new(),
            policy_engine: policy,
            capability_system: capability,
            state_engine: state,
            operation_engine: operation,
            interpreter,
            burn_engine,
            vault,
            audit,
            view_engine,
            crypto,
        }
    }

    /// Load a grammar into the runtime.
    pub fn load_grammar(&mut self, grammar: ChamberGrammar) -> SubstrateResult<()> {
        self.object_engine
            .register_schemas(grammar.object_types.clone());
        self.policy_engine.load_grammar(grammar)
    }

    /// Create a new world.
    pub fn create_world(
        &self,
        grammar_id: &str,
        objective: &str,
    ) -> SubstrateResult<WorldId> {
        // Verify grammar exists
        self.policy_engine.get_grammar(grammar_id)?;

        let world_id = self.world_engine.create_world(
            grammar_id.to_string(),
            objective.to_string(),
        )?;

        // Initialize state for this world
        self.state_engine.create_world_state(world_id);

        // Advance to Active phase
        self.world_engine
            .advance_phase(world_id, LifecyclePhase::Active)?;

        Ok(world_id)
    }

    /// Submit a transition request.
    pub fn submit(&self, request: &TransitionRequest) -> SubstrateResult<OperationResult> {
        let world = self.world_engine.get_world(request.world_id)?;
        let ctx = WorldContext {
            world_id: world.world_id,
            grammar_id: world.grammar_id.clone(),
            phase: world.lifecycle_phase,
            epoch: world.epoch,
        };

        // Special handling for TriggerBurn
        if let TransitionOperation::TriggerBurn { mode } = &request.operation {
            return self.execute_burn(world.world_id, *mode, &world.grammar_id);
        }

        self.interpreter.submit(request, &ctx)
    }

    /// Issue capabilities for the current phase.
    pub fn issue_capabilities(
        &self,
        world_id: WorldId,
        principal: Principal,
        primitives: &[Primitive],
    ) -> SubstrateResult<()> {
        let world = self.world_engine.get_world(world_id)?;
        let object_types: Vec<String> = self
            .policy_engine
            .get_grammar(&world.grammar_id)?
            .object_types
            .keys()
            .cloned()
            .collect();

        self.capability_system.issue_phase_capabilities(
            world_id,
            world.epoch,
            principal,
            primitives,
            object_types,
        );
        Ok(())
    }

    /// Advance world to next lifecycle phase.
    pub fn advance_phase(
        &self,
        world_id: WorldId,
        target: LifecyclePhase,
    ) -> SubstrateResult<()> {
        let world = self.world_engine.get_world(world_id)?;

        // Invalidate old epoch capabilities
        let old_epoch = world.epoch;
        self.capability_system
            .invalidate_epoch(world_id, old_epoch);

        self.world_engine.advance_phase(world_id, target)
    }

    /// Get legal actions for the current lifecycle phase of a world.
    /// Reads from substrate grammar — adapter must not compute this.
    pub fn get_legal_actions(&self, world_id: WorldId) -> SubstrateResult<Vec<Primitive>> {
        let world = self.world_engine.get_world(world_id)?;
        let grammar = self.policy_engine.get_grammar(&world.grammar_id)?;
        let phase_key = chambers_types::grammar::LifecyclePhaseKey::from(world.lifecycle_phase);
        Ok(grammar
            .phase_primitives
            .get(&phase_key)
            .cloned()
            .unwrap_or_default())
    }

    /// Get convergence readiness state for a world.
    pub fn get_convergence_state(
        &self,
        world_id: WorldId,
    ) -> SubstrateResult<chambers_state::ConvergenceReviewState> {
        let world = self.world_engine.get_world(world_id)?;
        let grammar = self.policy_engine.get_grammar(&world.grammar_id)?;
        let criteria = &grammar.convergence_criteria;

        self.state_engine.refresh_convergence(
            world_id,
            &criteria.required_types,
            criteria.challenges_block_convergence,
            criteria.contradictions_block_convergence,
        )?;
        self.state_engine.with_convergence(world_id, |c| c.clone())
    }

    /// List available grammar IDs.
    pub fn list_grammars(&self) -> Vec<String> {
        // For Phase 0/Level 1, only one grammar exists
        self.policy_engine
            .get_grammar("decision_chamber_v1")
            .map(|g| vec![g.grammar_id])
            .unwrap_or_default()
    }

    fn execute_burn(
        &self,
        world_id: WorldId,
        mode: TerminationMode,
        grammar_id: &str,
    ) -> SubstrateResult<OperationResult> {
        // Validate termination mode
        let has_artifact = self.vault.artifact_count_for_world(world_id) > 0;
        self.policy_engine
            .validate_termination(grammar_id, mode, has_artifact)?;

        // Terminate the world
        self.world_engine.terminate_world(world_id, mode)?;

        // Execute burn
        let _result = self.burn_engine.burn_world(world_id, mode)?;

        // Retire the world ID
        self.world_engine.retire_world_id(world_id);

        Ok(OperationResult::BurnTriggered)
    }
}
