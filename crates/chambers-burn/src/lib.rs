//! Burn engine for Chambers.
//!
//! Implements the five-layer destruction model:
//! 1. Logical burn — revoke capabilities, invalidate handles
//! 2. Cryptographic burn — destroy K_w
//! 3. Storage cleanup — delete world-scoped records
//! 4. Memory cleanup — zero runtime structures
//! 5. Semantic residue measurement (stub for benchmarking)

use chambers_audit::{AuditEventType, AuditLog};
use chambers_capability::CapabilitySystem;
use chambers_crypto::CryptoProvider;
use chambers_state::StateEngine;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::world::{TerminationMode, WorldId};
use std::sync::Arc;

/// Result of a burn operation.
#[derive(Debug, serde::Serialize)]
pub struct BurnResult {
    pub world_id: WorldId,
    pub mode: TerminationMode,
    pub layers_completed: Vec<String>,
    pub errors: Vec<String>,
    pub residue: Option<SemanticResidueReport>,
}

/// Post-burn semantic residue measurement.
/// Attempts to reconstruct world state from what remains after burn.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SemanticResidueReport {
    /// Can we still find the world in the state engine?
    pub state_engine_has_world: bool,
    /// Does a crypto key still exist for this world?
    pub crypto_key_exists: bool,
    /// Is the crypto key marked as destroyed?
    pub crypto_key_destroyed: bool,
    /// Number of substrate-scoped audit events (Tier 1). Expected: exactly 2 (created + destroyed).
    pub substrate_event_count: usize,
    /// Number of world-scoped audit events surviving burn (Tier 2). Expected: 0.
    pub world_events_surviving: usize,
    /// Are any world-scoped audit events leaking through burn?
    pub audit_leaks_internals: bool,
    /// Residue score: 0.0 = no recoverable world state, 1.0 = fully recoverable.
    pub residue_score: f64,
}

/// The burn engine orchestrates world destruction.
#[derive(Debug)]
pub struct BurnEngine {
    crypto: Arc<CryptoProvider>,
    state: Arc<StateEngine>,
    capability: Arc<CapabilitySystem>,
    audit: Arc<AuditLog>,
}

impl BurnEngine {
    pub fn new(
        crypto: Arc<CryptoProvider>,
        state: Arc<StateEngine>,
        capability: Arc<CapabilitySystem>,
        audit: Arc<AuditLog>,
    ) -> Self {
        Self {
            crypto,
            state,
            capability,
            audit,
        }
    }

    /// Execute the full burn sequence.
    pub fn burn_world(
        &self,
        world_id: WorldId,
        mode: TerminationMode,
    ) -> SubstrateResult<BurnResult> {
        let mut result = BurnResult {
            world_id,
            mode,
            layers_completed: Vec::new(),
            errors: Vec::new(),
            residue: None,
        };

        self.audit.record(
            world_id,
            AuditEventType::BurnStarted { mode },
        );

        // Layer 1: Logical burn
        match self.logical_burn(world_id) {
            Ok(()) => {
                result.layers_completed.push("logical".into());
                self.audit.record(
                    world_id,
                    AuditEventType::BurnLayerCompleted {
                        layer: "logical".into(),
                    },
                );
            }
            Err(e) => result.errors.push(format!("logical: {}", e)),
        }

        // Layer 2: Cryptographic burn
        match self.cryptographic_burn(world_id) {
            Ok(()) => {
                result.layers_completed.push("cryptographic".into());
                self.audit.record(
                    world_id,
                    AuditEventType::BurnLayerCompleted {
                        layer: "cryptographic".into(),
                    },
                );
            }
            Err(e) => result.errors.push(format!("cryptographic: {}", e)),
        }

        // Layer 3: Storage cleanup
        match self.storage_cleanup(world_id) {
            Ok(()) => {
                result.layers_completed.push("storage".into());
                self.audit.record(
                    world_id,
                    AuditEventType::BurnLayerCompleted {
                        layer: "storage".into(),
                    },
                );
            }
            Err(e) => result.errors.push(format!("storage: {}", e)),
        }

        // Layer 4: Memory cleanup
        match self.memory_cleanup(world_id) {
            Ok(()) => {
                result.layers_completed.push("memory".into());
                self.audit.record(
                    world_id,
                    AuditEventType::BurnLayerCompleted {
                        layer: "memory".into(),
                    },
                );
            }
            Err(e) => result.errors.push(format!("memory: {}", e)),
        }

        // Layer 5: Destroy world-scoped audit events (Tier 2)
        // After this, only Tier 1 events survive (WorldCreated + WorldDestroyed)
        self.audit.burn_world_events(world_id);
        result.layers_completed.push("audit_burn".into());

        // Layer 6: Semantic residue measurement
        let residue = self.semantic_measurement(world_id);
        result.residue = Some(residue);
        result.layers_completed.push("semantic_measurement".into());

        // Tier 1 event: world destroyed (this is the ONLY post-burn record besides WorldCreated)
        self.audit.record(
            world_id,
            AuditEventType::BurnCompleted { mode },
        );

        Ok(result)
    }

