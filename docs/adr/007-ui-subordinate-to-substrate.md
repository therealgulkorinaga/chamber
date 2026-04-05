# ADR-007: UI Is Subordinate to Substrate Law

## Status
Accepted

## Context
Level 1 adds a visual shell (HTML UI + HTTP adapter) on top of the Level 0 substrate. The risk is that the UI layer gradually reintroduces application-centric persistence, hidden state, or policy bypass.

## Decision

### Layer B — API Adapter
The adapter is a **stateless forwarder**. It:
- Translates HTTP requests into substrate API calls
- Returns substrate responses as JSON
- Does **not** perform any policy or capability checks
- Does **not** cache world state beyond a single request
- Does **not** offer any mutation path that bypasses the interpreter

### Layer C — UI Shell
The UI:
- Renders only what the substrate exposes via the adapter
- Invokes only legal substrate actions via the adapter
- **Never** mutates world state directly
- Uses **no** client-side persistence (localStorage, IndexedDB, service workers) for chamber internals
- Tracks only ephemeral UI state in memory (current selection, panel state)

### Forbidden Patterns
- Adapter performing policy checks
- UI caching world state beyond a single render cycle
- Client-side object graph modification
- Shadow object store or local chamber history
- Browser persistence of chamber internals
- Hidden draft buffers
- Extra lifecycle tracker

## Consequences
- Every UI action routes through the same interpreter path as the CLI
- The adapter can be audited for policy logic (must contain none)
- The UI can be audited for persistence (must find none beyond ephemeral state)
- The HTML file has no localStorage/IndexedDB calls
- All meaningful state lives in the substrate
