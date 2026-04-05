use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chambers_types::capability::Principal;
use chambers_types::error::SubstrateError;
use chambers_types::object::{LifecycleClass, ObjectId};
use chambers_types::primitive::{
    Primitive, SealAuthorization, TransitionOperation, TransitionRequest,
};
use chambers_types::world::{LifecyclePhase, TerminationMode, WorldId};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateWorldRequest {
    pub grammar_id: String,
    pub objective: String,
}

#[derive(Serialize)]
pub struct CreateWorldResponse {
    pub world_id: String,
}

#[derive(Deserialize)]
pub struct AdvancePhaseRequest {
    pub phase: String,
}

#[derive(Deserialize)]
pub struct BurnRequest {
    pub mode: String,
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    pub operation: String,
    pub object_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub lifecycle_class: Option<String>,
    pub preservable: Option<bool>,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    pub link_type: Option<String>,
    pub challenge_text: Option<String>,
    pub object_ids: Option<Vec<String>>,
    pub rankings: Option<Vec<i64>>,
    pub synthesis_type: Option<String>,
    pub condensed_payload: Option<serde_json::Value>,
    pub authorization: Option<String>,
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn map_error(err: SubstrateError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match &err {
        SubstrateError::WorldNotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
        SubstrateError::WorldTerminated(_) => (StatusCode::GONE, err.to_string()),
        SubstrateError::WorldIdReuse(_) => (StatusCode::CONFLICT, err.to_string()),
        SubstrateError::ObjectNotFound { .. } => (StatusCode::NOT_FOUND, err.to_string()),
        SubstrateError::GrammarNotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
        SubstrateError::PolicyViolation(_)
        | SubstrateError::OperationNotPermittedInPhase { .. }
        | SubstrateError::MissingCapability { .. }
        | SubstrateError::CapabilityRevoked
        | SubstrateError::CapabilityExpired
        | SubstrateError::WrongEpoch { .. }
        | SubstrateError::WrongWorld
        | SubstrateError::SealUnauthorized
        | SubstrateError::NotPreservable { .. }
        | SubstrateError::ConvergenceFailed { .. }
        | SubstrateError::NoArtifactForPreservation
        | SubstrateError::InvalidLifecycleTransition { .. }
        | SubstrateError::DuplicateLink { .. }
        | SubstrateError::InvalidLinkType { .. } => {
            (StatusCode::UNPROCESSABLE_ENTITY, err.to_string())
        }
        SubstrateError::UnknownObjectType(_)
        | SubstrateError::InvalidPayload { .. }
        | SubstrateError::BinaryPayloadRejected { .. }
        | SubstrateError::CrossWorldAccess { .. } => {
            (StatusCode::BAD_REQUEST, err.to_string())
        }
        SubstrateError::BurnFailed { .. }
        | SubstrateError::CryptoOperationFailed { .. } => {
            (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        }
    };
    (status, Json(serde_json::json!({ "error": message })))
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn parse_world_id(id: &str) -> Result<WorldId, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(id)
        .map(WorldId)
        .map_err(|_| bad_request(format!("invalid world id: {}", id)))
}

fn parse_object_id(id: &str) -> Result<ObjectId, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(id)
        .map(ObjectId)
        .map_err(|_| bad_request(format!("invalid object id: {}", id)))
}

fn parse_lifecycle_class(s: &str) -> Result<LifecycleClass, (StatusCode, Json<serde_json::Value>)> {
    match s {
        "Temporary" => Ok(LifecycleClass::Temporary),
        "Intermediate" => Ok(LifecycleClass::Intermediate),
        "Candidate" => Ok(LifecycleClass::Candidate),
        "Preservable" => Ok(LifecycleClass::Preservable),
        other => Err(bad_request(format!("invalid lifecycle class: {}", other))),
    }
}

fn researcher() -> Principal {
    Principal::new("researcher")
}

// ---------------------------------------------------------------------------
// World management
// ---------------------------------------------------------------------------

/// POST /api/worlds
pub async fn create_world(
    State(state): State<AppState>,
    Json(body): Json<CreateWorldRequest>,
) -> impl IntoResponse {
    let rt = state.lock().await;
    let world_id = match rt.create_world(&body.grammar_id, &body.objective) {
        Ok(wid) => wid,
        Err(e) => return map_error(e).into_response(),
    };

    // Auto-issue Active capabilities (mirrors CLI behaviour)
    let active_prims = [
        Primitive::CreateObject,
        Primitive::LinkObjects,
        Primitive::ChallengeObject,
        Primitive::GenerateAlternative,
        Primitive::RankSet,
        Primitive::SynthesizeSet,
        Primitive::CondenseObject,
        Primitive::TriggerBurn,
    ];
    if let Err(e) = rt.issue_capabilities(world_id, researcher(), &active_prims) {
        return map_error(e).into_response();
    }

    (
        StatusCode::CREATED,
        Json(CreateWorldResponse {
            world_id: world_id.to_string(),
        }),
    )
        .into_response()
}

/// GET /api/worlds/:id
pub async fn get_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.world_engine.get_world(world_id) {
        Ok(world) => Json(serde_json::to_value(&world).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/grammars
pub async fn list_grammars(State(state): State<AppState>) -> impl IntoResponse {
    let rt = state.lock().await;
    let ids = rt.list_grammars();
    Json(serde_json::json!({ "grammars": ids }))
}

// ---------------------------------------------------------------------------
// State inspection
// ---------------------------------------------------------------------------

/// GET /api/worlds/:id/objects
pub async fn get_objects(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.view_engine.conversation_view(world_id) {
        Ok(view) => Json(serde_json::to_value(&view).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/worlds/:id/graph
pub async fn get_graph(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.view_engine.graph_view(world_id) {
        Ok(view) => Json(serde_json::to_value(&view).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/worlds/:id/summary
pub async fn get_summary(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.view_engine.summary_view(world_id) {
        Ok(view) => Json(serde_json::to_value(&view).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/worlds/:id/legal-actions
pub async fn get_legal_actions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.get_legal_actions(world_id) {
        Ok(actions) => Json(serde_json::to_value(&actions).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/worlds/:id/convergence
pub async fn get_convergence(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    match rt.get_convergence_state(world_id) {
        Ok(cs) => Json(serde_json::to_value(&cs).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// POST /api/worlds/:id/submit
/// Accepts TransitionOperation in serde enum format (e.g. { "CreateObject": { ... } })
/// OR the flat SubmitRequest format (e.g. { "operation": "create_object", ... }).
pub async fn submit_operation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };

    // Try serde enum format first (from UI), then flat format (from curl)
    let operation = if let Ok(op) = serde_json::from_value::<TransitionOperation>(body.clone()) {
        op
    } else if let Ok(req) = serde_json::from_value::<SubmitRequest>(body) {
        match build_transition_operation(&req) {
            Ok(op) => op,
            Err(e) => return e.into_response(),
        }
    } else {
        return bad_request("invalid operation format").into_response();
    };

    let request = TransitionRequest {
        world_id,
        principal: researcher(),
        operation,
    };

    let rt = state.lock().await;
    match rt.submit(&request) {
        Ok(result) => Json(serde_json::to_value(&result).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// POST /api/worlds/:id/advance
pub async fn advance_phase(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AdvancePhaseRequest>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };

    let phase = match body.phase.as_str() {
        "Active" => LifecyclePhase::Active,
        "ConvergenceReview" => LifecyclePhase::ConvergenceReview,
        "Finalization" => LifecyclePhase::Finalization,
        other => return bad_request(format!("invalid phase: {}", other)).into_response(),
    };

    let rt = state.lock().await;
    if let Err(e) = rt.advance_phase(world_id, phase) {
        return map_error(e).into_response();
    }

    // Re-issue capabilities for the new phase (mirrors CLI)
    let prims = match phase {
        LifecyclePhase::Active => vec![
            Primitive::CreateObject,
            Primitive::LinkObjects,
            Primitive::ChallengeObject,
            Primitive::GenerateAlternative,
            Primitive::RankSet,
            Primitive::SynthesizeSet,
            Primitive::CondenseObject,
            Primitive::TriggerBurn,
        ],
        LifecyclePhase::ConvergenceReview => vec![
            Primitive::ChallengeObject,
            Primitive::CondenseObject,
            Primitive::LinkObjects,
            Primitive::TriggerBurn,
        ],
        LifecyclePhase::Finalization => vec![
            Primitive::SealArtifact,
            Primitive::CondenseObject,
            Primitive::TriggerBurn,
        ],
        _ => vec![],
    };

    if !prims.is_empty() {
        if let Err(e) = rt.issue_capabilities(world_id, researcher(), &prims) {
            return map_error(e).into_response();
        }
    }

    Json(serde_json::json!({ "phase": format!("{:?}", phase) })).into_response()
}

// ---------------------------------------------------------------------------
// Burn and residue
// ---------------------------------------------------------------------------

/// POST /api/worlds/:id/burn
pub async fn burn_world(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<BurnRequest>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };

    let mode = match body.mode.as_str() {
        "AbortBurn" => TerminationMode::AbortBurn,
        "ConvergedPreserving" => TerminationMode::ConvergedPreserving,
        "ConvergedTotalBurn" => TerminationMode::ConvergedTotalBurn,
        other => return bad_request(format!("invalid burn mode: {}", other)).into_response(),
    };

    let request = TransitionRequest {
        world_id,
        principal: researcher(),
        operation: TransitionOperation::TriggerBurn { mode },
    };

    let rt = state.lock().await;
    match rt.submit(&request) {
        Ok(result) => Json(serde_json::to_value(&result).unwrap()).into_response(),
        Err(e) => map_error(e).into_response(),
    }
}

/// GET /api/worlds/:id/residue
pub async fn get_residue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    let report = rt.burn_engine.measure_residue(world_id);
    Json(serde_json::to_value(&report).unwrap()).into_response()
}

/// GET /api/worlds/:id/burn-view
pub async fn get_burn_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    let view = rt.view_engine.burn_view(world_id);
    Json(serde_json::to_value(&view).unwrap()).into_response()
}

// ---------------------------------------------------------------------------
// Vault and audit
// ---------------------------------------------------------------------------

/// GET /api/vault
pub async fn get_vault(State(state): State<AppState>) -> impl IntoResponse {
    let rt = state.lock().await;
    let artifacts = rt.vault.all_artifacts();
    Json(serde_json::to_value(&artifacts).unwrap())
}

/// GET /api/worlds/:id/audit
pub async fn get_audit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let world_id = match parse_world_id(&id) {
        Ok(wid) => wid,
        Err(e) => return e.into_response(),
    };
    let rt = state.lock().await;
    let events = rt.audit.events_for_world(world_id);
    Json(serde_json::to_value(&events).unwrap()).into_response()
}

// ---------------------------------------------------------------------------
// SubmitRequest -> TransitionOperation conversion
// ---------------------------------------------------------------------------

fn build_transition_operation(
    req: &SubmitRequest,
) -> Result<TransitionOperation, (StatusCode, Json<serde_json::Value>)> {
    match req.operation.as_str() {
        "create_object" => {
            let object_type = req
                .object_type
                .as_deref()
                .ok_or_else(|| bad_request("create_object requires object_type"))?
                .to_string();
            let payload = req
                .payload
                .clone()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            let lifecycle_class = match req.lifecycle_class.as_deref() {
                Some(lc) => parse_lifecycle_class(lc)?,
                None => LifecycleClass::Temporary,
            };
            let preservable = req.preservable.unwrap_or(false);
            Ok(TransitionOperation::CreateObject {
                object_type,
                payload,
                lifecycle_class,
                preservable,
            })
        }
        "link_objects" => {
            let source_id = parse_object_id(
                req.source_id
                    .as_deref()
                    .ok_or_else(|| bad_request("link_objects requires source_id"))?,
            )?;
            let target_id = parse_object_id(
                req.target_id
                    .as_deref()
                    .ok_or_else(|| bad_request("link_objects requires target_id"))?,
            )?;
            let link_type = req
                .link_type
                .clone()
                .ok_or_else(|| bad_request("link_objects requires link_type"))?;
            Ok(TransitionOperation::LinkObjects {
                source_id,
                target_id,
                link_type,
            })
        }
        "challenge_object" => {
            let target_id = parse_object_id(
                req.target_id
                    .as_deref()
                    .ok_or_else(|| bad_request("challenge_object requires target_id"))?,
            )?;
            let challenge_text = req
                .challenge_text
                .clone()
                .ok_or_else(|| bad_request("challenge_object requires challenge_text"))?;
            Ok(TransitionOperation::ChallengeObject {
                target_id,
                challenge_text,
            })
        }
        "generate_alternative" => {
            let target_id = parse_object_id(
                req.target_id
                    .as_deref()
                    .ok_or_else(|| bad_request("generate_alternative requires target_id"))?,
            )?;
            let alternative_payload = req
                .payload
                .clone()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            Ok(TransitionOperation::GenerateAlternative {
                target_id,
                alternative_payload,
            })
        }
        "rank_set" => {
            let object_ids_raw = req
                .object_ids
                .as_ref()
                .ok_or_else(|| bad_request("rank_set requires object_ids"))?;
            let rankings = req
                .rankings
                .as_ref()
                .ok_or_else(|| bad_request("rank_set requires rankings"))?
                .clone();
            let object_ids: Result<Vec<ObjectId>, _> =
                object_ids_raw.iter().map(|s| parse_object_id(s)).collect();
            let object_ids = object_ids?;
            Ok(TransitionOperation::RankSet {
                object_ids,
                rankings,
            })
        }
        "synthesize_set" => {
            let object_ids_raw = req
                .object_ids
                .as_ref()
                .ok_or_else(|| bad_request("synthesize_set requires object_ids"))?;
            let source_ids: Result<Vec<ObjectId>, _> =
                object_ids_raw.iter().map(|s| parse_object_id(s)).collect();
            let source_ids = source_ids?;
            let synthesis_type = req
                .synthesis_type
                .clone()
                .ok_or_else(|| bad_request("synthesize_set requires synthesis_type"))?;
            let synthesis_payload = req
                .payload
                .clone()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            Ok(TransitionOperation::SynthesizeSet {
                source_ids,
                synthesis_type,
                synthesis_payload,
            })
        }
        "condense_object" => {
            let target_id = parse_object_id(
                req.target_id
                    .as_deref()
                    .ok_or_else(|| bad_request("condense_object requires target_id"))?,
            )?;
            let condensed_payload = req
                .condensed_payload
                .clone()
                .or_else(|| req.payload.clone())
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            Ok(TransitionOperation::CondenseObject {
                target_id,
                condensed_payload,
            })
        }
        "seal_artifact" => {
            let target_id = parse_object_id(
                req.target_id
                    .as_deref()
                    .ok_or_else(|| bad_request("seal_artifact requires target_id"))?,
            )?;
            let confirmer = req
                .authorization
                .as_deref()
                .unwrap_or("researcher")
                .to_string();
            let authorization = SealAuthorization::HumanConfirmed { confirmer };
            Ok(TransitionOperation::SealArtifact {
                target_id,
                authorization,
            })
        }
        other => Err(bad_request(format!("unknown operation: {}", other))),
    }
}
