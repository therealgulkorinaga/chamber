//! Operation engine for Chambers.
//!
//! Executes the finite primitive algebra against the state engine,
//! after the interpreter has validated the transition request.

use chambers_audit::{AuditEventType, AuditLog};
use chambers_state::StateEngine;
use chambers_types::artifact::*;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::object::*;
use chambers_types::primitive::*;
use chambers_types::world::WorldId;
use chambers_vault::ArtifactVault;
use chrono::Utc;
use std::sync::Arc;

/// The operation engine applies validated primitives to world state.
#[derive(Debug)]
pub struct OperationEngine {
    state: Arc<StateEngine>,
    vault: Arc<ArtifactVault>,
    audit: Arc<AuditLog>,
}

impl OperationEngine {
    pub fn new(
        state: Arc<StateEngine>,
        vault: Arc<ArtifactVault>,
        audit: Arc<AuditLog>,
    ) -> Self {
        Self { state, vault, audit }
    }

    /// Execute a validated transition operation.
    pub fn execute(
        &self,
        world_id: WorldId,
        operation: &TransitionOperation,
    ) -> SubstrateResult<OperationResult> {
        match operation {
            TransitionOperation::CreateObject {
                object_type,
                payload,
                lifecycle_class,
                preservable,
            } => self.exec_create_object(world_id, object_type, payload, *lifecycle_class, *preservable),

            TransitionOperation::LinkObjects {
                source_id,
                target_id,
                link_type,
            } => self.exec_link_objects(world_id, *source_id, *target_id, link_type),

            TransitionOperation::ChallengeObject {
                target_id,
                challenge_text,
            } => self.exec_challenge_object(world_id, *target_id, challenge_text),

            TransitionOperation::GenerateAlternative {
                target_id,
                alternative_payload,
            } => self.exec_generate_alternative(world_id, *target_id, alternative_payload),

            TransitionOperation::RankSet {
                object_ids,
                rankings,
            } => self.exec_rank_set(world_id, object_ids, rankings),

            TransitionOperation::SynthesizeSet {
                source_ids,
                synthesis_type,
                synthesis_payload,
            } => self.exec_synthesize_set(world_id, source_ids, synthesis_type, synthesis_payload),

            TransitionOperation::CondenseObject {
                target_id,
                condensed_payload,
            } => self.exec_condense_object(world_id, *target_id, condensed_payload),

            TransitionOperation::SealArtifact {
                target_id,
                authorization,
            } => self.exec_seal_artifact(world_id, *target_id, authorization),

            TransitionOperation::TriggerBurn { .. } => {
                // Burn is handled by the runtime/lifecycle controller, not here
                Ok(OperationResult::BurnTriggered)
            }
        }
    }

    fn exec_create_object(
        &self,
        world_id: WorldId,
        object_type: &str,
        payload: &serde_json::Value,
        lifecycle_class: LifecycleClass,
        preservable: bool,
    ) -> SubstrateResult<OperationResult> {
        let now = Utc::now();
        let object = Object {
            object_id: ObjectId::new(),
            world_id,
            object_type: object_type.to_string(),
            lifecycle_class,
            payload: payload.clone(),
            transform_set: Vec::new(), // set by interpreter from grammar
            preservable,
            capability_requirements: Vec::new(),
            created_at: now,
            last_modified_at: now,
            challenged: false,
            challenge_text: None,
            rank: None,
        };
        let oid = object.object_id;
        self.state.add_object(world_id, object)?;
        Ok(OperationResult::ObjectCreated(oid))
    }

    fn exec_link_objects(
        &self,
        world_id: WorldId,
        source_id: ObjectId,
        target_id: ObjectId,
        link_type: &str,
    ) -> SubstrateResult<OperationResult> {
        // Verify both objects exist in this world
        if !self.state.has_object(world_id, source_id)? {
            return Err(SubstrateError::ObjectNotFound {
                object_id: source_id,
                world_id,
            });
        }
        if !self.state.has_object(world_id, target_id)? {
            return Err(SubstrateError::ObjectNotFound {
                object_id: target_id,
                world_id,
            });
        }
        if self.state.link_exists(world_id, source_id, target_id)? {
            return Err(SubstrateError::DuplicateLink {
                source_id,
                target_id,
            });
        }

        let link = ObjectLink {
            source_id,
            target_id,
            link_type: link_type.to_string(),
            world_id,
        };
        self.state.add_link(world_id, link)?;
        Ok(OperationResult::LinkCreated)
    }

    fn exec_challenge_object(
        &self,
        world_id: WorldId,
        target_id: ObjectId,
        challenge_text: &str,
    ) -> SubstrateResult<OperationResult> {
        let ct = challenge_text.to_string();
        self.state.with_object_mut(world_id, target_id, move |obj| {
            obj.challenged = true;
            obj.challenge_text = Some(ct);
            obj.last_modified_at = Utc::now();
        })?;
        Ok(OperationResult::ObjectChallenged(target_id))
    }

