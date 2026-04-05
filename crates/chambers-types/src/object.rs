use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::primitive::Primitive;
use crate::world::WorldId;

/// Unique identifier for an object within a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub Uuid);

impl ObjectId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle class determines what happens to an object at burn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleClass {
    /// Destroyed at burn. Used for working state.
    Temporary,
    /// Destroyed at burn. Intermediate reasoning artifacts.
    Intermediate,
    /// May be promoted to Preservable if convergence criteria met.
    Candidate,
    /// Eligible for artifact sealing and vault survival.
    Preservable,
}

/// A typed object within a world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub object_id: ObjectId,
    pub world_id: WorldId,
    /// The grammar-defined type (e.g., "premise", "risk", "decision_summary").
    pub object_type: String,
    pub lifecycle_class: LifecycleClass,
    /// Structured payload — must conform to the schema for this object_type.
    pub payload: serde_json::Value,
    /// Which primitives may target this object.
    pub transform_set: Vec<Primitive>,
    /// Whether this object can be sealed into the vault.
    pub preservable: bool,
    /// Capabilities required to operate on this object.
    pub capability_requirements: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
    /// Whether this object has been challenged.
    pub challenged: bool,
    /// Challenge text, if any.
    pub challenge_text: Option<String>,
    /// Numeric rank (set by rank_set primitive).
    pub rank: Option<i64>,
}

/// A directed edge between two objects in the same world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLink {
    pub source_id: ObjectId,
    pub target_id: ObjectId,
    pub link_type: String,
    pub world_id: WorldId,
}
