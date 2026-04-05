# Level 1 Architecture Note

## System Boundary

```
┌──────────────────────────────────────────────────────┐
│ Browser (Layer C — UI Shell)                          │
│ ┌──────────────────────────────────────────────────┐  │
│ │ index.html + vanilla JS                          │  │
│ │ - Renders substrate state                        │  │
│ │ - Invokes actions via fetch() to /api/*          │  │
│ │ - NO localStorage, NO IndexedDB, NO SW           │  │
│ │ - Ephemeral state only (currentWorldId, panel)   │  │
│ └──────────────────────────────────────────────────┘  │
│                    ↕ HTTP (JSON)                       │
├──────────────────────────────────────────────────────┤
│ Adapter (Layer B — Stateless Forwarder)               │
│ ┌──────────────────────────────────────────────────┐  │
│ │ axum HTTP server (chambers-adapter crate)         │  │
│ │ - Parses HTTP → substrate types                   │  │
│ │ - Calls Runtime methods                          │  │
│ │ - Returns JSON                                    │  │
│ │ - NO policy logic                                │  │
│ │ - NO state caching                               │  │
│ │ - Auto-issues capabilities on create/advance     │  │
│ └──────────────────────────────────────────────────┘  │
│                    ↕ Rust function calls               │
├──────────────────────────────────────────────────────┤
│ Substrate (Layer A — Level 0)                         │
│ ┌──────────────────────────────────────────────────┐  │
│ │ Runtime (chambers-runtime crate)                  │  │
│ │ - World engine, object engine, policy engine      │  │
│ │ - Interpreter (5-check validation pipeline)       │  │
│ │ - Capability system, state engine                 │  │
│ │ - Burn engine (5-layer), artifact vault           │  │
│ │ - Audit log, view engine                          │  │
│ │ - ALL policy enforcement happens here             │  │
│ └──────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

## Data Flow

1. User clicks "Create Premise" in UI
2. JS builds `{ operation: "create_object", object_type: "premise", ... }`
3. `fetch("/api/worlds/{id}/submit", { method: "POST", body })` 
4. Adapter parses request, calls `runtime.submit(TransitionRequest)`
5. Interpreter validates: world scope → type → capability → lifecycle → preservation
6. Operation engine creates object in state engine
7. Adapter returns `{ "ObjectCreated": "uuid" }`
8. JS refreshes world state panel

## Allowed UI-Local State

| State | Type | Persisted? |
|-------|------|-----------|
| `currentWorldId` | JS variable | Memory only |
| Panel open/closed | CSS class | Memory only |
| Last fetch response | JS variable | Memory only |
| Loading spinner | CSS class | Memory only |

## Forbidden UI-Local State

- Object graph cache
- Shadow lifecycle tracker  
- Draft buffer of chamber internals
- Cached artifact copies
- Event logs of world internals
- localStorage entries
- IndexedDB databases
- Service worker registrations

## API Endpoints

| Endpoint | Method | Returns |
|----------|--------|---------|
| `/api/worlds` | POST | `{ world_id }` |
| `/api/worlds/:id` | GET | World metadata |
| `/api/worlds/:id/objects` | GET | Conversation view |
| `/api/worlds/:id/graph` | GET | Graph view |
| `/api/worlds/:id/summary` | GET | Summary view |
| `/api/worlds/:id/legal-actions` | GET | Legal primitives |
| `/api/worlds/:id/convergence` | GET | Convergence state |
| `/api/worlds/:id/submit` | POST | Operation result |
| `/api/worlds/:id/advance` | POST | Phase advance result |
| `/api/worlds/:id/burn` | POST | Burn result |
| `/api/worlds/:id/residue` | GET | Residue report |
| `/api/worlds/:id/burn-view` | GET | Burn view |
| `/api/worlds/:id/audit` | GET | Audit events |
| `/api/vault` | GET | Vault contents |
| `/api/grammars` | GET | Available grammars |
