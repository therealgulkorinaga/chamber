//! Chambers native app.
//!
//! Lobby = main window. Each chamber = isolated native webview.
//! No address bar, no history, no nav buttons.
//! Burn = window destroyed from Rust side.
//!
//! The HTTP adapter runs in background for chamber webviews to fetch from.

mod ipc;
mod ui_lobby;
mod ui_chamber;

use chambers_runtime::grammar_loader::decision_chamber_grammar;
use chambers_runtime::Runtime;
use chambers_types::capability::Principal;
use chambers_types::primitive::Primitive;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{WindowBuilder, WindowId};
use wry::WebViewBuilder;

/// Custom events sent from IPC/threads to the event loop.
#[derive(Debug)]
enum AppEvent {
    OpenChamber { world_id: String, demo: bool },
    CloseChamber { world_id: String },
    Quit,
}

fn main() {
    // Phase 2: harden process before anything else
    // - Disable core dumps (setrlimit RLIMIT_CORE 0)
    // - Deny debugger attachment (ptrace PT_DENY_ATTACH)
    chambers_crypto::mem_protect::harden_process();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Bootstrap runtime — shared between native app and HTTP adapter
    let mut runtime = Runtime::new();
    runtime
        .load_grammar(decision_chamber_grammar())
        .expect("failed to load grammar");
    let runtime = Arc::new(Mutex::new(runtime));

    // Start HTTP adapter in background thread (for chamber webview fetch calls)
    let rt_http = runtime.clone();
    let px_http = proxy.clone();
    thread::spawn(move || {
        let tokio_rt = tokio::runtime::Runtime::new().unwrap();
        tokio_rt.block_on(async {
            start_http_adapter(rt_http, px_http).await;
        });
    });

    // Give HTTP adapter time to start
    thread::sleep(std::time::Duration::from_millis(500));

    // --- Lobby window ---
    let lobby_window = WindowBuilder::new()
        .with_title("CHAMBERS")
        .with_fullscreen(Some(tao::window::Fullscreen::Borderless(None)))
        .with_decorations(false)
        .build(&event_loop)
        .expect("failed to create lobby window");

    let _lobby_webview = WebViewBuilder::new()
        .with_url("http://127.0.0.1:3000/lobby")
        .with_incognito(true) // Phase 2: no persistent WebKit storage
        .build(&lobby_window)
        .expect("failed to create lobby webview");

    let lobby_window_id = lobby_window.id();

    // Track chamber windows
    let mut chamber_windows: HashMap<String, (tao::window::Window, wry::WebView)> = HashMap::new();
    let mut windowid_to_world: HashMap<WindowId, String> = HashMap::new();

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                if window_id == lobby_window_id {
                    // Block Cmd+Q / window close. The only exit is Esc.
                    // Do nothing — the app stays alive.
                } else {
                    // Chamber window force-closed — abort burn
                    if let Some(wid) = windowid_to_world.remove(&window_id) {
                        abort_burn_if_alive(&runtime, &wid);
                        chamber_windows.remove(&wid);
                        lobby_window.set_visible(true);
                        lobby_window.set_focus();
                    }
                }
            }

            Event::UserEvent(ref app_event) => match app_event {
                AppEvent::OpenChamber { world_id, .. } => {
                    // Hide lobby — you're entering the chamber
                    lobby_window.set_visible(false);

                    // Fullscreen, no decorations — the chamber IS the screen
                    let chamber_win = WindowBuilder::new()
                        .with_title("CHAMBER")
                        .with_fullscreen(Some(tao::window::Fullscreen::Borderless(None)))
                        .with_decorations(false)
                        .build(event_loop)
                        .expect("failed to create chamber window");

                    let win_id = chamber_win.id();
                    let url = format!("http://127.0.0.1:3000/chamber/{}", world_id);

                    let chamber_webview = WebViewBuilder::new()
                        .with_url(&url)
                        .with_incognito(true) // Phase 2: no persistent WebKit storage
                        .build(&chamber_win)
                        .expect("failed to create chamber webview");

                    windowid_to_world.insert(win_id, world_id.clone());
                    chamber_windows.insert(world_id.clone(), (chamber_win, chamber_webview));
                }

                AppEvent::CloseChamber { world_id } => {
                    if let Some((_, _)) = chamber_windows.remove(world_id) {
                        windowid_to_world.retain(|_, v| v != world_id);
                    }
                    lobby_window.set_visible(true);
                    lobby_window.set_focus();
                }

                AppEvent::Quit => {
                    // Burn all active chambers before exiting
                    for (wid, _) in chamber_windows.drain() {
                        abort_burn_if_alive(&runtime, &wid);
                    }
                    windowid_to_world.clear();
                    *control_flow = ControlFlow::Exit;
                }
            },

            _ => {}
        }
    });
}

