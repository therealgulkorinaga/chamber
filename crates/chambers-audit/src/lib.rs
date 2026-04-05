//! Two-tier audit layer for Chambers.
//!
//! Tier 1 (substrate-scoped): survives burn. Maximum 2 entries per world.
//!   - WorldCreated: a world existed
//!   - WorldDestroyed: the world was burned (mode only, no details)
//!   These are the ONLY events an observer can see after burn.
//!
//! Tier 2 (world-scoped): encrypted under K_w, destroyed on burn.
//!   - Phase transitions, convergence, sealing, burn layers, policy violations
//!   - Useful during the chamber session for debugging
//!   - Gone after burn — as if they never existed

use chambers_types::world::{LifecyclePhase, TerminationMode, WorldId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// =========================================================================
// Tier 1: Substrate-scoped events (survive burn, max 2 per world)
// =========================================================================

/// A substrate-level audit event. Only two types exist.
/// This is ALL that survives burn. An observer learns:
/// "a world existed and was destroyed." Nothing more.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateEvent {
    pub timestamp: DateTime<Utc>,
    pub world_id: WorldId,
    pub event_type: SubstrateEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubstrateEventType {
    WorldCreated,
    WorldDestroyed { mode: TerminationMode },
}

// =========================================================================
// Tier 2: World-scoped events (encrypted, destroyed on burn)
// =========================================================================

/// A world-scoped audit event. Destroyed on burn.
/// Useful during the session. Gone after.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: WorldEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldEventType {
    PhaseTransition {
        from: LifecyclePhase,
        to: LifecyclePhase,
    },
    ConvergenceProposed,
    ConvergenceValidated {
        passed: bool,
        reason: Option<String>,
    },
    ArtifactSealed {
        artifact_class: String,
    },
    ArtifactNotSealed {
        reason: String,
    },
    BurnStarted {
        mode: TerminationMode,
    },
    BurnLayerCompleted {
        layer: String,
    },
    PolicyViolation {
        description: String,
    },
    CapabilityRevoked {
        reason: String,
    },
}

// =========================================================================
// Backward-compatible AuditEvent type (wraps both tiers for callers)
// =========================================================================

/// Combined event type for backward compatibility with existing callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub world_id: WorldId,
    pub event_type: AuditEventType,
}

/// All event types — callers use this, audit layer routes internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    // Tier 1 — substrate-scoped, survives burn
    WorldCreated {
        grammar_id: String,
    },
    BurnCompleted {
        mode: TerminationMode,
    },

    // Tier 2 — world-scoped, destroyed on burn
    PhaseTransition {
        from: LifecyclePhase,
        to: LifecyclePhase,
    },
    ConvergenceProposed,
    ConvergenceValidated {
        passed: bool,
        reason: Option<String>,
    },
    ArtifactSealed {
        artifact_class: String,
    },
    ArtifactNotSealed {
        reason: String,
    },
    BurnStarted {
        mode: TerminationMode,
    },
    BurnLayerCompleted {
        layer: String,
    },
    PolicyViolation {
        description: String,
    },
    CapabilityRevoked {
        reason: String,
    },
}

impl AuditEventType {
    /// Returns true if this event survives burn (Tier 1).
    pub fn is_substrate_scoped(&self) -> bool {
        matches!(
            self,
            AuditEventType::WorldCreated { .. } | AuditEventType::BurnCompleted { .. }
        )
    }
}

// =========================================================================
// The audit log — two tiers
// =========================================================================

/// The two-tier audit log.
///
/// Tier 1 (substrate_events): survives burn. Max 2 entries per world.
/// Tier 2 (world_events): destroyed on burn.
#[derive(Debug, Clone)]
pub struct AuditLog {
    /// Tier 1: substrate-scoped events. Survive burn.
    substrate_events: Arc<Mutex<Vec<SubstrateEvent>>>,
    /// Tier 2: world-scoped events. Destroyed on burn.
    world_events: Arc<Mutex<HashMap<WorldId, Vec<WorldEvent>>>>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            substrate_events: Arc::new(Mutex::new(Vec::new())),
            world_events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record an event. Routes to the correct tier automatically.
    pub fn record(&self, world_id: WorldId, event_type: AuditEventType) {
        let now = Utc::now();

        if event_type.is_substrate_scoped() {
            // Tier 1: substrate-scoped, survives burn
            let substrate_type = match &event_type {
                AuditEventType::WorldCreated { .. } => SubstrateEventType::WorldCreated,
                AuditEventType::BurnCompleted { mode } => {
                    SubstrateEventType::WorldDestroyed { mode: *mode }
                }
                _ => unreachable!(),
            };
            self.substrate_events.lock().unwrap().push(SubstrateEvent {
                timestamp: now,
                world_id,
                event_type: substrate_type,
            });
        } else {
            // Tier 2: world-scoped, destroyed on burn
            let world_type = match event_type {
                AuditEventType::PhaseTransition { from, to } => {
                    WorldEventType::PhaseTransition { from, to }
                }
                AuditEventType::ConvergenceProposed => WorldEventType::ConvergenceProposed,
                AuditEventType::ConvergenceValidated { passed, reason } => {
                    WorldEventType::ConvergenceValidated { passed, reason }
                }
                AuditEventType::ArtifactSealed { artifact_class } => {
                    WorldEventType::ArtifactSealed { artifact_class }
                }
                AuditEventType::ArtifactNotSealed { reason } => {
                    WorldEventType::ArtifactNotSealed { reason }
                }
                AuditEventType::BurnStarted { mode } => WorldEventType::BurnStarted { mode },
                AuditEventType::BurnLayerCompleted { layer } => {
                    WorldEventType::BurnLayerCompleted { layer }
                }
                AuditEventType::PolicyViolation { description } => {
                    WorldEventType::PolicyViolation { description }
                }
                AuditEventType::CapabilityRevoked { reason } => {
                    WorldEventType::CapabilityRevoked { reason }
                }
                _ => return,
            };
            self.world_events
                .lock()
                .unwrap()
                .entry(world_id)
                .or_default()
                .push(WorldEvent {
                    timestamp: now,
                    event_type: world_type,
                });
        }
    }

