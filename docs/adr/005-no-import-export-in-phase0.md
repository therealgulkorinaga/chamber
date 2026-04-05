# ADR-005: No Import/Export in Phase 0

## Status
Accepted

## Context
General import/export would create uncontrolled data flow channels that undermine the world-first isolation model. Every import path is a potential smuggling vector; every export path is a potential residue leak.

## Decision
Phase 0 has **no** ordinary import/export mechanism. The only data that crosses a world boundary is:

- A sealed artifact, through the `seal_artifact` primitive, into the artifact vault.
- Sparse audit events (lifecycle signals only, no world internals).

No file import, no clipboard, no external blob references, no API calls from within a world.

## Consequences
- The artifact vault is the sole authorized cross-world channel.
- World state is completely self-contained.
- This limits Phase 0's utility as a product — intentionally.
- Import/export is deferred to later phases with explicit policy controls.
