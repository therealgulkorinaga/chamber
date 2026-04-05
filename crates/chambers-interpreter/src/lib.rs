//! Closed interpreter for Chambers.
//!
//! The ONLY public route for world evolution.
//! Validates every transition request against:
//! 1. World-scope correctness
//! 2. Type compatibility
//! 3. Capability possession
//! 4. Lifecycle legality
//! 5. Preservation-law legality

use chambers_audit::AuditLog;
use chambers_capability::CapabilitySystem;
use chambers_operation::{OperationEngine, OperationResult};
use chambers_policy::PolicyEngine;
use chambers_state::StateEngine;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::object::ObjectId;
use chambers_types::primitive::*;
use chambers_types::world::{LifecyclePhase, WorldId};
use std::sync::Arc;

/// The interpreter — sole entry point for world state mutation.
#[derive(Debug)]
pub struct Interpreter {
    policy: Arc<PolicyEngine>,
    capability: Arc<CapabilitySystem>,
    state: Arc<StateEngine>,
    operation: Arc<OperationEngine>,
    audit: Arc<AuditLog>,
}

/// Minimal world info needed for validation.
pub struct WorldContext {
    pub world_id: WorldId,
    pub grammar_id: String,
    pub phase: LifecyclePhase,
    pub epoch: u32,
}

impl Interpreter {
    pub fn new(
        policy: Arc<PolicyEngine>,
        capability: Arc<CapabilitySystem>,
        state: Arc<StateEngine>,
        operation: Arc<OperationEngine>,
        audit: Arc<AuditLog>,
    ) -> Self {
        Self {
            policy,
            capability,
            state,
            operation,
            audit,
        }
    }

    /// Submit a transition request. This is the ONLY way to evolve world state.
    pub fn submit(
        &self,
        request: &TransitionRequest,
        ctx: &WorldContext,
    ) -> SubstrateResult<OperationResult> {
        let primitive = request.operation.primitive();

        // 1. World-scope correctness
        self.check_world_scope(ctx)?;

        // 2. Type compatibility
        self.check_type_compatibility(ctx, &request.operation)?;

        // 3. Capability possession
        self.check_capability(ctx, request, primitive)?;

        // 4. Lifecycle legality
        self.check_lifecycle_legality(ctx, primitive)?;

        // 5. Preservation-law legality
        self.check_preservation_law(ctx, &request.operation)?;

        // All checks passed — execute
        self.operation.execute(ctx.world_id, &request.operation)
    }

    /// Check 1: World exists and is not terminated.
    fn check_world_scope(&self, ctx: &WorldContext) -> SubstrateResult<()> {
        if ctx.phase == LifecyclePhase::Terminated {
            return Err(SubstrateError::WorldTerminated(ctx.world_id));
        }
        if !self.state.has_world(ctx.world_id) {
            return Err(SubstrateError::WorldNotFound(ctx.world_id));
        }
        Ok(())
    }

    /// Check 2: Object types in the request are valid for the grammar.
    fn check_type_compatibility(
        &self,
        ctx: &WorldContext,
        operation: &TransitionOperation,
    ) -> SubstrateResult<()> {
        match operation {
            TransitionOperation::CreateObject { object_type, .. } => {
                if !self
                    .policy
                    .is_object_type_allowed(&ctx.grammar_id, object_type)?
                {
                    return Err(SubstrateError::UnknownObjectType(object_type.clone()));
                }
            }
            TransitionOperation::LinkObjects {
                source_id,
                target_id,
                ..
            } => {
                self.verify_objects_in_world(ctx.world_id, &[*source_id, *target_id])?;
            }
            TransitionOperation::ChallengeObject { target_id, .. }
            | TransitionOperation::CondenseObject { target_id, .. }
            | TransitionOperation::GenerateAlternative { target_id, .. } => {
                self.verify_objects_in_world(ctx.world_id, &[*target_id])?;
            }
            TransitionOperation::SealArtifact { target_id, .. } => {
                self.verify_objects_in_world(ctx.world_id, &[*target_id])?;
            }
            TransitionOperation::RankSet { object_ids, rankings } => {
                if object_ids.len() != rankings.len() {
                    return Err(SubstrateError::PolicyViolation(
                        "rank_set: object_ids and rankings must have same length".into(),
                    ));
                }
                self.verify_objects_in_world(ctx.world_id, object_ids)?;
            }
            TransitionOperation::SynthesizeSet { source_ids, .. } => {
                self.verify_objects_in_world(ctx.world_id, source_ids)?;
            }
            TransitionOperation::TriggerBurn { .. } => {}
        }
        Ok(())
    }

