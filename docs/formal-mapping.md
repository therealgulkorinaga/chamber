# Formal Tuple-to-Code Mapping

## Chamber Tuple

The paper defines a Chamber as:

```
C = (O, T, P, K, V, L, R)
```

This document maps each tuple component to its concrete Rust implementation.

---

## O — Objective

**Paper**: The chamber's purpose, scoped to a grammar.

**Code**: `World.objective: String` + `ChamberGrammar.objective_class: String`

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Objective value | `World.objective` | `chambers-types` | `src/world.rs` |
| Objective class | `ChamberGrammar.objective_class` | `chambers-types` | `src/grammar.rs` |

---

## T — Typed Objects

**Paper**: All world content is typed, schema-validated, with constrained payloads.

**Code**: `Object` struct with `object_type`, `payload`, `lifecycle_class`, `preservable` flag.

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Typed object | `Object` | `chambers-types` | `src/object.rs` |
| Object ID | `ObjectId(Uuid)` | `chambers-types` | `src/object.rs` |
| Object type | `Object.object_type: String` | `chambers-types` | `src/object.rs` |
| Payload | `Object.payload: serde_json::Value` | `chambers-types` | `src/object.rs` |
| Lifecycle class | `LifecycleClass` enum: `Temporary`, `Intermediate`, `Candidate`, `Preservable` | `chambers-types` | `src/object.rs` |
| Schema | `ObjectTypeSpec.payload_schema` | `chambers-types` | `src/grammar.rs` |
| Transform set | `Object.transform_set: Vec<Primitive>` | `chambers-types` | `src/object.rs` |
| Object link | `ObjectLink { source_id, target_id, link_type, world_id }` | `chambers-types` | `src/object.rs` |

**Validation rules**:
- No opaque binary payloads (enforced by `ObjectEngine::validate_no_binary`)
- No external blob references (`$blob`, `$ref`, `$binary` keys rejected)
- No Base64-like smuggling (heuristic detection for strings >1000 chars)
- Payload size bounded per type (`ObjectTypeSpec.max_payload_bytes`)

---

## P — Primitives

**Paper**: A finite, closed set of operations. No dynamic primitive creation.

**Code**: `Primitive` enum (9 variants, no `Other`).

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Primitive algebra | `Primitive` enum | `chambers-types` | `src/primitive.rs` |
| Transition request | `TransitionRequest { world_id, principal, operation }` | `chambers-types` | `src/primitive.rs` |
| Operation params | `TransitionOperation` enum (9 variants) | `chambers-types` | `src/primitive.rs` |

**Primitive → Variant mapping**:

| Primitive | `TransitionOperation` variant |
|-----------|------------------------------|
| `create_object` | `CreateObject { object_type, payload, lifecycle_class, preservable }` |
| `link_objects` | `LinkObjects { source_id, target_id, link_type }` |
| `challenge_object` | `ChallengeObject { target_id, challenge_text }` |
| `generate_alternative` | `GenerateAlternative { target_id, alternative_payload }` |
| `rank_set` | `RankSet { object_ids, rankings }` |
| `synthesize_set` | `SynthesizeSet { source_ids, synthesis_type, synthesis_payload }` |
| `condense_object` | `CondenseObject { target_id, condensed_payload }` |
| `seal_artifact` | `SealArtifact { target_id, authorization }` |
| `trigger_burn` | `TriggerBurn { mode }` |

---

## K — Capabilities

**Paper**: World-scoped, epoch-scoped tokens. Invalidated on epoch advance.

**Code**: `CapabilityToken` struct + `CapabilitySystem` engine.

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Capability token | `CapabilityToken` | `chambers-types` | `src/capability.rs` |
| Token ID | `TokenId(Uuid)` | `chambers-types` | `src/capability.rs` |
| Principal | `Principal(String)` | `chambers-types` | `src/capability.rs` |
| Token issuance | `CapabilitySystem::issue_token()` | `chambers-capability` | `src/lib.rs` |
| Capability check | `CapabilitySystem::check_capability()` | `chambers-capability` | `src/lib.rs` |
| Epoch invalidation | `CapabilitySystem::invalidate_epoch()` | `chambers-capability` | `src/lib.rs` |