    /// Layer 1: Logical burn — revoke all capabilities, mark handles invalid.
    fn logical_burn(&self, world_id: WorldId) -> SubstrateResult<()> {
        self.capability.revoke_all_for_world(world_id);
        Ok(())
    }

    /// Layer 2: Cryptographic burn — destroy K_w.
    fn cryptographic_burn(&self, world_id: WorldId) -> SubstrateResult<()> {
        self.crypto
            .destroy_world_key(world_id)
            .map_err(|e| SubstrateError::BurnFailed {
                layer: "cryptographic".into(),
                reason: e.to_string(),
            })
    }

    /// Layer 3: Storage cleanup — remove world-scoped data.
    fn storage_cleanup(&self, world_id: WorldId) -> SubstrateResult<()> {
        // State engine handles object/link storage
        self.state.destroy_world_state(world_id)?;
        Ok(())
    }

    /// Layer 4: Memory cleanup — zero/drop in-memory structures.
    fn memory_cleanup(&self, world_id: WorldId) -> SubstrateResult<()> {
        // Destroy capability tokens from memory
        self.capability.destroy_world_tokens(world_id);
        Ok(())
    }

    /// Layer 5: Semantic residue measurement.
    /// Post-burn analysis: what can be recovered from remaining substrate state?
    /// Also available as a standalone measurement tool via `measure_residue`.
    pub fn measure_residue(&self, world_id: WorldId) -> SemanticResidueReport {
        self.semantic_measurement(world_id)
    }

    fn semantic_measurement(&self, world_id: WorldId) -> SemanticResidueReport {
        let state_engine_has_world = self.state.has_world(world_id);
        let crypto_key_exists = self.crypto.has_world_key(world_id);
        let crypto_key_destroyed = self.crypto.is_key_destroyed(world_id);

        // Count only substrate-scoped events (Tier 1) — this is the post-burn metadata
        let substrate_event_count = self.audit.substrate_event_count(world_id);

        // Check: do any world-scoped (Tier 2) events still exist? They shouldn't.
        let all_events = self.audit.events_for_world(world_id);
        let world_scoped_surviving = all_events.iter().filter(|e| !e.event_type.is_substrate_scoped()).count();
        let audit_leaks_internals = world_scoped_surviving > 0;

        // Compute residue score.
        // 0.0 = perfect burn (no recoverable world state).
        let mut score = 0.0;
        if state_engine_has_world {
            score += 0.4; // Major: full object graph recoverable
        }
        if crypto_key_exists {
            score += 0.4; // Major: ciphertext decryptable
        }
        if audit_leaks_internals {
            score += 0.15; // Moderate: world-scoped audit events survived burn
        }
        // Substrate events (Tier 1) are expected — max 2 (created + destroyed).
        // They reveal only that a world existed. Not scored as residue.

        SemanticResidueReport {
            state_engine_has_world,
            crypto_key_exists,
            crypto_key_destroyed,
            substrate_event_count,
            world_events_surviving: world_scoped_surviving,
            audit_leaks_internals,
            residue_score: score,
        }
    }
}