    /// Check 3: Principal has required capability.
    fn check_capability(
        &self,
        ctx: &WorldContext,
        request: &TransitionRequest,
        primitive: Primitive,
    ) -> SubstrateResult<()> {
        let object_type = self.infer_object_type(ctx.world_id, &request.operation);
        self.capability.check_capability(
            ctx.world_id,
            ctx.epoch,
            &request.principal,
            primitive,
            &object_type,
        )
    }

    /// Check 4: Primitive is allowed in current lifecycle phase.
    fn check_lifecycle_legality(
        &self,
        ctx: &WorldContext,
        primitive: Primitive,
    ) -> SubstrateResult<()> {
        if !self
            .policy
            .is_primitive_allowed(&ctx.grammar_id, primitive, ctx.phase)?
        {
            return Err(SubstrateError::OperationNotPermittedInPhase {
                operation: primitive,
                phase: ctx.phase,
            });
        }
        Ok(())
    }

    /// Check 5: Operation does not violate preservation law.
    fn check_preservation_law(
        &self,
        ctx: &WorldContext,
        operation: &TransitionOperation,
    ) -> SubstrateResult<()> {
        if let TransitionOperation::SealArtifact { target_id, authorization } = operation {
            // Verify object is of preservable class (plaintext index, no decryption)
            let object_type = self
                .state
                .object_type(ctx.world_id, *target_id)?
                .ok_or(SubstrateError::ObjectNotFound {
                    object_id: *target_id,
                    world_id: ctx.world_id,
                })?;

            if !self.policy.can_preserve_object(&ctx.grammar_id, &object_type)? {
                return Err(SubstrateError::NotPreservable { object_type });
            }

            // Must be in finalization phase
            if ctx.phase != LifecyclePhase::Finalization {
                return Err(SubstrateError::OperationNotPermittedInPhase {
                    operation: Primitive::SealArtifact,
                    phase: ctx.phase,
                });
            }

            // Must have authorization (model cannot unilaterally seal)
            match authorization {
                SealAuthorization::HumanConfirmed { .. } | SealAuthorization::PolicyApproved { .. } => {}
            }
        }
        Ok(())
    }

    fn verify_objects_in_world(
        &self,
        world_id: WorldId,
        object_ids: &[ObjectId],
    ) -> SubstrateResult<()> {
        for oid in object_ids {
            if !self.state.has_object(world_id, *oid)? {
                return Err(SubstrateError::ObjectNotFound {
                    object_id: *oid,
                    world_id,
                });
            }
        }
        Ok(())
    }

    fn infer_object_type(&self, world_id: WorldId, operation: &TransitionOperation) -> String {
        match operation {
            TransitionOperation::CreateObject { object_type, .. } => object_type.clone(),
            TransitionOperation::SynthesizeSet { synthesis_type, .. } => synthesis_type.clone(),
            TransitionOperation::LinkObjects { source_id, .. } => {
                self.state
                    .object_type(world_id, *source_id)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
            }
            TransitionOperation::ChallengeObject { target_id, .. }
            | TransitionOperation::CondenseObject { target_id, .. }
            | TransitionOperation::GenerateAlternative { target_id, .. }
            | TransitionOperation::SealArtifact { target_id, .. } => {
                self.state
                    .object_type(world_id, *target_id)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
            }
            TransitionOperation::RankSet { object_ids, .. } => {
                object_ids.first().map(|id| {
                    self.state
                        .object_type(world_id, *id)
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                }).unwrap_or_default()
            }
            TransitionOperation::TriggerBurn { .. } => String::new(),
        }
    }
}
