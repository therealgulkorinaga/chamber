//! State engine for Chambers.
//!
//! Holds the encrypted object graph, links, lifecycle phase,
//! capability graph, convergence review state, and temporary
//! render state per world.
//!
//! All object/link data is encrypted at rest under K_w via EncryptedWorldState.

use chambers_crypto::CryptoProvider;
use chambers_crypto::encrypted_store::EncryptedWorldState;
use chambers_types::object::{Object, ObjectId, ObjectLink};
use chambers_types::world::WorldId;
use chambers_types::error::{SubstrateError, SubstrateResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zeroize::Zeroize;

/// Convergence review state — tracks whether a world is ready to finalize.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ConvergenceReviewState {
    /// Set of unresolved contradiction object IDs.
    pub unresolved_contradictions: Vec<ObjectId>,
    /// Map of required type → whether at least one exists.
    pub mandatory_type_satisfaction: HashMap<String, bool>,
    /// Pointer to the candidate artifact (decision_summary) if one exists.
    pub candidate_artifact_id: Option<ObjectId>,
    /// Whether convergence has been proposed.
    pub convergence_proposed: bool,
    /// Whether convergence passed validation.
    pub convergence_validated: Option<bool>,
    /// Validation failure reason, if any.
    pub validation_failure_reason: Option<String>,
}

/// Temporary render state — caches for views.
/// Tagged as temporary. Participates in burn. Must not create persistent traces.
#[derive(Debug, Clone, Default)]
pub struct RenderState {
    /// Cached conversation entries (ephemeral).
    pub conversation_cache: Option<Vec<String>>,
    /// Cached graph adjacency (ephemeral).
    pub graph_cache: Option<Vec<(ObjectId, ObjectId)>>,
    /// Whether render state is dirty (needs refresh).
    pub dirty: bool,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            conversation_cache: None,
            graph_cache: None,
            dirty: true,
        }
    }

    /// Clear all render caches. Called during burn.
    pub fn clear(&mut self) {
        self.conversation_cache = None;
        self.graph_cache = None;
        self.dirty = true;
    }
}

/// Bundle of encrypted world data + non-encrypted metadata.
#[derive(Debug)]
pub struct EncryptedWorldStateBundle {
    pub encrypted: EncryptedWorldState,
    pub convergence: ConvergenceReviewState,
    pub render: RenderState,
}

/// The state engine manages world states across all active worlds.
/// All object/link data is encrypted via CryptoProvider.
#[derive(Debug)]
pub struct StateEngine {
    worlds: Arc<Mutex<HashMap<WorldId, EncryptedWorldStateBundle>>>,
    crypto: Arc<CryptoProvider>,
}

impl StateEngine {
    pub fn new(crypto: Arc<CryptoProvider>) -> Self {
        Self {
            worlds: Arc::new(Mutex::new(HashMap::new())),
            crypto,
        }
    }

    pub fn create_world_state(&self, world_id: WorldId) {
        self.worlds.lock().unwrap().insert(
            world_id,
            EncryptedWorldStateBundle {
                encrypted: EncryptedWorldState::new(),
                convergence: ConvergenceReviewState::default(),
                render: RenderState::new(),
            },
        );
    }

    /// Remove all state for a world (used during burn).
    /// Securely wipes all ciphertext content before dropping.
    pub fn destroy_world_state(&self, world_id: WorldId) -> SubstrateResult<()> {
        if let Some(mut bundle) = self.worlds.lock().unwrap().remove(&world_id) {
            bundle.encrypted.secure_wipe();
            // Wipe convergence state
            bundle.convergence.validation_failure_reason.zeroize();
            bundle.convergence.mandatory_type_satisfaction.clear();
            bundle.convergence.unresolved_contradictions.clear();
            bundle.convergence.candidate_artifact_id = None;
            bundle.convergence.convergence_proposed = false;
            bundle.convergence.convergence_validated = None;
            // Wipe render caches
            bundle.render.clear();
        }
        Ok(())
    }

    pub fn has_world(&self, world_id: WorldId) -> bool {
        self.worlds.lock().unwrap().contains_key(&world_id)
    }

    // --- Object operations (encrypt/decrypt via CryptoProvider) ---

