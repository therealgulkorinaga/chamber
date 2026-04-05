use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::world::WorldId;

/// Unique identifier for a sealed artifact in the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub Uuid);

impl ArtifactId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

/// A sealed artifact that survived world termination.
/// This is the ONLY form of data that may cross world boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: ArtifactId,
    pub source_world_id: WorldId,
    pub artifact_class: String,
    pub payload: serde_json::Value,
    pub sealed_at: DateTime<Utc>,
    /// Minimal provenance — no world internals.
    pub provenance_metadata: ProvenanceMetadata,
    pub vault_policy_class: String,
}

/// Minimal provenance metadata for a sealed artifact.
/// Deliberately constrained to prevent world-internal leakage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    pub grammar_id: String,
    pub objective_summary: String,
    pub world_created_at: DateTime<Utc>,
    pub world_terminated_at: DateTime<Utc>,
}
