use thiserror::Error;

use crate::object::ObjectId;
use crate::primitive::Primitive;
use crate::world::{LifecyclePhase, WorldId};

/// Errors from the substrate runtime.
#[derive(Debug, Error)]
pub enum SubstrateError {
    // World errors
    #[error("world not found: {0}")]
    WorldNotFound(WorldId),

    #[error("world already terminated: {0}")]
    WorldTerminated(WorldId),

    #[error("world ID reuse attempted: {0}")]
    WorldIdReuse(WorldId),

    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidLifecycleTransition {
        from: LifecyclePhase,
        to: LifecyclePhase,
    },

    // Object errors
    #[error("object not found: {object_id} in world {world_id}")]
    ObjectNotFound {
        object_id: ObjectId,
        world_id: WorldId,
    },

    #[error("unknown object type: {0}")]
    UnknownObjectType(String),

    #[error("invalid payload for type {object_type}: {reason}")]
    InvalidPayload {
        object_type: String,
        reason: String,
    },

    #[error("cross-world access denied: object {object_id} belongs to world {owner_world}, not {requesting_world}")]
    CrossWorldAccess {
        object_id: ObjectId,
        owner_world: WorldId,
        requesting_world: WorldId,
    },

    #[error("binary payload rejected for type {object_type}: no opaque binary payloads allowed")]
    BinaryPayloadRejected { object_type: String },

    // Capability errors
    #[error("no capability for operation {operation} in world {world_id}")]
    MissingCapability {
        operation: Primitive,
        world_id: WorldId,
    },

    #[error("capability token revoked")]
    CapabilityRevoked,

    #[error("capability token expired")]
    CapabilityExpired,

    #[error("capability token from wrong epoch: token epoch {token_epoch}, world epoch {world_epoch}")]
    WrongEpoch { token_epoch: u32, world_epoch: u32 },

    #[error("capability token from wrong world")]
    WrongWorld,

    // Lifecycle errors
    #[error("operation {operation} not permitted in phase {phase:?}")]
    OperationNotPermittedInPhase {
        operation: Primitive,
        phase: LifecyclePhase,
    },

    // Preservation errors
    #[error("object type {object_type} is not preservable under grammar preservation law")]
    NotPreservable { object_type: String },

    #[error("seal requires authorization — model cannot unilaterally preserve")]
    SealUnauthorized,

    // Convergence errors
    #[error("convergence check failed: {reason}")]
    ConvergenceFailed { reason: String },

    #[error("no decision summary exists for converged-preserving termination")]
    NoArtifactForPreservation,

    // Burn errors
    #[error("burn failed at layer {layer}: {reason}")]
    BurnFailed { layer: String, reason: String },

    // Policy errors
    #[error("grammar not found: {0}")]
    GrammarNotFound(String),

    #[error("policy violation: {0}")]
    PolicyViolation(String),

    // Link errors
    #[error("duplicate link from {source_id} to {target_id}")]
    DuplicateLink {
        source_id: ObjectId,
        target_id: ObjectId,
    },

    #[error("link type {link_type} not permitted between {source_type} and {target_type}")]
    InvalidLinkType {
        link_type: String,
        source_type: String,
        target_type: String,
    },

    #[error("crypto operation failed for world {world_id}: {reason}")]
    CryptoOperationFailed { world_id: WorldId, reason: String },
}

pub type SubstrateResult<T> = Result<T, SubstrateError>;
