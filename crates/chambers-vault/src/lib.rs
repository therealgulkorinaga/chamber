//! Artifact vault for Chambers.
//!
//! The sole authorized cross-world channel.
//! Stores only sealed artifacts with minimal provenance.
//! World internals never enter the vault.

use chambers_types::artifact::*;
use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::world::WorldId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The artifact vault — persists across world burns.
#[derive(Debug, Clone)]
pub struct ArtifactVault {
    artifacts: Arc<Mutex<HashMap<ArtifactId, Artifact>>>,
}

impl ArtifactVault {
    pub fn new() -> Self {
        Self {
            artifacts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store a sealed artifact. Only callable through the SealArtifact primitive path.
    pub fn store_artifact(&self, artifact: Artifact) -> SubstrateResult<ArtifactId> {
        let id = artifact.artifact_id;
        self.artifacts.lock().unwrap().insert(id, artifact);
        Ok(id)
    }

    /// Retrieve an artifact by ID.
    pub fn get_artifact(&self, artifact_id: ArtifactId) -> Option<Artifact> {
        self.artifacts.lock().unwrap().get(&artifact_id).cloned()
    }

    /// List all artifacts from a specific source world.
    pub fn artifacts_from_world(&self, world_id: WorldId) -> Vec<Artifact> {
        self.artifacts
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.source_world_id == world_id)
            .cloned()
            .collect()
    }

    /// Count artifacts from a specific world.
    pub fn artifact_count_for_world(&self, world_id: WorldId) -> usize {
        self.artifacts
            .lock()
            .unwrap()
            .values()
            .filter(|a| a.source_world_id == world_id)
            .count()
    }

    /// List all artifacts.
    pub fn all_artifacts(&self) -> Vec<Artifact> {
        self.artifacts.lock().unwrap().values().cloned().collect()
    }
}

impl Default for ArtifactVault {
    fn default() -> Self {
        Self::new()
    }
}
