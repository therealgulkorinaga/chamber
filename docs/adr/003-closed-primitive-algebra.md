# ADR-003: Closed Primitive Algebra

## Status
Accepted

## Context
If world evolution can occur through arbitrary operations, the substrate cannot enforce lifecycle law, preservation law, or burn completeness. The system must control exactly what mutations are possible.

## Decision
All world state evolution occurs through a **finite, closed set of primitives**:

1. `create_object` — create a typed object
2. `link_objects` — create a directed edge between objects
3. `challenge_object` — mark an object as challenged
4. `generate_alternative` — create an alternative linked to an existing object
5. `rank_set` — assign numeric rankings to a set of objects
6. `synthesize_set` — combine multiple objects into a synthesis
7. `condense_object` — replace payload with condensed form
8. `seal_artifact` — move a preservable object into the vault
9. `trigger_burn` — initiate world destruction

No dynamic primitive creation is permitted. The `Primitive` enum in Rust has no `Other(String)` variant.

## Consequences
- The interpreter can exhaustively validate every possible operation.
- Grammar definitions can precisely specify which primitives are legal in which epoch.
- No backdoor mutations are possible through the public API.
- Adding a new primitive requires a code change and grammar update.