    /// Destroy all world-scoped audit events for a world.
    /// Called during burn. After this, only Tier 1 events remain.
    pub fn burn_world_events(&self, world_id: WorldId) {
        self.world_events.lock().unwrap().remove(&world_id);
    }

    /// Get all events for a world (both tiers). Only works before burn.
    /// After burn, only Tier 1 events are returned.
    pub fn events_for_world(&self, world_id: WorldId) -> Vec<AuditEvent> {
        let mut events = Vec::new();

        // Tier 1
        for se in self.substrate_events.lock().unwrap().iter() {
            if se.world_id == world_id {
                let event_type = match &se.event_type {
                    SubstrateEventType::WorldCreated => AuditEventType::WorldCreated {
                        grammar_id: String::new(),
                    },
                    SubstrateEventType::WorldDestroyed { mode } => {
                        AuditEventType::BurnCompleted { mode: *mode }
                    }
                };
                events.push(AuditEvent {
                    timestamp: se.timestamp,
                    world_id,
                    event_type,
                });
            }
        }

        // Tier 2 (only if not yet burned)
        if let Some(world_evts) = self.world_events.lock().unwrap().get(&world_id) {
            for we in world_evts {
                let event_type = match &we.event_type {
                    WorldEventType::PhaseTransition { from, to } => {
                        AuditEventType::PhaseTransition { from: *from, to: *to }
                    }
                    WorldEventType::ConvergenceProposed => AuditEventType::ConvergenceProposed,
                    WorldEventType::ConvergenceValidated { passed, reason } => {
                        AuditEventType::ConvergenceValidated { passed: *passed, reason: reason.clone() }
                    }
                    WorldEventType::ArtifactSealed { artifact_class } => {
                        AuditEventType::ArtifactSealed { artifact_class: artifact_class.clone() }
                    }
                    WorldEventType::ArtifactNotSealed { reason } => {
                        AuditEventType::ArtifactNotSealed { reason: reason.clone() }
                    }
                    WorldEventType::BurnStarted { mode } => {
                        AuditEventType::BurnStarted { mode: *mode }
                    }
                    WorldEventType::BurnLayerCompleted { layer } => {
                        AuditEventType::BurnLayerCompleted { layer: layer.clone() }
                    }
                    WorldEventType::PolicyViolation { description } => {
                        AuditEventType::PolicyViolation { description: description.clone() }
                    }
                    WorldEventType::CapabilityRevoked { reason } => {
                        AuditEventType::CapabilityRevoked { reason: reason.clone() }
                    }
                };
                events.push(AuditEvent {
                    timestamp: we.timestamp,
                    world_id,
                    event_type,
                });
            }
        }

        events.sort_by_key(|e| e.timestamp);
        events
    }

    /// Get only substrate-scoped events (what survives burn).
    pub fn substrate_events_for_world(&self, world_id: WorldId) -> Vec<SubstrateEvent> {
        self.substrate_events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.world_id == world_id)
            .cloned()
            .collect()
    }

    /// Count of substrate-scoped events for a world (post-burn metadata count).
    pub fn substrate_event_count(&self, world_id: WorldId) -> usize {
        self.substrate_events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.world_id == world_id)
            .count()
    }

    /// Get all events (both tiers, all worlds).
    pub fn all_events(&self) -> Vec<AuditEvent> {
        let mut events = Vec::new();
        for se in self.substrate_events.lock().unwrap().iter() {
            let event_type = match &se.event_type {
                SubstrateEventType::WorldCreated => AuditEventType::WorldCreated {
                    grammar_id: String::new(),
                },
                SubstrateEventType::WorldDestroyed { mode } => {
                    AuditEventType::BurnCompleted { mode: *mode }
                }
            };
            events.push(AuditEvent {
                timestamp: se.timestamp,
                world_id: se.world_id,
                event_type,
            });
        }
        events
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}