fn abort_burn_if_alive(runtime: &Arc<Mutex<Runtime>>, world_id_str: &str) {
    if let Ok(uuid) = uuid::Uuid::parse_str(world_id_str) {
        let wid = chambers_types::world::WorldId(uuid);
        let rt = runtime.lock().unwrap();
        if let Ok(w) = rt.world_engine.get_world(wid) {
            if w.lifecycle_phase != chambers_types::world::LifecyclePhase::Terminated {
                let req = chambers_types::primitive::TransitionRequest {
                    world_id: wid,
                    principal: Principal::new("operator"),
                    operation: chambers_types::primitive::TransitionOperation::TriggerBurn {
                        mode: chambers_types::world::TerminationMode::AbortBurn,
                    },
                };
                let _ = rt.submit(&req);
            }
        }
    }
}

fn load_demo_data(runtime: &Arc<Mutex<Runtime>>, world_id_str: &str) {
    let uuid = match uuid::Uuid::parse_str(world_id_str) {
        Ok(u) => u,
        Err(_) => return,
    };
    let wid = chambers_types::world::WorldId(uuid);
    let p = Principal::new("operator");

    use chambers_types::object::LifecycleClass;
    use chambers_types::primitive::{TransitionOperation, TransitionRequest};

    let mut ids: HashMap<String, chambers_types::object::ObjectId> = HashMap::new();

    // Lock/unlock per operation to avoid blocking the HTTP adapter thread
    let add = |ids: &mut HashMap<String, chambers_types::object::ObjectId>, key: &str, otype: &str, payload: serde_json::Value, lc: LifecycleClass| {
        let req = TransitionRequest {
            world_id: wid,
            principal: p.clone(),
            operation: TransitionOperation::CreateObject {
                object_type: otype.to_string(),
                payload,
                lifecycle_class: lc,
                preservable: false,
            },
        };
        let rt = runtime.lock().unwrap();
        if let Ok(chambers_operation::OperationResult::ObjectCreated(oid)) = rt.submit(&req) {
            ids.insert(key.to_string(), oid);
        }
    };

    add(&mut ids, "p1", "premise", serde_json::json!({"statement":"Our current auth is a custom PHP implementation from 2016 with known security issues.","source":"security audit Q1"}), LifecycleClass::Temporary);
    add(&mut ids, "p2", "premise", serde_json::json!({"statement":"Engineering team has 2 developers with identity/auth experience.","source":"team lead"}), LifecycleClass::Temporary);
    add(&mut ids, "p3", "premise", serde_json::json!({"statement":"Three customer contracts require SOC2 compliance by end of year.","source":"sales team"}), LifecycleClass::Temporary);
    add(&mut ids, "c1", "constraint", serde_json::json!({"description":"Must achieve SOC2 Type II compliance within 8 months.","severity":"hard"}), LifecycleClass::Temporary);
    add(&mut ids, "c2", "constraint", serde_json::json!({"description":"Migration must not cause more than 4 hours of downtime.","severity":"hard"}), LifecycleClass::Temporary);
    add(&mut ids, "c3", "constraint", serde_json::json!({"description":"Annual cost must stay under $50,000.","severity":"soft"}), LifecycleClass::Temporary);
    add(&mut ids, "a1", "alternative", serde_json::json!({"description":"Auth0 — managed identity platform","pros":"Fast to implement, SOC2 certified, handles MFA/SSO out of box","cons":"Vendor lock-in, per-user pricing scales badly"}), LifecycleClass::Intermediate);
    add(&mut ids, "a2", "alternative", serde_json::json!({"description":"Build custom with Passport.js + PostgreSQL","pros":"Full control, no vendor dependency, one-time cost","cons":"6+ months to build, security risk during development"}), LifecycleClass::Intermediate);
    add(&mut ids, "a3", "alternative", serde_json::json!({"description":"Keycloak self-hosted","pros":"Open source, full control, no per-user cost","cons":"Complex to operate, requires dedicated DevOps"}), LifecycleClass::Intermediate);
    add(&mut ids, "r1", "risk", serde_json::json!({"description":"Custom build takes 6+ months, missing SOC2 deadline — loss of $400K ARR.","likelihood":"high","impact":"critical"}), LifecycleClass::Temporary);
    add(&mut ids, "r2", "risk", serde_json::json!({"description":"Auth0 per-user pricing exceeds budget at 50K+ users (month 18).","likelihood":"medium","impact":"medium"}), LifecycleClass::Temporary);
    add(&mut ids, "r3", "risk", serde_json::json!({"description":"Keycloak operational complexity causes auth outages in first 6 months.","likelihood":"medium","impact":"high"}), LifecycleClass::Temporary);
    add(&mut ids, "u1", "upside", serde_json::json!({"description":"Auth0 gets us SOC2-ready in under 2 months, unblocking all 3 contracts immediately.","magnitude":"high"}), LifecycleClass::Temporary);
    add(&mut ids, "u2", "upside", serde_json::json!({"description":"Keycloak gives full data sovereignty for EU expansion planned Q3.","magnitude":"medium"}), LifecycleClass::Temporary);

    // Links — also lock/unlock per call
    let link = |ids: &HashMap<String, chambers_types::object::ObjectId>, src: &str, tgt: &str, lt: &str| {
        if let (Some(&s), Some(&t)) = (ids.get(src), ids.get(tgt)) {
            let req = TransitionRequest {
                world_id: wid,
                principal: p.clone(),
                operation: TransitionOperation::LinkObjects {
                    source_id: s, target_id: t, link_type: lt.to_string(),
                },
            };
            let rt = runtime.lock().unwrap();
            let _ = rt.submit(&req);
        }
    };
    link(&ids, "r1", "a2", "risks");
    link(&ids, "r2", "a1", "risks");
    link(&ids, "r3", "a3", "risks");
    link(&ids, "u1", "a1", "benefits");
    link(&ids, "u2", "a3", "benefits");
}