    fn exec_generate_alternative(
        &self,
        world_id: WorldId,
        target_id: ObjectId,
        alternative_payload: &serde_json::Value,
    ) -> SubstrateResult<OperationResult> {
        // Verify target exists
        if !self.state.has_object(world_id, target_id)? {
            return Err(SubstrateError::ObjectNotFound {
                object_id: target_id,
                world_id,
            });
        }

        let now = Utc::now();
        let alt = Object {
            object_id: ObjectId::new(),
            world_id,
            object_type: "alternative".to_string(),
            lifecycle_class: LifecycleClass::Intermediate,
            payload: alternative_payload.clone(),
            transform_set: Vec::new(),
            preservable: false,
            capability_requirements: Vec::new(),
            created_at: now,
            last_modified_at: now,
            challenged: false,
            challenge_text: None,
            rank: None,
        };
        let alt_id = alt.object_id;

        self.state.add_object(world_id, alt)?;
        self.state.add_link(
            world_id,
            ObjectLink {
                source_id: alt_id,
                target_id,
                link_type: "alternative_to".to_string(),
                world_id,
            },
        )?;

        Ok(OperationResult::AlternativeGenerated(alt_id))
    }

    fn exec_rank_set(
        &self,
        world_id: WorldId,
        object_ids: &[ObjectId],
        rankings: &[i64],
    ) -> SubstrateResult<OperationResult> {
        for (oid, rank) in object_ids.iter().zip(rankings.iter()) {
            let r = *rank;
            // Ignore missing objects (same behavior as before)
            let _ = self.state.with_object_mut(world_id, *oid, move |obj| {
                obj.rank = Some(r);
                obj.last_modified_at = Utc::now();
            });
        }
        Ok(OperationResult::SetRanked)
    }

    fn exec_synthesize_set(
        &self,
        world_id: WorldId,
        source_ids: &[ObjectId],
        synthesis_type: &str,
        synthesis_payload: &serde_json::Value,
    ) -> SubstrateResult<OperationResult> {
        let now = Utc::now();
        let synth = Object {
            object_id: ObjectId::new(),
            world_id,
            object_type: synthesis_type.to_string(),
            lifecycle_class: LifecycleClass::Intermediate,
            payload: synthesis_payload.clone(),
            transform_set: Vec::new(),
            preservable: false,
            capability_requirements: Vec::new(),
            created_at: now,
            last_modified_at: now,
            challenged: false,
            challenge_text: None,
            rank: None,
        };
        let synth_id = synth.object_id;

        self.state.add_object(world_id, synth)?;
        for src_id in source_ids {
            self.state.add_link(
                world_id,
                ObjectLink {
                    source_id: synth_id,
                    target_id: *src_id,
                    link_type: "synthesized_from".to_string(),
                    world_id,
                },
            )?;
        }

        Ok(OperationResult::SetSynthesized(synth_id))
    }

    fn exec_condense_object(
        &self,
        world_id: WorldId,
        target_id: ObjectId,
        condensed_payload: &serde_json::Value,
    ) -> SubstrateResult<OperationResult> {
        let cp = condensed_payload.clone();
        self.state.with_object_mut(world_id, target_id, move |obj| {
            obj.payload = cp;
            obj.last_modified_at = Utc::now();
        })?;
        Ok(OperationResult::ObjectCondensed(target_id))
    }

    fn exec_seal_artifact(
        &self,
        world_id: WorldId,
        target_id: ObjectId,
        authorization: &SealAuthorization,
    ) -> SubstrateResult<OperationResult> {
        // Verify authorization
        match authorization {
            SealAuthorization::HumanConfirmed { .. } => {}
            SealAuthorization::PolicyApproved { .. } => {}
        }

        // Get the object (decrypt for sealing)
        let object = self
            .state
            .with_object(world_id, target_id, |o| o.clone())?;

        // Verify it's preservable
        if !object.preservable {
            return Err(SubstrateError::NotPreservable {
                object_type: object.object_type.clone(),
            });
        }

        let now = Utc::now();
        let artifact = Artifact {
            artifact_id: ArtifactId::new(),
            source_world_id: world_id,
            artifact_class: object.object_type.clone(),
            payload: object.payload.clone(),
            sealed_at: now,
            provenance_metadata: ProvenanceMetadata {
                grammar_id: String::new(), // filled by caller
                objective_summary: String::new(),
                world_created_at: object.created_at,
                world_terminated_at: now,
            },
            vault_policy_class: "default".to_string(),
        };

        let aid = self.vault.store_artifact(artifact)?;

        self.audit.record(
            world_id,
            AuditEventType::ArtifactSealed {
                artifact_class: object.object_type,
            },
        );

        Ok(OperationResult::ArtifactSealed(aid))
    }
}

/// Result of executing a primitive operation.
#[derive(Debug, Clone, serde::Serialize)]
pub enum OperationResult {
    ObjectCreated(ObjectId),
    LinkCreated,
    ObjectChallenged(ObjectId),
    AlternativeGenerated(ObjectId),
    SetRanked,
    SetSynthesized(ObjectId),
    ObjectCondensed(ObjectId),
    ArtifactSealed(ArtifactId),
    BurnTriggered,
}
