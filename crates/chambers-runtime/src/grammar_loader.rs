//! Grammar loader — reads grammar JSON files into ChamberGrammar structs.

use chambers_types::grammar::*;
use chambers_types::object::LifecycleClass;
use chambers_types::primitive::Primitive;
use chambers_types::world::TerminationMode;
use std::collections::HashMap;

/// Build the Decision Chamber grammar programmatically.
/// This mirrors the JSON definition in grammars/decision_chamber.json.
pub fn decision_chamber_grammar() -> ChamberGrammar {
    let mut object_types = HashMap::new();

    // Helper to create object type specs
    let add_type = |types: &mut HashMap<String, ObjectTypeSpec>,
                    name: &str,
                    max_bytes: usize,
                    transforms: Vec<Primitive>,
                    default_lc: LifecycleClass,
                    preservable: bool| {
        types.insert(
            name.to_string(),
            ObjectTypeSpec {
                type_name: name.to_string(),
                payload_schema: serde_json::json!({"type": "object"}),
                max_payload_bytes: max_bytes,
                transform_set: transforms,
                default_lifecycle_class: default_lc,
                can_be_preservable: preservable,
            },
        );
    };

    add_type(
        &mut object_types,
        "decision_objective",
        10000,
        vec![Primitive::CondenseObject],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "premise",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "support_statement",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "constraint",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "risk",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::RankSet,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "upside",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::RankSet,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "contradiction",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
        ],
        LifecycleClass::Temporary,
        false,
    );
    add_type(
        &mut object_types,
        "alternative",
        5000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::RankSet,
            Primitive::GenerateAlternative,
        ],
        LifecycleClass::Intermediate,
        false,
    );
    add_type(
        &mut object_types,
        "recommendation",
        10000,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::SynthesizeSet,
        ],
        LifecycleClass::Candidate,
        false,
    );
    add_type(
        &mut object_types,
        "decision_summary",
        20000,
        vec![Primitive::CondenseObject, Primitive::SealArtifact],
        LifecycleClass::Preservable,
        true,
    );

    let mut phase_primitives = HashMap::new();
    phase_primitives.insert(
        LifecyclePhaseKey::Created,
        vec![Primitive::CreateObject],
    );
    phase_primitives.insert(
        LifecyclePhaseKey::Active,
        vec![
            Primitive::CreateObject,
            Primitive::LinkObjects,
            Primitive::ChallengeObject,
            Primitive::GenerateAlternative,
            Primitive::RankSet,
            Primitive::SynthesizeSet,
            Primitive::CondenseObject,
            Primitive::TriggerBurn,
        ],
    );
    phase_primitives.insert(
        LifecyclePhaseKey::ConvergenceReview,
        vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::TriggerBurn,
        ],
    );
    phase_primitives.insert(
        LifecyclePhaseKey::Finalization,
        vec![
            Primitive::SealArtifact,
            Primitive::CondenseObject,
            Primitive::TriggerBurn,
        ],
    );

    ChamberGrammar {
        grammar_id: "decision_chamber_v1".to_string(),
        name: "Decision Chamber".to_string(),
        description: "A chamber for structured decision-making. Only the final decision_summary may survive.".to_string(),
        objective_class: "decision_objective".to_string(),
        object_types,
        phase_primitives,
        preservable_classes: vec!["decision_summary".to_string()],
        allowed_views: vec![
            "conversation".to_string(),
            "graph".to_string(),
            "summary".to_string(),
            "burn".to_string(),
        ],
        termination_modes: vec![
            TerminationLaw {
                mode: TerminationMode::ConvergedPreserving,
                description: "Preserve one decision_summary and burn all else.".to_string(),
                requires_artifact: true,
            },
            TerminationLaw {
                mode: TerminationMode::ConvergedTotalBurn,
                description: "Convergence reached but nothing preserved — total burn.".to_string(),
                requires_artifact: false,
            },
            TerminationLaw {
                mode: TerminationMode::AbortBurn,
                description: "Aborted before convergence — total burn, no artifact.".to_string(),
                requires_artifact: false,
            },
        ],
        convergence_criteria: ConvergenceCriteria {
            required_types: vec!["decision_summary".to_string()],
            challenges_block_convergence: true,
            contradictions_block_convergence: true,
        },
        permitted_links: vec![
            LinkSpec {
                link_type: "supports".to_string(),
                source_types: vec!["support_statement".to_string()],
                target_types: vec!["premise".to_string(), "alternative".to_string(), "recommendation".to_string()],
            },
            LinkSpec {
                link_type: "constrains".to_string(),
                source_types: vec!["constraint".to_string()],
                target_types: vec!["alternative".to_string(), "recommendation".to_string(), "decision_objective".to_string()],
            },
            LinkSpec {
                link_type: "risks".to_string(),
                source_types: vec!["risk".to_string()],
                target_types: vec!["alternative".to_string(), "recommendation".to_string()],
            },
            LinkSpec {
                link_type: "benefits".to_string(),
                source_types: vec!["upside".to_string()],
                target_types: vec!["alternative".to_string(), "recommendation".to_string()],
            },
            LinkSpec {
                link_type: "contradicts".to_string(),
                source_types: vec!["contradiction".to_string()],
                target_types: vec!["premise".to_string(), "support_statement".to_string(), "alternative".to_string()],
            },
            LinkSpec {
                link_type: "alternative_to".to_string(),
                source_types: vec!["alternative".to_string()],
                target_types: vec!["alternative".to_string(), "recommendation".to_string(), "decision_objective".to_string()],
            },
            LinkSpec {
                link_type: "synthesized_from".to_string(),
                source_types: vec!["recommendation".to_string(), "decision_summary".to_string()],
                target_types: vec![
                    "premise".to_string(), "support_statement".to_string(),
                    "constraint".to_string(), "risk".to_string(),
                    "upside".to_string(), "alternative".to_string(),
                ],
            },
            LinkSpec {
                link_type: "based_on".to_string(),
                source_types: vec!["decision_summary".to_string()],
                target_types: vec!["recommendation".to_string()],
            },
        ],
    }
}