**Validation rules**:
- Token must match world_id
- Token must match current epoch
- Token must not be revoked or expired
- Token must permit the requested operation
- Token must permit the target object type (or be unrestricted)

---

## V — Views

**Paper**: Read-only projections. Not independent persistence channels.

**Code**: `ViewEngine` with four view types.

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Conversation view | `ConversationView` | `chambers-view` | `src/lib.rs` |
| Graph view | `GraphView` | `chambers-view` | `src/lib.rs` |
| Summary view | `SummaryView` | `chambers-view` | `src/lib.rs` |
| Burn view | `BurnView` | `chambers-view` | `src/lib.rs` |

**Invariant**: Views are derived from world state. They do not mutate state. They do not persist independently.

---

## L — Lifecycle Law

**Paper**: Worlds progress through phases. Capabilities narrow. Termination is formal.

**Code**: `LifecyclePhase` enum + `WorldEngine` state machine.

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Lifecycle phase | `LifecyclePhase` enum: `Created`, `Active`, `ConvergenceReview`, `Finalization`, `Terminated` | `chambers-types` | `src/world.rs` |
| Phase transition | `LifecyclePhase::can_transition_to()` | `chambers-types` | `src/world.rs` |
| Epoch | `World.epoch: u32` | `chambers-types` | `src/world.rs` |
| Termination mode | `TerminationMode` enum: `ConvergedPreserving`, `ConvergedTotalBurn`, `AbortBurn` | `chambers-types` | `src/world.rs` |

**State machine**:
```
Created → Active → ConvergenceReview → Finalization → Terminated
                 ↙ (rejection/rework)
   ConvergenceReview → Active
   
Abort: any non-terminated → Terminated
```

---

## R — Preservation Law

**Paper**: Only approved artifact classes may survive. Everything else burns.

**Code**: `ChamberGrammar.preservable_classes` + `PolicyEngine::can_preserve_object()` + `SealAuthorization`.

| Paper concept | Rust type | Crate | File |
|--------------|-----------|-------|------|
| Preservable classes | `ChamberGrammar.preservable_classes: Vec<String>` | `chambers-types` | `src/grammar.rs` |
| Preservation check | `PolicyEngine::can_preserve_object()` | `chambers-policy` | `src/lib.rs` |
| Seal authorization | `SealAuthorization` enum: `HumanConfirmed`, `PolicyApproved` | `chambers-types` | `src/primitive.rs` |
| Sealed artifact | `Artifact` | `chambers-types` | `src/artifact.rs` |
| Artifact vault | `ArtifactVault` | `chambers-vault` | `src/lib.rs` |

**Invariants**:
- Only objects with `preservable: true` and type in `preservable_classes` can be sealed.
- Sealing requires explicit authorization (model cannot unilaterally seal).
- Sealing is only permitted in `Finalization` phase.
- The vault is the sole cross-world channel.
- Vault stores minimal provenance metadata (no world internals).

---

## Cross-Cutting: Interpreter Validation Pipeline

The interpreter enforces all tuple components through a 5-check pipeline:

```
TransitionRequest
    │
    ├─ 1. World-scope correctness    (world exists, not terminated)
    ├─ 2. Type compatibility          (object type in grammar)    [T]
    ├─ 3. Capability possession       (token valid for operation) [K]
    ├─ 4. Lifecycle legality          (primitive allowed in phase) [L]
    └─ 5. Preservation-law legality   (seal only preservable)     [R]
    │
    ▼
  OperationEngine::execute()  →  State mutation  [P]
```

**Location**: `Interpreter::submit()` in `chambers-interpreter/src/lib.rs`
