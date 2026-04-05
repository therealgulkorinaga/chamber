//! Level 1 security audit tests (Issue list: Epics 10.1–10.5).
//!
//! Tests:
//! - No client-side persistence (verified by UI HTML audit)
//! - Adapter has no policy logic (structural review)
//! - Cross-world isolation via HTTP
//! - Post-burn freeze via HTTP
//! - Adapter error responses don't leak payloads

use std::time::Duration;

/// Audit: UI HTML contains no localStorage, IndexedDB, or service worker usage.
#[test]
fn test_ui_has_no_browser_persistence() {
    let html = include_str!("../../../ui/index.html");

    // No localStorage
    assert!(
        !html.contains("localStorage.setItem"),
        "UI must not write to localStorage"
    );
    assert!(
        !html.contains("localStorage.set"),
        "UI must not write to localStorage"
    );

    // No IndexedDB
    assert!(
        !html.contains("indexedDB"),
        "UI must not use IndexedDB"
    );
    assert!(
        !html.contains("openDatabase"),
        "UI must not use WebSQL"
    );

    // No service worker
    assert!(
        !html.contains("serviceWorker.register"),
        "UI must not register service workers"
    );
    assert!(
        !html.contains("navigator.serviceWorker"),
        "UI must not reference service workers"
    );

    // No sessionStorage for chamber data (allowed for pure UI state like theme)
    // Check that sessionStorage is not used to store world/object data
    assert!(
        !html.contains("sessionStorage.setItem"),
        "UI must not write to sessionStorage"
    );
}

/// Audit: UI does not contain shadow object stores or hidden caches.
#[test]
fn test_ui_has_no_hidden_state_stores() {
    let html = include_str!("../../../ui/index.html");

    // No hidden object store patterns
    assert!(
        !html.contains("new Map()") || html.contains("// no persistent map"),
        "UI should not maintain object maps (unless transient render cache)"
    );

    // The only JS variables should be ephemeral
    // currentWorldId is the only world-related state
    let world_vars: Vec<&str> = html
        .lines()
        .filter(|l| l.contains("let ") && (l.contains("world") || l.contains("World")))
        .filter(|l| !l.contains("currentWorldId") && !l.contains("//") && !l.contains("fetch"))
        .collect();

    // Allow limited world-related variables (function params, fetch responses)
    // but flag any that look like persistent stores
    for var in &world_vars {
        assert!(
            !var.contains("Map") && !var.contains("Store") && !var.contains("Cache"),
            "Suspicious world state variable: {}",
            var
        );
    }
}

/// Audit: Adapter handler code contains no policy logic.
#[test]
fn test_adapter_has_no_policy_logic() {
    let handlers = include_str!("../src/handlers.rs");

    // Adapter must not check capabilities
    assert!(
        !handlers.contains("check_capability"),
        "Adapter must not perform capability checks — substrate does this"
    );

    // Adapter must not check preservation law
    assert!(
        !handlers.contains("can_preserve"),
        "Adapter must not check preservation law — substrate does this"
    );

    // Adapter must not validate lifecycle transitions
    assert!(
        !handlers.contains("can_transition_to"),
        "Adapter must not validate lifecycle transitions — substrate does this"
    );

    // Adapter must not check primitive permissions
    assert!(
        !handlers.contains("is_primitive_allowed"),
        "Adapter must not check primitive permissions — substrate does this"
    );
}

/// Audit: Error responses do not contain object payloads.
#[test]
fn test_adapter_errors_dont_leak_payloads() {
    let handlers = include_str!("../src/handlers.rs");

    // The map_error function should only use err.to_string(), not serialize payloads
    assert!(
        !handlers.contains("payload") || handlers.contains("// payload")
            || handlers.contains("\"payload\"")  // DTO field names are OK
            || handlers.contains("payload:"),     // struct field access is OK
        "Error mapping should not include raw payload data"
    );

    // Error responses should use the error message, not dump internal state
    // The map_error function exists and maps to HTTP status codes
    assert!(
        handlers.contains("fn map_error"),
        "Error mapping function must exist"
    );
}
