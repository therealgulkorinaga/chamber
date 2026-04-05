use serde::{Deserialize, Serialize};

use crate::capability::Principal;
use crate::object::ObjectId;
use crate::world::WorldId;

/// The closed, finite set of primitive operations.
/// No dynamic primitive creation is permitted.
/// All world evolution occurs through these primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Primitive {
    CreateObject,
    LinkObjects,
    ChallengeObject,
    GenerateAlternative,
    RankSet,
    SynthesizeSet,
    CondenseObject,
    SealArtifact,
    TriggerBurn,
}

impl Primitive {
    /// All primitives in the algebra.
    pub const ALL: &'static [Primitive] = &[
        Primitive::CreateObject,
        Primitive::LinkObjects,
        Primitive::ChallengeObject,
        Primitive::GenerateAlternative,
        Primitive::RankSet,
        Primitive::SynthesizeSet,
        Primitive::CondenseObject,
        Primitive::SealArtifact,
        Primitive::TriggerBurn,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Primitive::CreateObject => "create_object",
            Primitive::LinkObjects => "link_objects",
            Primitive::ChallengeObject => "challenge_object",
            Primitive::GenerateAlternative => "generate_alternative",
            Primitive::RankSet => "rank_set",
            Primitive::SynthesizeSet => "synthesize_set",
            Primitive::CondenseObject => "condense_object",
            Primitive::SealArtifact => "seal_artifact",
            Primitive::TriggerBurn => "trigger_burn",
        }
    }
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A transition request submitted to the interpreter.
/// This is the ONLY way to evolve world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    pub world_id: WorldId,
    pub principal: Principal,
    pub operation: TransitionOperation,
}

/// The specific operation requested, with typed parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionOperation {
    CreateObject {
        object_type: String,
        payload: serde_json::Value,
        lifecycle_class: crate::object::LifecycleClass,
        preservable: bool,
    },
    LinkObjects {
        source_id: ObjectId,
        target_id: ObjectId,
        link_type: String,
    },
    ChallengeObject {
        target_id: ObjectId,
        challenge_text: String,
    },
    GenerateAlternative {
        target_id: ObjectId,
        alternative_payload: serde_json::Value,
    },
    RankSet {
        object_ids: Vec<ObjectId>,
        rankings: Vec<i64>,
    },
    SynthesizeSet {
        source_ids: Vec<ObjectId>,
        synthesis_type: String,
        synthesis_payload: serde_json::Value,
    },
    CondenseObject {
        target_id: ObjectId,
        condensed_payload: serde_json::Value,
    },
    SealArtifact {
        target_id: ObjectId,
        authorization: SealAuthorization,
    },
    TriggerBurn {
        mode: crate::world::TerminationMode,
    },
}

impl TransitionOperation {
    /// Returns which primitive this operation corresponds to.
    pub fn primitive(&self) -> Primitive {
        match self {
            TransitionOperation::CreateObject { .. } => Primitive::CreateObject,
            TransitionOperation::LinkObjects { .. } => Primitive::LinkObjects,
            TransitionOperation::ChallengeObject { .. } => Primitive::ChallengeObject,
            TransitionOperation::GenerateAlternative { .. } => Primitive::GenerateAlternative,
            TransitionOperation::RankSet { .. } => Primitive::RankSet,
            TransitionOperation::SynthesizeSet { .. } => Primitive::SynthesizeSet,
            TransitionOperation::CondenseObject { .. } => Primitive::CondenseObject,
            TransitionOperation::SealArtifact { .. } => Primitive::SealArtifact,
            TransitionOperation::TriggerBurn { .. } => Primitive::TriggerBurn,
        }
    }
}

/// Authorization for sealing an artifact.
/// The model alone cannot unilaterally seal — requires explicit authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SealAuthorization {
    /// Human confirmed via CLI or other interface.
    HumanConfirmed { confirmer: String },
    /// Policy engine auto-approved (for testing).
    PolicyApproved { policy_rule: String },
}
