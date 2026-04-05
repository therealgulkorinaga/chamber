//! Capability system for Chambers.
//!
//! World-scoped, epoch-scoped capability tokens.
//! On epoch advance, all tokens are invalidated unless reissued.

use chambers_types::capability::*;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::primitive::Primitive;
use chambers_types::world::WorldId;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The capability system manages token issuance, validation, and revocation.
#[derive(Debug)]
pub struct CapabilitySystem {
    tokens: Arc<Mutex<HashMap<TokenId, CapabilityToken>>>,
}

impl CapabilitySystem {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Issue a new capability token.
    pub fn issue_token(
        &self,
        world_id: WorldId,
        epoch: u32,
        principal: Principal,
        permitted_operation: Primitive,
        permitted_object_types: Vec<String>,
    ) -> CapabilityToken {
        let token = CapabilityToken {
            token_id: TokenId::new(),
            world_id,
            epoch,
            principal,
            permitted_operation,
            permitted_object_types,
            issued_at: Utc::now(),
            expires_at: None,
            revoked: false,
        };
        self.tokens
            .lock()
            .unwrap()
            .insert(token.token_id, token.clone());
        token
    }

    /// Check if a principal has capability for an operation in a world.
    pub fn check_capability(
        &self,
        world_id: WorldId,
        world_epoch: u32,
        principal: &Principal,
        operation: Primitive,
        object_type: &str,
    ) -> SubstrateResult<()> {
        let tokens = self.tokens.lock().unwrap();
        let matching = tokens.values().find(|t| {
            t.world_id == world_id
                && t.principal == *principal
                && t.permitted_operation == operation
                && (t.permitted_object_types.is_empty()
                    || t.permitted_object_types.contains(&object_type.to_string()))
        });

        match matching {
            None => Err(SubstrateError::MissingCapability {
                operation,
                world_id,
            }),
            Some(token) => {
                if token.revoked {
                    return Err(SubstrateError::CapabilityRevoked);
                }
                if let Some(expires) = token.expires_at {
                    if Utc::now() > expires {
                        return Err(SubstrateError::CapabilityExpired);
                    }
                }
                if token.epoch != world_epoch {
                    return Err(SubstrateError::WrongEpoch {
                        token_epoch: token.epoch,
                        world_epoch,
                    });
                }
                if token.world_id != world_id {
                    return Err(SubstrateError::WrongWorld);
                }
                Ok(())
            }
        }
    }

    /// Invalidate all tokens for a world at a given epoch.
    pub fn invalidate_epoch(&self, world_id: WorldId, old_epoch: u32) {
        let mut tokens = self.tokens.lock().unwrap();
        for token in tokens.values_mut() {
            if token.world_id == world_id && token.epoch == old_epoch {
                token.revoked = true;
            }
        }
    }

    /// Revoke a specific token.
    pub fn revoke_token(&self, token_id: TokenId) -> SubstrateResult<()> {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.get_mut(&token_id) {
            token.revoked = true;
            Ok(())
        } else {
            Ok(()) // idempotent
        }
    }

    /// Revoke all tokens for a world (used during burn).
    pub fn revoke_all_for_world(&self, world_id: WorldId) {
        let mut tokens = self.tokens.lock().unwrap();
        for token in tokens.values_mut() {
            if token.world_id == world_id {
                token.revoked = true;
            }
        }
    }

    /// Remove all tokens for a world from memory (used during burn cleanup).
    pub fn destroy_world_tokens(&self, world_id: WorldId) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, t| t.world_id != world_id);
    }

    /// Issue a standard set of capabilities for a lifecycle phase.
    pub fn issue_phase_capabilities(
        &self,
        world_id: WorldId,
        epoch: u32,
        principal: Principal,
        allowed_primitives: &[Primitive],
        object_types: Vec<String>,
    ) -> Vec<CapabilityToken> {
        allowed_primitives
            .iter()
            .map(|prim| {
                self.issue_token(
                    world_id,
                    epoch,
                    principal.clone(),
                    *prim,
                    object_types.clone(),
                )
            })
            .collect()
    }
}

impl Default for CapabilitySystem {
    fn default() -> Self {
        Self::new()
    }
}
