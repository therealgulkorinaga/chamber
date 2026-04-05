use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique, non-reusable identifier for a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub Uuid);

impl WorldId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl std::fmt::Display for WorldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle phases a world passes through.
/// Transitions are strictly forward-only: Created → Active → ConvergenceReview → Finalization → Terminated.
/// Abort is possible from any non-terminated state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhase {
    Created,
    Active,
    ConvergenceReview,
    Finalization,
    Terminated,
}

impl LifecyclePhase {
    /// Returns whether transitioning from `self` to `target` is legal.
    pub fn can_transition_to(&self, target: LifecyclePhase) -> bool {
        use LifecyclePhase::*;
        matches!(
            (self, target),
            (Created, Active)
                | (Active, ConvergenceReview)
                | (ConvergenceReview, Active) // rejection → rework
                | (ConvergenceReview, Finalization)
                | (Finalization, Terminated)
                // Abort path: any non-terminated state can go to Terminated
                | (Created, Terminated)
                | (Active, Terminated)
                | (ConvergenceReview, Terminated)
        )
    }

    /// Returns the epoch index associated with this phase.
    pub fn epoch_index(&self) -> u32 {
        match self {
            LifecyclePhase::Created => 0,
            LifecyclePhase::Active => 1,
            LifecyclePhase::ConvergenceReview => 2,
            LifecyclePhase::Finalization => 3,
            LifecyclePhase::Terminated => 4,
        }
    }
}

/// How a world terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerminationMode {
    /// One artifact preserved, everything else burned.
    ConvergedPreserving,
    /// Convergence reached but nothing preserved — total burn.
    ConvergedTotalBurn,
    /// Aborted before convergence — total burn, no artifact.
    AbortBurn,
}

/// Reference to a cryptographic key (never the key material itself).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRef(pub String);

/// A world — the primary semantic unit in Chambers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub world_id: WorldId,
    pub grammar_id: String,
    pub objective: String,
    pub lifecycle_phase: LifecyclePhase,
    pub epoch: u32,
    pub world_key_ref: KeyRef,
    pub artifact_key_ref: Option<KeyRef>,
    pub created_at: DateTime<Utc>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub termination_mode: Option<TerminationMode>,
}
