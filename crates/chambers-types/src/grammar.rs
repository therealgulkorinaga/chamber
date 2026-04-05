use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::primitive::Primitive;
use crate::world::LifecyclePhase;

/// A chamber grammar definition — the declarative specification
/// of what a chamber allows, enforces, and preserves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberGrammar {
    pub grammar_id: String,
    pub name: String,
    pub description: String,

    /// The objective class for this chamber type.
    pub objective_class: String,

    /// Permitted object types and their schemas.
    pub object_types: HashMap<String, ObjectTypeSpec>,

    /// Which primitives are permitted in which lifecycle phases.
    pub phase_primitives: HashMap<LifecyclePhaseKey, Vec<Primitive>>,

    /// Which object types may survive world termination.
    pub preservable_classes: Vec<String>,

    /// Allowed view types.
    pub allowed_views: Vec<String>,

    /// Termination law: legal termination modes.
    pub termination_modes: Vec<TerminationLaw>,

    /// Convergence criteria.
    pub convergence_criteria: ConvergenceCriteria,

    /// Permitted link types between object types.
    pub permitted_links: Vec<LinkSpec>,
}

/// Specification of an object type within a grammar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTypeSpec {
    pub type_name: String,
    /// JSON Schema for payload validation (as serde_json::Value).
    pub payload_schema: serde_json::Value,
    /// Maximum payload size in bytes.
    pub max_payload_bytes: usize,
    /// Which primitives may target objects of this type.
    pub transform_set: Vec<Primitive>,
    /// Default lifecycle class for new objects of this type.
    pub default_lifecycle_class: crate::object::LifecycleClass,
    /// Whether objects of this type can be marked preservable.
    pub can_be_preservable: bool,
}

/// Key for phase-specific primitive permissions.
/// Serialized as string for HashMap compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhaseKey {
    Created,
    Active,
    ConvergenceReview,
    Finalization,
}

impl From<LifecyclePhase> for LifecyclePhaseKey {
    fn from(phase: LifecyclePhase) -> Self {
        match phase {
            LifecyclePhase::Created => LifecyclePhaseKey::Created,
            LifecyclePhase::Active => LifecyclePhaseKey::Active,
            LifecyclePhase::ConvergenceReview => LifecyclePhaseKey::ConvergenceReview,
            LifecyclePhase::Finalization => LifecyclePhaseKey::Finalization,
            LifecyclePhase::Terminated => LifecyclePhaseKey::Finalization, // no ops in terminated
        }
    }
}

/// A legal termination mode for this grammar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationLaw {
    pub mode: crate::world::TerminationMode,
    /// Description of what this mode means for this grammar.
    pub description: String,
    /// Whether an artifact is required for this mode.
    pub requires_artifact: bool,
}

/// Criteria the convergence checker evaluates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceCriteria {
    /// Object types that must exist for convergence.
    pub required_types: Vec<String>,
    /// Whether unresolved challenges block convergence.
    pub challenges_block_convergence: bool,
    /// Whether unresolved contradictions block convergence.
    pub contradictions_block_convergence: bool,
}

/// Specification of a permitted link between object types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSpec {
    pub link_type: String,
    pub source_types: Vec<String>,
    pub target_types: Vec<String>,
}