/// Start the HTTP adapter on a background thread.
async fn start_http_adapter(runtime: Arc<Mutex<Runtime>>, proxy: EventLoopProxy<AppEvent>) {
    use axum::routing::{get, post};
    use axum::extract::{Path, State};
    use axum::response::IntoResponse;
    use axum::Json;

    type SharedRuntime = Arc<Mutex<Runtime>>;

    use axum::response::Html;

    // Chamber clipboard — world-scoped, in-memory, zeroed on burn
    let clipboard: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // HTML page routes + app control endpoints
    let lobby_html_str = ui_lobby::lobby_html();
    let px_close = proxy.clone();
    let app = axum::Router::new()
        .route("/lobby", get(move || {
            let h = lobby_html_str.clone();
            async move { Html(h) }
        }))
        .route("/chamber/:id", get(move |Path(id): Path<String>| {
            async move { Html(ui_chamber::chamber_html(&id)) }
        }))
        // App control endpoints — tell the main thread to open/close windows
        .route("/app/open-chamber/:id", post({
            let px = proxy.clone();
            move |Path(id): Path<String>| {
                let px = px.clone();
                async move {
                    let _ = px.send_event(AppEvent::OpenChamber { world_id: id, demo: false });
                    Json(serde_json::json!({"ok": true}))
                }
            }
        }))
        .route("/app/close-chamber/:id", post(move |Path(id): Path<String>| {
            let px = px_close.clone();
            async move {
                let _ = px.send_event(AppEvent::CloseChamber { world_id: id });
                Json(serde_json::json!({"ok": true}))
            }
        }))
        // Chamber clipboard — world-scoped, never touches system pasteboard
        .route("/app/clipboard/copy", post({
            let cb = clipboard.clone();
            move |Json(body): Json<serde_json::Value>| {
                let cb = cb.clone();
                async move {
                    let wid = body["world_id"].as_str().unwrap_or("");
                    let text = body["text"].as_str().unwrap_or("");
                    cb.lock().unwrap().insert(wid.to_string(), text.to_string());
                    Json(serde_json::json!({"ok": true}))
                }
            }
        }))
        .route("/app/clipboard/paste", post({
            let cb = clipboard.clone();
            move |Json(body): Json<serde_json::Value>| {
                let cb = cb.clone();
                async move {
                    let wid = body["world_id"].as_str().unwrap_or("");
                    let text = cb.lock().unwrap().get(wid).cloned().unwrap_or_default();
                    Json(serde_json::json!({"text": text}))
                }
            }
        }))
        .route("/app/clipboard/burn", post({
            let cb = clipboard.clone();
            move |Json(body): Json<serde_json::Value>| {
                let cb = cb.clone();
                async move {
                    let wid = body["world_id"].as_str().unwrap_or("");
                    if let Some(mut val) = cb.lock().unwrap().remove(wid) {
                        // Zeroize the clipboard content before dropping
                        use zeroize::Zeroize;
                        val.zeroize();
                    }
                    Json(serde_json::json!({"ok": true}))
                }
            }
        }))
        .route("/app/quit", post({
            let px = proxy.clone();
            move || {
                let px = px.clone();
                async move {
                    let _ = px.send_event(AppEvent::Quit);
                    Json(serde_json::json!({"ok": true}))
                }
            }
        }))
        // API endpoints
        .route("/api/worlds", post({
            let rt = runtime.clone();
            move |Json(body): Json<serde_json::Value>| {
                let rt = rt.clone();
                async move {
                    let grammar = body["grammar_id"].as_str().unwrap_or("decision_chamber_v1");
                    let objective = body["objective"].as_str().unwrap_or("");
                    let r = rt.lock().unwrap();
                    match r.create_world(grammar, objective) {
                        Ok(wid) => {
                            let _ = r.issue_capabilities(wid, Principal::new("operator"), &[
                                Primitive::CreateObject, Primitive::LinkObjects,
                                Primitive::ChallengeObject, Primitive::GenerateAlternative,
                                Primitive::RankSet, Primitive::SynthesizeSet,
                                Primitive::CondenseObject, Primitive::TriggerBurn,
                            ]);
                            Json(serde_json::json!({"world_id": wid.0.to_string()})).into_response()
                        }
                        Err(e) => Json(serde_json::json!({"error": e.to_string()})).into_response(),
                    }
                }
            }
        }))
        .route("/api/worlds/:id", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move { world_handler(&rt, &id, |rt, wid| serde_json::to_value(&rt.world_engine.get_world(wid).ok()).unwrap()) }
            }
        }))
        .route("/api/worlds/:id/summary", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move { world_handler(&rt, &id, |rt, wid| serde_json::to_value(&rt.view_engine.summary_view(wid).ok()).unwrap()) }
            }
        }))
        .route("/api/worlds/:id/graph", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move { world_handler(&rt, &id, |rt, wid| serde_json::to_value(&rt.view_engine.graph_view(wid).ok()).unwrap()) }
            }
        }))
        .route("/api/worlds/:id/convergence", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move { world_handler(&rt, &id, |rt, wid| serde_json::to_value(&rt.get_convergence_state(wid).ok()).unwrap()) }
            }
        }))
        .route("/api/worlds/:id/legal-actions", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move { world_handler(&rt, &id, |rt, wid| serde_json::to_value(&rt.get_legal_actions(wid).ok()).unwrap()) }
            }
        }))
        .route("/api/worlds/:id/submit", post({
            let rt = runtime.clone();
            move |Path(id): Path<String>, Json(body): Json<serde_json::Value>| {
                let rt = rt.clone();
                async move {
                    let uuid = match uuid::Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return Json(serde_json::json!({"error":"bad id"})).into_response() };
                    let wid = chambers_types::world::WorldId(uuid);
                    let op = match serde_json::from_value::<chambers_types::primitive::TransitionOperation>(body) {
                        Ok(o) => o, Err(e) => return Json(serde_json::json!({"error": e.to_string()})).into_response()
                    };
                    let req = chambers_types::primitive::TransitionRequest { world_id: wid, principal: Principal::new("operator"), operation: op };
                    let r = rt.lock().unwrap();
                    match r.submit(&req) {
                        Ok(result) => Json(serde_json::to_value(&result).unwrap()).into_response(),
                        Err(e) => Json(serde_json::json!({"error": e.to_string()})).into_response(),
                    }
                }
            }
        }))
        .route("/api/worlds/:id/advance", post({
            let rt = runtime.clone();
            move |Path(id): Path<String>, Json(body): Json<serde_json::Value>| {
                let rt = rt.clone();
                async move {
                    let uuid = match uuid::Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return Json(serde_json::json!({"error":"bad id"})).into_response() };
                    let wid = chambers_types::world::WorldId(uuid);
                    let phase = match body["phase"].as_str().unwrap_or("") {
                        "Active" => chambers_types::world::LifecyclePhase::Active,
                        "ConvergenceReview" => chambers_types::world::LifecyclePhase::ConvergenceReview,
                        "Finalization" => chambers_types::world::LifecyclePhase::Finalization,
                        _ => return Json(serde_json::json!({"error":"bad phase"})).into_response(),
                    };
                    let r = rt.lock().unwrap();
                    if let Err(e) = r.advance_phase(wid, phase) { return Json(serde_json::json!({"error": e.to_string()})).into_response(); }
                    let prims: Vec<Primitive> = match phase {
                        chambers_types::world::LifecyclePhase::Active => vec![Primitive::CreateObject,Primitive::LinkObjects,Primitive::ChallengeObject,Primitive::GenerateAlternative,Primitive::RankSet,Primitive::SynthesizeSet,Primitive::CondenseObject,Primitive::TriggerBurn],
                        chambers_types::world::LifecyclePhase::ConvergenceReview => vec![Primitive::ChallengeObject,Primitive::CondenseObject,Primitive::LinkObjects,Primitive::TriggerBurn],
                        chambers_types::world::LifecyclePhase::Finalization => vec![Primitive::SealArtifact,Primitive::CondenseObject,Primitive::TriggerBurn],
                        _ => vec![],
                    };
                    let _ = r.issue_capabilities(wid, Principal::new("operator"), &prims);
                    Json(serde_json::json!({"ok":true})).into_response()
                }
            }
        }))
        .route("/api/worlds/:id/burn", post({
            let rt = runtime.clone();
            move |Path(id): Path<String>, Json(body): Json<serde_json::Value>| {
                let rt = rt.clone();
                async move {
                    let uuid = match uuid::Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return Json(serde_json::json!({"error":"bad id"})).into_response() };
                    let wid = chambers_types::world::WorldId(uuid);
                    let mode = match body["mode"].as_str().unwrap_or("AbortBurn") {
                        "ConvergedPreserving" => chambers_types::world::TerminationMode::ConvergedPreserving,
                        "ConvergedTotalBurn" => chambers_types::world::TerminationMode::ConvergedTotalBurn,
                        _ => chambers_types::world::TerminationMode::AbortBurn,
                    };
                    let req = chambers_types::primitive::TransitionRequest {
                        world_id: wid, principal: Principal::new("operator"),
                        operation: chambers_types::primitive::TransitionOperation::TriggerBurn { mode },
                    };
                    let r = rt.lock().unwrap();
                    match r.submit(&req) {
                        Ok(_) => Json(serde_json::json!({"ok":true})).into_response(),
                        Err(e) => Json(serde_json::json!({"error": e.to_string()})).into_response(),
                    }
                }
            }
        }))
        .route("/api/worlds/:id/residue", get({
            let rt = runtime.clone();
            move |Path(id): Path<String>| {
                let rt = rt.clone();
                async move {
                    let uuid = match uuid::Uuid::parse_str(&id) { Ok(u) => u, Err(_) => return Json(serde_json::json!({"error":"bad id"})).into_response() };
                    let wid = chambers_types::world::WorldId(uuid);
                    let r = rt.lock().unwrap();
                    let residue = r.burn_engine.measure_residue(wid);
                    Json(serde_json::to_value(&residue).unwrap()).into_response()
                }
            }
        }))
        .route("/api/vault", get({
            let rt = runtime.clone();
            move || {
                let rt = rt.clone();
                async move {
                    let r = rt.lock().unwrap();
                    Json(serde_json::to_value(&r.vault.all_artifacts()).unwrap()).into_response()
                }
            }
        }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn world_handler(
    rt: &Arc<Mutex<Runtime>>,
    id: &str,
    f: impl FnOnce(&Runtime, chambers_types::world::WorldId) -> serde_json::Value,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    use axum::Json;
    let uuid = match uuid::Uuid::parse_str(id) {
        Ok(u) => u,
        Err(_) => return Json(serde_json::json!({"error":"bad id"})).into_response(),
    };
    let wid = chambers_types::world::WorldId(uuid);
    let r = rt.lock().unwrap();
    Json(f(&r, wid)).into_response()
}
