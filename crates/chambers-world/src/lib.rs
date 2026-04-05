//! World engine for Chambers.
//!
//! Responsible for world creation, ID allocation, namespace isolation,
//! lifecycle phase tracking, and termination dispatch.

use chambers_audit::{AuditEventType, AuditLog};
use chambers_crypto::CryptoProvider;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::world::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The world engine manages world lifecycles.
#[derive(Debug)]
pub struct WorldEngine {
    /// All worlds (live and terminated, pre-burn cleanup).
    worlds: Arc<Mutex<HashMap<WorldId, World>>>,
    /// Retired world IDs — never reused.
    retired_ids: Arc<Mutex<Vec<WorldId>>>,
    /// Crypto provider for key generation.
    crypto: Arc<CryptoProvider>,
    /// Audit log.
    audit: Arc<AuditLog>,
}

impl WorldEngine {
    pub fn new(crypto: Arc<CryptoProvider>, audit: Arc<AuditLog>) -> Self {
        Self {
            worlds: Arc::new(Mutex::new(HashMap::new())),
            retired_ids: Arc::new(Mutex::new(Vec::new())),
            crypto,
            audit,
        }
    }

    /// Create a new world from a grammar.
    pub fn create_world(
        &self,
        grammar_id: String,
        objective: String,
    ) -> SubstrateResult<WorldId> {
        let world_id = WorldId::new();

        // Verify no reuse
        if self.retired_ids.lock().unwrap().contains(&world_id) {
            return Err(SubstrateError::WorldIdReuse(world_id));
        }

        // Generate world-scoped key K_w
        self.crypto
            .generate_world_key(world_id)
            .map_err(|e| SubstrateError::BurnFailed {
                layer: "key_generation".into(),
                reason: e.to_string(),
            })?;

        let key_ref = KeyRef(format!("kw_{}", world_id));

        let world = World {
            world_id,
            grammar_id: grammar_id.clone(),
            objective,
            lifecycle_phase: LifecyclePhase::Created,
            epoch: 0,
            world_key_ref: key_ref,
            artifact_key_ref: None,
            created_at: Utc::now(),
            terminated_at: None,
            termination_mode: None,
        };

        self.worlds.lock().unwrap().insert(world_id, world);

        self.audit.record(
            world_id,
            AuditEventType::WorldCreated { grammar_id },
        );

        Ok(world_id)
    }

    /// Advance the lifecycle phase of a world.
    pub fn advance_phase(
        &self,
        world_id: WorldId,
        target: LifecyclePhase,
    ) -> SubstrateResult<()> {
        let mut worlds = self.worlds.lock().unwrap();
        let world = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;

        if world.lifecycle_phase == LifecyclePhase::Terminated {
            return Err(SubstrateError::WorldTerminated(world_id));
        }

        if !world.lifecycle_phase.can_transition_to(target) {
            return Err(SubstrateError::InvalidLifecycleTransition {
                from: world.lifecycle_phase,
                to: target,
            });
        }

        let from = world.lifecycle_phase;
        world.lifecycle_phase = target;
        world.epoch = target.epoch_index();

        if target == LifecyclePhase::Terminated {
            world.terminated_at = Some(Utc::now());
        }

        self.audit.record(
            world_id,
            AuditEventType::PhaseTransition { from, to: target },
        );

        Ok(())
    }

    /// Mark a world as terminated with a specific mode.
    pub fn terminate_world(
        &self,
        world_id: WorldId,
        mode: TerminationMode,
    ) -> SubstrateResult<()> {
        let mut worlds = self.worlds.lock().unwrap();
        let world = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;

        if world.lifecycle_phase == LifecyclePhase::Terminated {
            return Ok(()); // idempotent
        }

        let from = world.lifecycle_phase;
        world.lifecycle_phase = LifecyclePhase::Terminated;
        world.epoch = LifecyclePhase::Terminated.epoch_index();
        world.terminated_at = Some(Utc::now());
        world.termination_mode = Some(mode);

        self.audit.record(
            world_id,
            AuditEventType::PhaseTransition {
                from,
                to: LifecyclePhase::Terminated,
            },
        );

        Ok(())
    }

    /// Retire a world ID after burn — can never be reused.
    pub fn retire_world_id(&self, world_id: WorldId) {
        self.worlds.lock().unwrap().remove(&world_id);
        self.retired_ids.lock().unwrap().push(world_id);
    }

    /// Get a world by ID.
    pub fn get_world(&self, world_id: WorldId) -> SubstrateResult<World> {
        self.worlds
            .lock()
            .unwrap()
            .get(&world_id)
            .cloned()
            .ok_or(SubstrateError::WorldNotFound(world_id))
    }

    /// Check if a world exists and is not terminated.
    pub fn is_world_active(&self, world_id: WorldId) -> bool {
        self.worlds
            .lock()
            .unwrap()
            .get(&world_id)
            .map(|w| w.lifecycle_phase != LifecyclePhase::Terminated)
            .unwrap_or(false)
    }

    /// Check if a world ID has been retired.
    pub fn is_retired(&self, world_id: WorldId) -> bool {
        self.retired_ids.lock().unwrap().contains(&world_id)
    }

    /// Get the current lifecycle phase of a world.
    pub fn get_phase(&self, world_id: WorldId) -> SubstrateResult<LifecyclePhase> {
        Ok(self.get_world(world_id)?.lifecycle_phase)
    }

    /// Get the current epoch of a world.
    pub fn get_epoch(&self, world_id: WorldId) -> SubstrateResult<u32> {
        Ok(self.get_world(world_id)?.epoch)
    }
}