    pub fn add_object(&self, world_id: WorldId, object: Object) -> SubstrateResult<()> {
        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        bundle.render.dirty = true;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.add_object(&object, key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e,
            })
    }

    pub fn with_object<F, R>(
        &self,
        world_id: WorldId,
        object_id: ObjectId,
        f: F,
    ) -> SubstrateResult<R>
    where
        F: FnOnce(&Object) -> R,
    {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.with_object(object_id, key, f)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e,
            })
    }

    pub fn with_object_mut<F>(
        &self,
        world_id: WorldId,
        object_id: ObjectId,
        f: F,
    ) -> SubstrateResult<()>
    where
        F: FnOnce(&mut Object),
    {
        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        bundle.render.dirty = true;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.with_object_mut(object_id, key, f)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e,
            })
    }

    pub fn has_object(&self, world_id: WorldId, object_id: ObjectId) -> SubstrateResult<bool> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.has_object(object_id))
    }

    pub fn object_type(
        &self,
        world_id: WorldId,
        object_id: ObjectId,
    ) -> SubstrateResult<Option<String>> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.object_type(object_id).map(|s| s.to_string()))
    }

    pub fn is_preservable(
        &self,
        world_id: WorldId,
        object_id: ObjectId,
    ) -> SubstrateResult<bool> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.is_preservable(object_id))
    }

    // --- Link operations ---

    pub fn add_link(&self, world_id: WorldId, link: ObjectLink) -> SubstrateResult<()> {
        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        bundle.render.dirty = true;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.add_link(&link, key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e,
            })
    }

    pub fn link_exists(
        &self,
        world_id: WorldId,
        source_id: ObjectId,
        target_id: ObjectId,
    ) -> SubstrateResult<bool> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.link_exists(source_id, target_id, key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })
    }

    // --- Bulk read (for views — decrypts one at a time) ---

    pub fn all_objects_decrypted(&self, world_id: WorldId) -> SubstrateResult<Vec<Object>> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.all_objects_decrypted(key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })
    }

    pub fn all_links_decrypted(&self, world_id: WorldId) -> SubstrateResult<Vec<ObjectLink>> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.all_links_decrypted(key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })
    }

    // --- Counts (no decryption) ---

    pub fn object_count(&self, world_id: WorldId) -> SubstrateResult<usize> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.object_count())
    }

    pub fn link_count(&self, world_id: WorldId) -> SubstrateResult<usize> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.link_count())
    }

    pub fn has_objects_of_type(
        &self,
        world_id: WorldId,
        object_type: &str,
    ) -> SubstrateResult<bool> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(bundle.encrypted.has_objects_of_type(object_type))
    }

    pub fn has_unresolved_challenges(&self, world_id: WorldId) -> SubstrateResult<bool> {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        self.crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.has_unresolved_challenges(key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })
    }

    // --- Convergence (no encryption — metadata only) ---

    pub fn with_convergence<F, R>(&self, world_id: WorldId, f: F) -> SubstrateResult<R>
    where
        F: FnOnce(&ConvergenceReviewState) -> R,
    {
        let worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(f(&bundle.convergence))
    }

    pub fn with_convergence_mut<F, R>(&self, world_id: WorldId, f: F) -> SubstrateResult<R>
    where
        F: FnOnce(&mut ConvergenceReviewState) -> R,
    {
        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        Ok(f(&mut bundle.convergence))
    }

    /// Update convergence review state based on current world state.
    pub fn refresh_convergence(
        &self,
        world_id: WorldId,
        required_types: &[String],
        challenges_block: bool,
        contradictions_block: bool,
    ) -> SubstrateResult<()> {
        // Gather data we need while holding the lock, then update convergence.
        // We need to do crypto operations, so we gather everything in one pass.

        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;

        // Check mandatory types (plaintext index, no decryption)
        bundle.convergence.mandatory_type_satisfaction.clear();
        for t in required_types {
            bundle
                .convergence
                .mandatory_type_satisfaction
                .insert(t.clone(), bundle.encrypted.has_objects_of_type(t));
        }

        // Check contradictions (needs decryption to read payload.resolved)
        let all_objects = self
            .crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.all_objects_decrypted(key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?;

        bundle.convergence.unresolved_contradictions = all_objects
            .iter()
            .filter(|o| {
                o.object_type == "contradiction"
                    && o.payload.get("resolved") != Some(&serde_json::Value::Bool(true))
            })
            .map(|o| o.object_id)
            .collect();

        // Find candidate artifact (preservable flag is in plaintext index)
        bundle.convergence.candidate_artifact_id = bundle
            .encrypted
            .objects
            .values()
            .find(|e| e.preservable)
            .map(|e| e.object_id);

        // Check challenges (needs decryption)
        let has_challenges = self
            .crypto
            .with_world_key(world_id, |key| {
                bundle.encrypted.has_unresolved_challenges(key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?;

        // Validate convergence
        let all_mandatory = bundle
            .convergence
            .mandatory_type_satisfaction
            .values()
            .all(|v| *v);

        let challenges_ok = !challenges_block || !has_challenges;
        let contradictions_ok =
            !contradictions_block || bundle.convergence.unresolved_contradictions.is_empty();

        if all_mandatory && challenges_ok && contradictions_ok {
            bundle.convergence.convergence_validated = Some(true);
            bundle.convergence.validation_failure_reason = None;
        } else {
            bundle.convergence.convergence_validated = Some(false);
            let mut reasons = Vec::new();
            if !all_mandatory {
                reasons.push("missing required object types".to_string());
            }
            if !challenges_ok {
                reasons.push("unresolved challenges".to_string());
            }
            if !contradictions_ok {
                reasons.push(format!(
                    "{} unresolved contradictions",
                    bundle.convergence.unresolved_contradictions.len()
                ));
            }
            bundle.convergence.validation_failure_reason = Some(reasons.join("; "));
        }

        Ok(())
    }

    // --- For compound mutations that need atomicity ---

    pub fn with_encrypted_state_mut<F, R>(
        &self,
        world_id: WorldId,
        f: F,
    ) -> SubstrateResult<R>
    where
        F: FnOnce(&mut EncryptedWorldState, &chambers_crypto::WorldKey) -> Result<R, String>,
    {
        let mut worlds = self.worlds.lock().unwrap();
        let bundle = worlds
            .get_mut(&world_id)
            .ok_or(SubstrateError::WorldNotFound(world_id))?;
        bundle.render.dirty = true;
        self.crypto
            .with_world_key(world_id, |key| {
                f(&mut bundle.encrypted, key)
            })
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e.to_string(),
            })?
            .map_err(|e| SubstrateError::CryptoOperationFailed {
                world_id,
                reason: e,
            })
    }
}
