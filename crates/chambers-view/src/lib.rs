//! View layer for Chambers.
//!
//! Read-only projections of world state.
//! Views do not mutate state and do not persist independently.

use chambers_state::StateEngine;
use chambers_types::object::{Object, ObjectLink};
use chambers_types::world::WorldId;
use chambers_types::error::SubstrateResult;
use chambers_vault::ArtifactVault;
use serde::Serialize;
use std::sync::Arc;

/// View engine — provides read-only projections.
#[derive(Debug)]
pub struct ViewEngine {
    state: Arc<StateEngine>,
    vault: Arc<ArtifactVault>,
}

impl ViewEngine {
    pub fn new(state: Arc<StateEngine>, vault: Arc<ArtifactVault>) -> Self {
        Self { state, vault }
    }

    /// Conversation view: chronological list of objects.
    pub fn conversation_view(&self, world_id: WorldId) -> SubstrateResult<ConversationView> {
        let mut objects = self.state.all_objects_decrypted(world_id)?;
        objects.sort_by_key(|o| o.created_at);
        Ok(ConversationView {
            world_id,
            entries: objects
                .iter()
                .map(|o| ConversationEntry {
                    object_id: o.object_id.to_string(),
                    object_type: o.object_type.clone(),
                    payload_summary: payload_summary(&o.payload),
                    payload: o.payload.clone(),
                    lifecycle_class: format!("{:?}", o.lifecycle_class),
                    preservable: o.preservable,
                    challenged: o.challenged,
                    created_at: o.created_at.to_rfc3339(),
                })
                .collect(),
        })
    }

    /// Graph view: objects and links.
    pub fn graph_view(&self, world_id: WorldId) -> SubstrateResult<GraphView> {
        let objects = self.state.all_objects_decrypted(world_id)?;
        let links = self.state.all_links_decrypted(world_id)?;
        Ok(GraphView {
            world_id,
            nodes: objects
                .iter()
                .map(|o| GraphNode {
                    id: o.object_id.to_string(),
                    object_type: o.object_type.clone(),
                    lifecycle_class: format!("{:?}", o.lifecycle_class),
                    preservable: o.preservable,
                    payload: o.payload.clone(),
                    challenged: o.challenged,
                })
                .collect(),
            edges: links
                .iter()
                .map(|l| GraphEdge {
                    source: l.source_id.to_string(),
                    target: l.target_id.to_string(),
                    link_type: l.link_type.clone(),
                })
                .collect(),
        })
    }

    /// Summary view: counts and phase info.
    pub fn summary_view(&self, world_id: WorldId) -> SubstrateResult<SummaryView> {
        let objects = self.state.all_objects_decrypted(world_id)?;
        let mut type_counts = std::collections::HashMap::new();
        for obj in &objects {
            *type_counts.entry(obj.object_type.clone()).or_insert(0) += 1;
        }
        let object_count = self.state.object_count(world_id)?;
        let link_count = self.state.link_count(world_id)?;
        let has_unresolved_challenges = self.state.has_unresolved_challenges(world_id)?;
        Ok(SummaryView {
            world_id,
            object_count,
            link_count,
            type_counts,
            has_unresolved_challenges,
        })
    }

    /// Burn view: post-burn report.
    pub fn burn_view(&self, world_id: WorldId) -> BurnView {
        let artifacts = self.vault.artifacts_from_world(world_id);
        BurnView {
            world_id,
            artifacts_preserved: artifacts.len(),
            artifact_classes: artifacts.iter().map(|a| a.artifact_class.clone()).collect(),
            world_state_destroyed: !self.state.has_world(world_id),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConversationView {
    pub world_id: WorldId,
    pub entries: Vec<ConversationEntry>,
}

#[derive(Debug, Serialize)]
pub struct ConversationEntry {
    pub object_id: String,
    pub object_type: String,
    pub payload_summary: String,
    pub payload: serde_json::Value,
    pub lifecycle_class: String,
    pub preservable: bool,
    pub challenged: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct GraphView {
    pub world_id: WorldId,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub object_type: String,
    pub lifecycle_class: String,
    pub preservable: bool,
    pub payload: serde_json::Value,
    pub challenged: bool,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub link_type: String,
}

#[derive(Debug, Serialize)]
pub struct SummaryView {
    pub world_id: WorldId,
    pub object_count: usize,
    pub link_count: usize,
    pub type_counts: std::collections::HashMap<String, usize>,
    pub has_unresolved_challenges: bool,
}

#[derive(Debug, Serialize)]
pub struct BurnView {
    pub world_id: WorldId,
    pub artifacts_preserved: usize,
    pub artifact_classes: Vec<String>,
    pub world_state_destroyed: bool,
}

fn payload_summary(payload: &serde_json::Value) -> String {
    match payload {
        serde_json::Value::String(s) => {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }
        serde_json::Value::Object(m) => {
            let keys: Vec<&String> = m.keys().take(5).collect();
            format!("{{{}}}", keys.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", "))
        }
        other => format!("{}", other),
    }
}
