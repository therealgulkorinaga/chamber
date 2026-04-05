use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitive::Primitive;
use crate::world::WorldId;

/// Unique identifier for a capability token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenId(pub Uuid);

impl TokenId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

/// A capability token granting permission for specific operations within a world and epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub token_id: TokenId,
    pub world_id: WorldId,
    /// The epoch this token is valid for.
    pub epoch: u32,
    /// The principal (user, orchestrator, etc.) this token was issued to.
    pub principal: Principal,
    /// Which primitive operation this token permits.
    pub permitted_operation: Primitive,
    /// Which object types this token may target.
    pub permitted_object_types: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

/// The principal identity requesting an operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Principal(pub String);

impl Principal {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
