# Chambers Phase 0 — Synthesized Issue List

## Research Substrate, Benchmark Harness, and Implementation Backlog

This document synthesizes two Phase 0 planning artifacts into a single executable backlog with research governance. It preserves Document B's atomic issue structure (numbered, dependency-tracked, estimated, labeled) while integrating Document A's program-level success criteria, decision gates, orchestration depth, vault granularity, and research integrity framework.

**Total issues: 103**  
**Milestones: 5**  
**Estimated duration: 12–16 weeks (single engineer) / 6–8 weeks (2–3 engineers)**

---

## Legend

- **Priority**: P0 (blocking), P1 (high), P2 (normal), P3 (low)
- **Estimate**: Small (≤4h), Medium (1–2d), Large (3–5d)
- **Labels**: component, milestone, type (feat, fix, test, doc)

---

# Program Charter

## Operating Principle

**Do not add convenience faster than you add proof.**

## Program-Level Success Criteria

Phase 0 is successful only if ALL of the following are true:

1. A real substrate runtime exists, not just a paper architecture.
2. The runtime enforces a closed primitive algebra and rejects undeclared behavior.
3. One reference chamber grammar runs end-to-end.
4. Burn semantics are implemented in a way that matches the paper's five-layer hierarchy.
5. The benchmark harness can test H1–H3 against disposable VM and constrained microVM baselines.
6. Results are good enough to tell whether Chambers is a real runtime category or merely a conceptual reframing.

## Decision Gate

Proceed to Phase 1 only if:

- H1–H3 are testable and preliminary runs suggest a measurable semantic-residue delta.
- The substrate does not recreate the very persistence culture Chambers is trying to escape.
- The architecture remains world-first rather than drifting into "app with better marketing."

If those are not true, Phase 0 has done its job by disproving or narrowing the idea early.

## Definition of Done

Phase 0 is done only when a neutral technical reviewer can confirm:

- The runtime does not behave like ordinary app software with a prettier story.
- The world model is genuinely enforced by substrate law.
- The Decision Chamber is not just a scripted UI but a grammar-bound world.
- Burn is not just delete-by-another-name.
- The artifact vault is the only cross-world channel.
- The benchmark is fair to VM and microVM baselines.
- The measured results are strong enough to justify a Phase 1 build.

## Exit Criteria (Every Epic)

An epic is done only when:

- Code exists.
- Tests exist.
- Documentation exists.
- Benchmark impact is understood.
- No hidden persistence path is knowingly accepted without being documented.

---

# Milestone 1 — Substrate Skeleton (Issues 1–26)

## Architecture Foundation (1–4)

### Issue 1: Repository structure

**Description**  
Create monorepo: `/runtime`, `/grammars`, `/benchmarks`, `/docs`, `/tests`, `/scripts`, `/artifacts`, `/fixtures`. Include README with architecture overview, contribution rules, and coding standards.  
**Acceptance Criteria**  
- Repo organized around substrate-first architecture.
- Benchmark assets isolated from runtime code.
- No ambiguous "misc" folders.  
**Dependencies** None  
**Labels** `infra`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

### Issue 2: Architecture decision records

**Description**  
Write ADRs preventing drift into app-centric or generic workflow software. Required ADRs: world-first architecture, trusted-substrate assumption, closed primitive algebra, cryptographic erasure as primary burn primitive, no import/export in Phase 0, one reference chamber only.  
**Acceptance Criteria**  
- All major design decisions written down.
- Future changes evaluated against ADRs.  
**Dependencies** Issue 1  
**Labels** `doc`, `P0`, `docs`, `Milestone1`  
**Estimate** Medium

### Issue 3: Security assumptions document

**Description**  
State clearly what Phase 0 does not solve: trusted substrate assumption, lower-platform out-of-scope issues, no general import/export, no hardware trust guarantees.  
**Acceptance Criteria**  
- Stored in `/docs`.
- Reviewable before implementation begins.  
**Dependencies** Issue 2  
**Labels** `doc`, `P0`, `docs`, `Milestone1`  
**Estimate** Small

### Issue 4: Formal tuple-to-code mapping document

**Description**  
Map Chamber tuple `C = (O, T, P, K, V, L, R)` into concrete runtime structures. Document `Objective` schema, `TypedObject` schema, `Primitive` interface, `CapabilityToken` structure, `ViewSpec`, `LifecycleLaw`, `PreservationLaw`. Include validation rules.  
**Acceptance Criteria**  
- Every tuple component has a runtime representation.
- No hidden state exists outside declared structures.
- Paper language connected to code.  
**Dependencies** Issue 2  
**Labels** `doc`, `P0`, `docs`, `Milestone1`  
**Estimate** Medium

**Risk** Formal model drifts from implementation over time. Mitigate by treating this document as living spec reviewed at each milestone.

---

## World Engine (5–10)

### Issue 5: World ID generator with no-reuse invariant

**Description**  
Create a thread-safe generator producing unique, monotonic world IDs. After burn, the same ID is never reused. Store retired IDs (simple JSON file persisted across restarts).  
**Acceptance Criteria**  
- `WorldId::generate()` returns new ID each call.
- After `WorldId::retire(id)`, future `generate()` never returns that ID.
- Retired set survives substrate restart.  
**Dependencies** None  
**Labels** `world-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

### Issue 6: World creation API

**Description**  
Implement `WorldEngine::create_world(grammar_id, objective, policy_context)` returning a `World` handle. Allocate world ID, create namespace, initialize lifecycle phase to `Created`, generate world-scoped key material.  
**Acceptance Criteria**  
- Returns `World` with unique ID.
- No object from any previous world is addressable.
- Logs `world_created` to audit layer.  
**Dependencies** Issue 5  
**Labels** `world-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 7: Lifecycle phase tracking and state machine

**Description**  
Implement lifecycle phase enum: `Created → Active → ConvergenceReview → Finalization → PreserveAndBurn | TotalBurn | AbortBurn → Destroyed`. Implement transition validator with tests for invalid transitions.  
**Acceptance Criteria**  
- `World::transition_phase(new_phase)` succeeds only for allowed edges.
- Invalid transition returns error.
- Abort path works from any active state.
- Destroyed worlds cannot transition further.
- Phase changes logged to audit.  
**Dependencies** Issue 6  
**Labels** `world-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 8: World termination dispatch

**Description**  
Add `WorldEngine::terminate_world(world_id, termination_mode)` coordinating with burn engine and lifecycle controller. Termination modes: `ConvergedPreserving`, `ConvergedTotalBurn`, `AbortBurn`.  
**Acceptance Criteria**  
- Calls burn engine with correct mode.
- Marks world as `Destroyed`.
- Logs termination mode.  
**Dependencies** Issue 7, Issue 55 (burn engine skeleton)  
**Labels** `world-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 9: World namespace isolation

**Description**  
Ensure object IDs, capability tokens, and state are strictly scoped to the world ID. Add runtime check in all operations that accept a `world_id`.  
**Acceptance Criteria**  
- Attempt to access object from different world fails with `CrossWorldAccessError`.
- Unit test confirms isolation.  
**Dependencies** Issue 6, Issue 14  
**Labels** `world-engine`, `P1`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 10: World engine invariant – no reuse after burn

**Description**  
After `terminate_world`, world ID is added to retired set. Any subsequent `create_world` with the same ID is prevented.  
**Acceptance Criteria**  
- Integration test creates world, burns it, creates new world — IDs differ.  
**Dependencies** Issue 5, Issue 8  
**Labels** `world-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

---

## Object Engine (11–17)

### Issue 11: Object schema definition and validation

**Description**  
Define `ObjectSchema` struct: `object_type`, `payload_schema` (JSON Schema or similar), `size_bound`, `admissible_encoding`. Implement validator.  
**Acceptance Criteria**  
- Rejects opaque binary payloads.
- Rejects Base64 strings exceeding configurable size.
- Rejects external blob references.  
**Dependencies** None  
**Labels** `object-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 12: Payload schema enforcement

**Description**  
Prevent the object system from becoming a thin wrapper around arbitrary content. Allowed Phase 0 payload classes: bounded scalar text, bounded structured text, symbolic labels, bounded relational references. Explicitly forbidden: opaque binary, external blobs, oversized text, Base64-like smuggling.  
**Acceptance Criteria**  
- Invalid payloads rejected deterministically.
- Schema constraints enforced before object storage.
- Tests include deliberate smuggling attempts.  
**Dependencies** Issue 11  
**Labels** `object-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

**Risk** False positives on legitimate encoded text. Too much permissiveness weakens the ontology; too much strictness makes the chamber unusable. Calibrate through testing.

### Issue 13: Object creation API

**Description**  
Implement `ObjectEngine::create_object(world_id, object_type, payload, lifecycle_class, preservable_flag, capability_reqs)` returning `ObjectId`. Validate payload against schema.  
**Acceptance Criteria**  
- Fails if payload invalid.
- Assigns unique object ID within world.
- No object can exist without world_id.
- No object can be created with unknown type.  
**Dependencies** Issue 11  
**Labels** `object-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 14: Object fields – transform_set and capability_requirements

**Description**  
Add `transform_set: Vec<Transform>` and `capability_requirements: Vec<CapabilityRequirement>` to `Object`. Operation engine checks transform set before applying primitive.  
**Acceptance Criteria**  
- Object creation accepts initial transform set.
- Serialization/deserialization preserves fields.  
**Dependencies** Issue 13  
**Labels** `object-engine`, `P1`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 15: Object retrieval and mutation

**Description**  
Implement `get_object(world_id, object_id)` and `update_object(world_id, object_id, new_payload, new_transform_set)` with validation.  
**Acceptance Criteria**  
- Cannot retrieve object from different world.
- Update fails if object is already burned or world terminated.  
**Dependencies** Issue 13  
**Labels** `object-engine`, `P1`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 16: Object store persistence (in-memory)

**Description**  
Implement `ObjectStore` trait with in-memory implementation. Support world-scoped queries. Restart clears worlds (acceptable for research substrate).  
**Acceptance Criteria**  
- `list_objects(world_id)` returns all objects.
- `delete_object(world_id, object_id)` removes from store.
- Burn engine can iterate over world objects.  
**Dependencies** Issue 13  
**Labels** `object-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 17: Object lifecycle_class semantics

**Description**  
Define `LifecycleClass` enum: `Temporary`, `Intermediate`, `Candidate`, `Preservable`. Enforce that only `Preservable` objects can be part of an artifact.  
**Acceptance Criteria**  
- `seal_artifact` rejects non-`Preservable` objects.
- Burn engine deletes all `Temporary` and `Intermediate` objects.  
**Dependencies** Issue 13, Issue 35  
**Labels** `object-engine`, `P1`, `feat`, `Milestone1`  
**Estimate** Small

---

## Policy Engine (18–22)

### Issue 18: Policy engine skeleton – grammar loading

**Description**  
Implement `PolicyEngine` loading grammar from JSON or TOML. Grammar defines: admitted object classes, primitive set, allowed views, preservation law, termination law.  
**Acceptance Criteria**  
- Loads Decision Chamber grammar.
- Provides `is_object_type_allowed(world, type)`.
- Invalid grammars rejected at load time.  
**Dependencies** None  
**Labels** `policy-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 19: Permitted primitive validation

**Description**  
Add `PolicyEngine::is_primitive_allowed(world_id, primitive_name, current_phase)`.  
**Acceptance Criteria**  
- Returns `true` only if primitive is in grammar's operation set and allowed in current phase.  
**Dependencies** Issue 18  
**Labels** `policy-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

### Issue 20: Preservation-law checker

**Description**  
Implement `PolicyEngine::can_preserve_object(object, grammar)`. Returns true only if object's class is in grammar's `preservable_classes` and lifecycle conditions satisfied.  
**Acceptance Criteria**  
- For Decision Chamber, only `decision_summary` passes.
- Non-preservable classes can never cross into artifact vault.
- Conditional preservation requires lifecycle satisfaction.  
**Dependencies** Issue 18  
**Labels** `policy-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

### Issue 21: Termination-law validation

**Description**  
Add `PolicyEngine::validate_termination(world_state, requested_mode)`.  
**Acceptance Criteria**  
- Returns error if requesting preservation but no eligible artifact.  
**Dependencies** Issue 20  
**Labels** `policy-engine`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 22: State transition legality

**Description**  
Implement `PolicyEngine::is_transition_legal(current_phase, target_phase, grammar)`.  
**Acceptance Criteria**  
- Returns false if target phase requires convergence but world not converged.
- Called by lifecycle controller before transition.  
**Dependencies** Issue 18, Issue 7  
**Labels** `policy-engine`, `P1`, `feat`, `Milestone1`  
**Estimate** Small

---

## Interpreter & Lifecycle Controller (23–26)

### Issue 23: Closed interpreter request type

**Description**  
Define `TransitionRequest` enum: `CreateObject`, `LinkObjects`, `ChallengeObject`, `GenerateAlternative`, `RankSet`, `SynthesizeSet`, `CondenseObject`, `SealArtifact`, `TriggerBurn`. Each variant carries typed parameters.  
**Acceptance Criteria**  
- Serialization/deserialization for logging.
- Type-safe parameters.  
**Dependencies** None  
**Labels** `interpreter`, `P0`, `feat`, `Milestone1`  
**Estimate** Small

### Issue 24: Interpreter request validation pipeline

**Description**  
Implement `Interpreter::submit(request, world_id)` performing in order: type compatibility, capability possession, lifecycle legality, preservation-law legality, world-scope correctness.  
**Acceptance Criteria**  
- Each check has dedicated function returning `Result`.
- On failure, returns descriptive error and does not mutate state.
- Interpreter is the only public route for world evolution.
- Direct state mutation outside interpreter is prohibited by code structure.  
**Dependencies** Issue 23, Issue 19  
**Labels** `interpreter`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

**Risk** Backdoors through helper utilities. State mutation outside interpreter. Hidden side effects in primitive execution. Mitigate by code review and structural prohibition.

### Issue 25: Interpreter state transition application

**Description**  
After validation, apply transition by calling appropriate operation engine method. Wrap in simple in-memory lock for atomicity.  
**Acceptance Criteria**  
- State changes only after all validations pass.
- Audit log records request and outcome.  
**Dependencies** Issue 24  
**Labels** `interpreter`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

### Issue 26: Lifecycle controller – phase advancement and finalization authority

**Description**  
Implement `LifecycleController` owning phase machine. Coordinates with policy engine, convergence checker, and burn engine. Implements three-part finalization: (1) finalizer proposes, (2) substrate validates, (3) human/policy authorizes. For Phase 0, CLI confirmation as human authority.  
**Acceptance Criteria**  
- Cannot advance to ConvergenceReview without convergence check passing.
- Without explicit confirmation, artifact is not sealed.
- Calls policy engine for legal transition.  
**Dependencies** Issue 7, Issue 22  
**Labels** `lifecycle`, `P0`, `feat`, `Milestone1`  
**Estimate** Medium

---

# Milestone 2 — Primitive Enforcement (Issues 27–42)

## Capability System (27–32)

### Issue 27: Capability token definition and issuance

**Description**  
Define `CapabilityToken` with fields: `token_id`, `world_id`, `epoch`, `principal`, `permitted_operation`, `permitted_object_types`, `issued_at`, `expires_at`, `revoked_flag`. Implement `CapabilitySystem::issue_token(...)`.  
**Acceptance Criteria**  
- Tokens are world- and epoch-scoped.
- No cross-world capability.  
**Dependencies** Issue 5, Issue 33 (epoch)  
**Labels** `capability`, `P0`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 28: Capability possession check

**Description**  
Implement `CapabilitySystem::check_capability(world_id, token_id, operation, object_type)`.  
**Acceptance Criteria**  
- Revoked tokens fail. Expired tokens fail. Wrong epoch fails.
- Tokens checked cheaply and deterministically.  
**Dependencies** Issue 27  
**Labels** `capability`, `P0`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 29: Epoch advancement and capability invalidation

**Description**  
On lifecycle phase transition, `CapabilitySystem::invalidate_epoch(world_id, old_epoch)`. All old-epoch tokens become invalid unless re-issued.  
**Acceptance Criteria**  
- After epoch change, old tokens fail check.
- New tokens issued under new epoch work.
- Exploratory capabilities invalid in finalization unless reissued.
- Stale tokens fail hard.  
**Dependencies** Issue 28, Issue 33  
**Labels** `capability`, `P0`, `feat`, `Milestone2`  
**Estimate** Medium

**Risk** Stale handles remaining usable. Race conditions on epoch advancement. Mitigate by testing and sparse logging of epoch transitions.

### Issue 30: Capability revocation API

**Description**  
Add `CapabilitySystem::revoke_token(token_id)`. Immediate effect on all checks.  
**Acceptance Criteria**  
- Revoked token fails check even if not expired.
- Audit log records revocation.  
**Dependencies** Issue 27  
**Labels** `capability`, `P1`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 31: Capability requirements on objects

**Description**  
Operation engine verifies that request's capability token satisfies all requirements of target object before mutation.  
**Acceptance Criteria**  
- Attempt to mutate object without required capability fails.
- Test: create object requiring `rank_set` capability, try without token → error.  
**Dependencies** Issue 14, Issue 28  
**Labels** `capability`, `P1`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 32: Capability issuance templates by lifecycle phase

**Description**  
Define capability templates: exploratory (broad creation), convergence (narrowed, add seal), finalization (seal and burn only). Issue tokens on world creation. Reissue on epoch advancement.  
**Acceptance Criteria**  
- No world starts with finalization-only powers.
- Action space narrows as epochs advance.  
**Dependencies** Issue 27, Issue 29  
**Labels** `capability`, `P1`, `feat`, `Milestone2`  
**Estimate** Medium

---

## Finite Primitive Algebra (33–42)

### Issue 33: Primitive – create_object

**Description**  
Implement operation handler for `CreateObject`. Takes type, payload, lifecycle_class, preservable_flag. Validates payload schema, assigns unique ID.  
**Acceptance Criteria**  
- Validates payload schema.
- Requires capability with `create_object` permission.  
**Dependencies** Issue 13  
**Labels** `operation-engine`, `P0`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 34: Primitive – link_objects

**Description**  
Creates directed edge between two objects (source → target). Store in adjacency list per world. Validate both exist in same world.  
**Acceptance Criteria**  
- Link queryable for graph view.
- No duplicate links.
- Burn engine deletes links.  
**Dependencies** Issue 15  
**Labels** `operation-engine`, `P0`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 35: Primitive – challenge_object

**Description**  
Marks an object as challenged with challenge text. May block convergence.  
**Acceptance Criteria**  
- Object gets `challenged` flag and challenge text.
- Convergence checker can detect unresolved challenges.  
**Dependencies** Issue 15  
**Labels** `operation-engine`, `P1`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 36: Primitive – generate_alternative

**Description**  
Creates new `alternative` object linked to parent as "alternative_to".  
**Acceptance Criteria**  
- New object type `alternative`.
- Link type `alternative_to` stored.  
**Dependencies** Issue 33, Issue 34  
**Labels** `operation-engine`, `P1`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 37: Primitive – rank_set

**Description**  
Assigns numeric rank to each object in a set.  
**Acceptance Criteria**  
- Ranking retrievable for summary view.
- Overwrites previous rank.  
**Dependencies** Issue 15  
**Labels** `operation-engine`, `P1`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 38: Primitive – synthesize_set

**Description**  
Creates new synthesis object aggregating inputs, linked to all source objects.  
**Acceptance Criteria**  
- Creates object with type from grammar (e.g., `recommendation`).
- Links to all source objects.  
**Dependencies** Issue 33, Issue 34  
**Labels** `operation-engine`, `P1`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 39: Primitive – condense_object

**Description**  
Replaces payload with condensed version, stores original in `previous_payloads`. Used to reduce size before sealing.  
**Acceptance Criteria**  
- Does not change object ID.
- Current payload becomes condensed.  
**Dependencies** Issue 15  
**Labels** `operation-engine`, `P2`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 40: Primitive – seal_artifact

**Description**  
Moves `Preservable` object permitted by grammar into artifact vault. Only way an object survives termination.  
**Acceptance Criteria**  
- Object copied to vault with minimal provenance.
- Fails if object type not in grammar's preservable classes.
- Direct vault writes impossible without policy pass.  
**Dependencies** Issue 20, Issue 59  
**Labels** `operation-engine`, `P0`, `feat`, `Milestone2`  
**Estimate** Medium

### Issue 41: Primitive – trigger_burn

**Description**  
Initiates world termination. Routes through lifecycle controller.  
**Acceptance Criteria**  
- Only callable with appropriate capability.
- Delegates to `WorldEngine::terminate_world`.  
**Dependencies** Issue 8, Issue 30  
**Labels** `operation-engine`, `P0`, `feat`, `Milestone2`  
**Estimate** Small

### Issue 42: Determinism harness

**Description**  
Record transition requests, replay on same world-state, compare outputs. Flag nondeterministic paths.  
**Acceptance Criteria**  
- Deterministic substrate operations for same state and same request.
- Nondeterministic behavior limited to explicitly approved orchestration layer.  
**Dependencies** Issue 25  
**Labels** `interpreter`, `P1`, `test`, `Milestone2`  
**Estimate** Medium

---

# Milestone 3 — Burn Engine (Issues 43–58)

## Key Hierarchy & Cryptographic Burn (43–48)

### Issue 43: Key generation for world

**Description**  
Implement `CryptoProvider::generate_world_key()` returning symmetric key `K_w` (AES-256-GCM or ChaCha20-Poly1305). Store in memory only, never persisted in clear.  
**Acceptance Criteria**  
- Key size 256 bits. Secure RNG. World-scoped, not shared.  
**Dependencies** None  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Small

### Issue 44: Substrate sealing key `K_s` and key wrapping

**Description**  
Generate persistent substrate key `K_s` (for Phase 0, store encrypted with passphrase). Implement `wrap_key(K_w, K_s)`. `K_w` never touches disk unencrypted.  
**Acceptance Criteria**  
- Wrapped key storable in world metadata.
- Unwrapping requires `K_s`.
- Without `K_s`, `K_w` cannot be recovered.  
**Dependencies** Issue 43  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 45: World state encryption using `K_w`

**Description**  
All world objects' payloads encrypted before storage using `K_w` with AEAD mode. Store IV/nonce alongside ciphertext.  
**Acceptance Criteria**  
- On-disk object store contains ciphertext only.
- On world load, decrypt using `K_w` (unwrapped from `K_s`).
- Burn destroys `K_w` from memory.  
**Dependencies** Issue 43, Issue 44  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Large

**Risk** Key leakage into logs. Key material lingering in memory. Wrapping design accidentally preserving recovery path. Mitigate by using `mlock` to prevent swapping, explicit zeroing, and code review.

### Issue 46: Cryptographic burn – destroy `K_w`

**Description**  
Implement `BurnEngine::cryptographic_burn(world_id)`: zero `K_w` from memory, delete cached copies, invalidate unwrap path by discarding wrapped key.  
**Acceptance Criteria**  
- No world-scoped key remains after cryptographic burn.
- Retained ciphertext cannot be reopened through normal runtime paths.  
**Dependencies** Issue 45  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 47: Artifact key `K_a` for sealed artifacts

**Description**  
For each preserved artifact, generate separate `K_a` encrypting artifact payload. `K_a` stored in vault metadata (protected by substrate key). World burn does not destroy `K_a`.  
**Acceptance Criteria**  
- `seal_artifact` generates `K_a`.
- World burn leaves `K_a` intact.  
**Dependencies** Issue 40, Issue 43  
**Labels** `burn-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 48: Burn flow integration – logical + crypto + storage + memory

**Description**  
Create `BurnEngine::burn_world(world_id, mode)` calling in order: logical burn, cryptographic burn, storage cleanup, memory cleanup, then semantic measurement pass.  
**Acceptance Criteria**  
- Single function orchestrates all layers.
- If any step fails, logs error but continues (best effort).
- After burn, world cannot be loaded again.  
**Dependencies** Issue 46, Issue 49, Issue 50, Issue 51  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Large

---

## Storage & Memory Cleanup (49–51)

### Issue 49: Logical burn – revoke capabilities and invalidate handles

**Description**  
Implement `LogicalBurn::revoke_all_world_capabilities(world_id)`. Iterate all capability tokens for world, mark revoked. Invalidate open handles.  
**Acceptance Criteria**  
- All capability checks for this world fail post-burn.
- World cannot be resumed after logical burn.  
**Dependencies** Issue 30  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Small

### Issue 50: Storage cleanup – delete temporary objects and indexes

**Description**  
Remove all files/database entries belonging to world from object store, link store, and capability store. Secondary to cryptographic erasure because physical overwrite on flash is unreliable.  
**Acceptance Criteria**  
- No world files remain on disk after call (under trusted OS).
- Returns list of deleted items for audit.  
**Dependencies** Issue 16  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 51: Memory cleanup – zero runtime structures

**Description**  
Clear in-memory caches, object graphs, transient orchestrator context. Use explicit zeroing for sensitive buffers.  
**Acceptance Criteria**  
- After call, any reference to world ID returns `WorldNotFound`.
- World-scoped inference context removed.  
**Dependencies** Issue 16  
**Labels** `burn-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Medium

---

## Semantic Burn & Validation (52–55)

### Issue 52: Semantic burn analyzer

**Description**  
Post-burn tool attempting to reconstruct world state from remaining artifacts and logs. Measurement tool, not enforcement.  
**Acceptance Criteria**  
- Reads audit logs, vault, and substrate leftovers.
- Outputs reconstructability score (fraction of object payloads recoverable).  
**Dependencies** Issue 48  
**Labels** `burn-engine`, `P2`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 53: Burn engine error handling and idempotency

**Description**  
Calling `burn_world` twice on same world is idempotent. If a step fails, log and continue.  
**Acceptance Criteria**  
- Second burn call returns `Ok(BurnAlreadyCompleted)`.
- No panic on missing files.  
**Dependencies** Issue 48  
**Labels** `burn-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Small

### Issue 54: Burn audit logging

**Description**  
Record: burn start, each layer completion, errors, final status. Include world ID, termination mode, timestamp.  
**Acceptance Criteria**  
- Audit file contains burn trace.
- No world internals (payloads) logged.  
**Dependencies** Issue 48  
**Labels** `burn-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Small

### Issue 55: Burn engine skeleton (initial implementation)

**Description**  
Create placeholder `BurnEngine` struct with methods for each layer. Initially just logs. Full implementation filled by Issues 43–54.  
**Acceptance Criteria**  
- Compiles. Can be called without crashing.  
**Dependencies** None  
**Labels** `burn-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Small

---

## State Engine & Views (56–58)

### Issue 56: Live object graph in state engine

**Description**  
Store world as explicit graph, not a bag of files. Support typed edges, reachability queries, graph pruning during burn.  
**Acceptance Criteria**  
- All world internals trace through world graph.
- No hidden side state drives chamber meaning.  
**Dependencies** Issue 13, Issue 34  
**Labels** `state-engine`, `P0`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 57: Convergence review state tracking

**Description**  
Track: unresolved contradiction set, mandatory-class completion map, candidate artifact pointer, finalization readiness flags.  
**Acceptance Criteria**  
- Convergence inspectable as substrate state.
- Finalizer cannot rely on hidden model-only judgment.  
**Dependencies** Issue 56  
**Labels** `state-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Medium

### Issue 58: Temporary render state isolation

**Description**  
Isolate render state. Tag as temporary. Ensure render state participates in burn.  
**Acceptance Criteria**  
- Views do not create undeclared persistent traces.
- Render caches cleared on burn.  
**Dependencies** Issue 56  
**Labels** `view-engine`, `P1`, `feat`, `Milestone3`  
**Estimate** Small

---

# Milestone 4 — Decision Chamber (Issues 59–75)

## Artifact Vault (59–62)

### Issue 59: Vault storage model

**Description**  
Create sole authorized cross-world channel. Separate vault persistence domain. Define artifact schema, minimal provenance metadata, append-only semantics.  
**Acceptance Criteria**  
- Only approved artifact classes enter the vault.
- World internals cannot be stored unless explicitly part of artifact schema.
- Vault persists across substrate restarts.  
**Dependencies** Issue 47  
**Labels** `artifact-vault`, `P0`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 60: Provenance minimization

**Description**  
Define minimal provenance: `artifact_id`, `source_world_id`, `artifact_class`, `sealed_at`, minimal policy metadata. No rich internal graph metadata by default.  
**Acceptance Criteria**  
- Provenance policy documented.
- Hidden graph topology, discarded branches, and intermediate state not implicitly preserved.  
**Dependencies** Issue 59  
**Labels** `artifact-vault`, `P1`, `feat`, `Milestone4`  
**Estimate** Small

**Risk** Convenience pressure to over-store provenance. Future institutional needs pushing metadata creep. Mitigate by treating this as a core architectural boundary, not a configurable preference.

### Issue 61: Vault access boundaries

**Description**  
Policy-gate reads. Prohibit live-world namespace access. Allow only artifact retrieval semantics.  
**Acceptance Criteria**  
- Worlds do not address other worlds directly.
- Vault is the only cross-world channel.  
**Dependencies** Issue 59  
**Labels** `artifact-vault`, `P0`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 62: Audit leakage review for vault

**Description**  
Verify vault metadata does not constitute a hidden residue source. Review what cross-world inferences are possible from vault contents alone.  
**Acceptance Criteria**  
- Documented analysis of vault as residue boundary.
- Accepted residue documented; unaccepted residue mitigated.  
**Dependencies** Issue 60  
**Labels** `security`, `P1`, `test`, `Milestone4`  
**Estimate** Small

---

## Grammar & Convergence (63–68)

### Issue 63: Decision Chamber grammar file

**Description**  
JSON/TOML grammar file: object classes, primitive set, allowed views, preservation law, termination law per paper Section 9.  
**Acceptance Criteria**  
- Loadable by policy engine.
- Contains exactly the listed classes and primitives.  
**Dependencies** None  
**Labels** `decision-chamber`, `P0`, `feat`, `Milestone4`  
**Estimate** Small

### Issue 64: Decision Chamber object type schemas

**Description**  
Define payload schemas for: `premise`, `support_statement`, `constraint`, `risk`, `upside`, `contradiction`, `alternative`, `recommendation`, `decision_summary`.  
**Acceptance Criteria**  
- JSON Schema for each type.
- Object engine validates against these.
- No binary payloads allowed.  
**Dependencies** Issue 11  
**Labels** `decision-chamber`, `P0`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 65: Convergence checker for Decision Chamber

**Description**  
Given world state and grammar, returns true if: (1) `decision_summary` exists, (2) mandatory objects present or discharged, (3) no unresolved contradictions if grammar says they block.  
**Acceptance Criteria**  
- Unresolved contradictions block convergence.
- Returns false if summary missing.  
**Dependencies** Issue 63  
**Labels** `decision-chamber`, `P0`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 66: Grammar semantics tests

**Description**  
Test contradiction links, preservation-blocking conditions, mandatory-object resolution, abort path.  
**Acceptance Criteria**  
- Grammar behavior explicit enough for benchmarking.
- No hidden semantic rules outside code and docs.  
**Dependencies** Issue 65  
**Labels** `decision-chamber`, `P0`, `test`, `Milestone4`  
**Estimate** Medium

### Issue 67: Decision Chamber view implementations

**Description**  
Four views: `conversation_view` (linear log), `graph_view` (objects and links), `summary_view` (`decision_summary`), `burn_view` (destruction confirmation). Views are read-only projections.  
**Acceptance Criteria**  
- Each view returns structured data (JSON or plain text).
- Conversation is rendering of world-state, not independent persistence channel.
- Graph derived from world-state only.
- Summary view cannot show undeclared survivors.
- Burn view does not persist world-state improperly.  
**Dependencies** Issue 34, Issue 56  
**Labels** `decision-chamber`, `P1`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 68: Sparse audit layer implementation

**Description**  
Log only: world creation, convergence proposal, validation result, artifact sealed (or not), burn completion, policy violation. No world internals by default.  
**Acceptance Criteria**  
- Logs human-readable.
- No payload or object content stored.
- Audit policy documented and testable.  
**Dependencies** None  
**Labels** `audit`, `P0`, `feat`, `Milestone4`  
**Estimate** Small

---

## Orchestration (69–72)

### Issue 69: Symbolic planner (primary orchestrator)

**Description**  
Build rules-based intent interpreter and deterministic planner for Decision Chamber. Map chamber evolution to substrate primitives. Avoid premature LLM dependence.  
**Acceptance Criteria**  
- One full chamber runs without an LLM.
- Architecture remains meaningful without large model dependence.  
**Dependencies** Issues 33–41  
**Labels** `orchestration`, `P0`, `feat`, `Milestone4`  
**Estimate** Large

### Issue 70: Optional small model path

**Description**  
Add model-assisted orchestration only if symbolic planning is too brittle. Model output routes through substrate primitives.  
**Acceptance Criteria**  
- Model has no direct persistence control.
- Model output validated by interpreter before execution.  
**Dependencies** Issue 69  
**Labels** `orchestration`, `P2`, `feat`, `Milestone4`  
**Estimate** Medium

### Issue 71: LLM path (if needed)

**Description**  
Support more capable orchestrator without breaking world law. Treat KV cache and prompt context as world-state. Burn model context per world. Keep weights substrate-scoped.  
**Acceptance Criteria**  
- Model cannot define new primitives.
- Model cannot widen its own capabilities.
- Model context cleared on burn.  
**Dependencies** Issue 70  
**Labels** `orchestration`, `P3`, `feat`, `Milestone4`  
**Estimate** Large

**Risk** Jailbreaks leading to unintended primitive sequences. Indirect information flow through allowed sequences. Model context leakage in traces or debugging.

### Issue 72: Audit leakage review for orchestration layer

**Description**  
Inspect log contents, stack traces, error payloads, debug flags for hidden persistence from orchestration.  
**Acceptance Criteria**  
- No object payloads leak into logs by default.
- No hidden debug mode undermines sealed-world assumptions.  
**Dependencies** Issue 69  
**Labels** `security`, `P1`, `test`, `Milestone4`  
**Estimate** Small

---

## End-to-End (73–75)

### Issue 73: E2E test – create, converge, preserve, burn

**Description**  
Create world, add premise/support/constraint/alternative, rank, synthesize recommendation, create summary, propose convergence, confirm sealing, trigger burn. Verify artifact in vault and world burned.  
**Acceptance Criteria**  
- Test passes. No panic or leak.  
**Dependencies** Issues 63–69  
**Labels** `decision-chamber`, `P0`, `test`, `Milestone4`  
**Estimate** Large

### Issue 74: E2E test – abort burn without artifact

**Description**  
Create world, add objects, trigger abort before convergence. Verify no artifact in vault.  
**Acceptance Criteria**  
- Vault empty for this world.
- All objects burned.  
**Dependencies** Issue 73  
**Labels** `decision-chamber`, `P0`, `test`, `Milestone4`  
**Estimate** Medium

### Issue 75: CLI harness for interactive chamber sessions

**Description**  
CLI tool for researchers to create chamber, submit primitive requests, inspect views, trigger burn. Not a product; a research instrument.  
**Acceptance Criteria**  
- Usable for benchmark task execution.  
**Dependencies** Issue 73  
**Labels** `cli`, `P2`, `feat`, `Milestone4`  
**Estimate** Medium

---

# Milestone 5 — Benchmark Harness (Issues 76–103)

## Baseline Implementations (76–80)

### Issue 76: Disposable VM baseline

**Description**  
Script launching lightweight VM, running bounded decision task (same as Chamber), destroying VM. Measure residue: leftover disk blocks, memory snapshots, logs. Ephemeral guest image, no persistent disk save.  
**Acceptance Criteria**  
- Automated, repeatable runs.
- Baseline configured tightly enough to be credible.  
**Dependencies** None  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Large

### Issue 77: Constrained microVM baseline (Firecracker)

**Description**  
Firecracker microVM: minimal Linux kernel, rootfs in ramfs, no network, single process. Run same decision task as shell script. Destroy microVM. Measure residue.  
**Acceptance Criteria**  
- Baseline strong enough that a Chambers win is meaningful.
- Comparison not against a weak strawman.  
**Dependencies** None  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Large

### Issue 78: Baseline task definition

**Description**  
Define concrete, reproducible decision task (e.g., "Choose between three cloud providers based on cost, latency, compliance"). Write task scripts for each baseline simulating same reasoning steps.  
**Acceptance Criteria**  
- Task specification in markdown.
- Same input data across all conditions.  
**Dependencies** Issue 76, Issue 77  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 79: Chamber orchestrator for benchmark task

**Description**  
Adapt symbolic orchestrator to perform exact same decision task. Ensure steps are comparable.  
**Acceptance Criteria**  
- Produces same final decision output (text).  
**Dependencies** Issue 69, Issue 78  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 80: Baseline VM/Firecracker automation

**Description**  
Automate VM image building (Packer or similar) and Firecracker setup script. Ensure reproducibility.  
**Acceptance Criteria**  
- One command builds each baseline.
- Works on Ubuntu 22.04+.  
**Dependencies** Issue 76, Issue 77  
**Labels** `infra`, `P1`, `feat`, `Milestone5`  
**Estimate** Medium

---

## Residue Measurement (81–85)

### Issue 81: Object recovery tool

**Description**  
Post-termination scanner: check all substrate storage (object store, logs, vault, memory dumps) and attempt to recover non-preserved object payloads. Output `recoverable_object_fraction`.  
**Acceptance Criteria**  
- Works for Chamber and VM baselines (adapt to filesystem scanning).
- Returns metric 0.0–1.0.  
**Dependencies** Issue 52  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 82: Graph edge recovery measurement

**Description**  
Extend residue tool to reconstruct link/edge structure. Compare with original graph (logged during run). Output `recoverable_edge_fraction`.  
**Acceptance Criteria**  
- Detects edges from filesystem traces for VM baselines.
- For Chamber, checks vault, logs, storage.  
**Dependencies** Issue 81  
**Labels** `benchmark`, `P1`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 83: Metadata residue measurement

**Description**  
Measure surviving metadata: timestamps, object counts, operation logs. Output `metadata_survival_count`.  
**Acceptance Criteria**  
- Counts entries in audit logs, filesystem metadata.  
**Dependencies** Issue 81  
**Labels** `benchmark`, `P2`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 84: Evaluator reconstruction time measurement

**Description**  
Measure time required for evaluator (or automated script) to reconstruct intermediate reasoning steps from post-burn residues.  
**Acceptance Criteria**  
- Metric: `reconstruction_time_seconds`.  
**Dependencies** Issue 81  
**Labels** `benchmark`, `P0`, `test`, `Milestone5`  
**Estimate** Medium

### Issue 85: Semantic residue entropy reduction metric

**Description**  
Information-theoretic metric: given post-burn state, compute conditional entropy of original world state. H(original | leftover). Higher entropy = less residue.  
**Acceptance Criteria**  
- Report in bits.
- Compare across conditions.  
**Dependencies** Issue 81  
**Labels** `benchmark`, `P2`, `feat`, `Milestone5`  
**Estimate** Medium

---

## Lifecycle Comprehension Study (86–90)

### Issue 86: User prediction test harness

**Description**  
CLI or web tool presenting participants with world state description, asking: "What will survive?" and "What will be destroyed?" Record answers.  
**Acceptance Criteria**  
- Questions generated from real chamber runs.
- Logs accuracy and confidence.  
**Dependencies** Issue 73  
**Labels** `benchmark`, `P1`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 87: Baseline comprehension test (VM disposable)

**Description**  
Same prediction test for disposable VM environment.  
**Acceptance Criteria**  
- Comparable question set. Results stored for comparison.  
**Dependencies** Issue 86  
**Labels** `benchmark`, `P1`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 88: Survival prediction accuracy scoring

**Description**  
Scoring function comparing user predictions to actual post-burn residue measurements.  
**Acceptance Criteria**  
- Metric computed per participant and aggregated across cohort.  
**Dependencies** Issue 86, Issue 87  
**Labels** `benchmark`, `P1`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 89: Cognitive load questionnaire

**Description**  
NASA-TLX or similar short questionnaire after each test run. Record subjective workload.  
**Acceptance Criteria**  
- Results saved to CSV. Comparable across conditions.  
**Dependencies** Issue 86  
**Labels** `benchmark`, `P2`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 90: Benchmark runner – full comparison

**Description**  
Python script running Chamber task N times, VM baseline N times, microVM baseline N times. Collects all residue and comprehension metrics. Outputs summary table.  
**Acceptance Criteria**  
- Produces table with means and standard deviations.
- Runs in under 2 hours for N=10.  
**Dependencies** Issue 79, Issue 81, Issue 88  
**Labels** `benchmark`, `P0`, `feat`, `Milestone5`  
**Estimate** Large

---

## Hypothesis Testing (91–94)

### Issue 91: H1 test – lower recoverable semantic residue

**Description**  
Compare `recoverable_object_fraction` and `recoverable_edge_fraction` between Chamber and baselines. Use t-test or non-parametric equivalent.  
**Acceptance Criteria**  
- Output p-value and effect size.
- Falsification if p > 0.05 or effect size trivial.  
**Dependencies** Issue 90  
**Labels** `benchmark`, `P0`, `test`, `Milestone5`  
**Estimate** Medium

### Issue 92: H2 test – better user prediction accuracy

**Description**  
Compare prediction accuracy between Chamber and VM baseline. Minimum 5 internal testers.  
**Acceptance Criteria**  
- Statistical test result. Report if Chamber improves accuracy.  
**Dependencies** Issue 88  
**Labels** `benchmark`, `P0`, `test`, `Milestone5`  
**Estimate** Medium

### Issue 93: H3 test – reconstructable intermediate reasoning traces

**Description**  
Compare reconstruction time/feasibility across conditions.  
**Acceptance Criteria**  
- Lower time for Chamber is positive.
- Metric: `reconstruction_time_seconds`.  
**Dependencies** Issue 84  
**Labels** `benchmark`, `P0`, `test`, `Milestone5`  
**Estimate** Medium

### Issue 94: Falsification report generator

**Description**  
Script taking benchmark results, outputting "thesis supported / not supported" verdict based on predefined thresholds (e.g., residue reduction > 50% or p < 0.05).  
**Acceptance Criteria**  
- Human-readable report.
- Includes raw data link.
- Metrics defined before results known.  
**Dependencies** Issue 91, Issue 92, Issue 93  
**Labels** `benchmark`, `P1`, `feat`, `Milestone5`  
**Estimate** Small

---

## Infrastructure & Reproducibility (95–99)

### Issue 95: Deterministic random seed control

**Description**  
Ensure all RNG (keys, object IDs, ordering) can be seeded with fixed value.  
**Acceptance Criteria**  
- CLI flag `--seed 42`. Same seed produces identical world state sequence.  
**Dependencies** Issue 43  
**Labels** `infra`, `P1`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 96: Benchmark artifact storage

**Description**  
Store each benchmark run's raw data (residue measurements, logs, user responses) in structured format (JSON lines).  
**Acceptance Criteria**  
- Each run gets unique ID. Data re-analyzable.  
**Dependencies** Issue 90  
**Labels** `infra`, `P1`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 97: Benchmark result visualization

**Description**  
Generate plots (bar charts, box plots) comparing residue fractions across conditions.  
**Acceptance Criteria**  
- Outputs PNG/PDF. Includes error bars.  
**Dependencies** Issue 90  
**Labels** `benchmark`, `P2`, `feat`, `Milestone5`  
**Estimate** Small

### Issue 98: CI integration for benchmarks (optional)

**Description**  
Run full benchmark suite nightly. Report results to dashboard.  
**Acceptance Criteria**  
- Fails if performance degrades beyond threshold.  
**Dependencies** Issue 90  
**Labels** `infra`, `P3`, `feat`, `Milestone5`  
**Estimate** Medium

### Issue 99: Storage and flash reality check documentation

**Description**  
Document flash limitations, cryptographic erasure assumptions, limits of post-burn claims. Ensure burn claims remain faithful to actual storage behavior.  
**Acceptance Criteria**  
- Docs do not imply guaranteed physical overwrite.  
**Dependencies** Issue 48  
**Labels** `doc`, `P1`, `docs`, `Milestone5`  
**Estimate** Small

---

## Cleanup & Final Validation (100–103)

### Issue 100: Code review for substrate trust assumptions

**Description**  
Manual review of all substrate components for hidden persistence (accidental caching, undeclared logging).  
**Acceptance Criteria**  
- List of found issues fixed or documented.
- No silent persistence.  
**Dependencies** All previous  
**Labels** `security`, `P0`, `task`, `Milestone5`  
**Estimate** Medium

### Issue 101: Burn engine verification – cryptographic erasure test

**Description**  
Test running world, performing burn, then attempting to recover `K_w` from memory dump. Must be impossible under trusted substrate. Use `mlock` to prevent swapping.  
**Acceptance Criteria**  
- Test passes.  
**Dependencies** Issue 46  
**Labels** `burn-engine`, `P0`, `test`, `Milestone5`  
**Estimate** Medium

### Issue 102: Object smuggling prevention test

**Description**  
Attempt to create object with Base64-encoded binary exceeding bounds. Must be rejected.  
**Acceptance Criteria**  
- Rejection with `InvalidPayload`. Test in CI.  
**Dependencies** Issue 12  
**Labels** `object-engine`, `P0`, `test`, `Milestone5`  
**Estimate** Small

### Issue 103: Final Phase 0 acceptance report and documentation

**Description**  
Write report summarizing: which acceptance criteria are met, benchmark results, falsification outcomes, open issues. Complete README with build instructions, how to run chamber, how to run benchmarks. Formal model document: tuple mapping, lifecycle state machine, primitive algebra spec, capability model, preservation law, burn semantics. Researcher operator guide: setup, task scripts, baseline comparison, residue inspection.  
**Acceptance Criteria**  
- Reviewed by project lead.
- New developer can go from clone to passing tests in < 30 minutes.
- Stored in repository.  
**Dependencies** All previous  
**Labels** `doc`, `P0`, `docs`, `Milestone5`  
**Estimate** Large

---

# Cross-Cutting Labels Summary

| Label | Count |
|-------|-------|
| `world-engine` | 6 |
| `object-engine` | 7 |
| `policy-engine` | 5 |
| `interpreter` | 4 |
| `lifecycle` | 1 |
| `capability` | 6 |
| `operation-engine` | 9 |
| `burn-engine` | 15 |
| `artifact-vault` | 4 |
| `state-engine` | 2 |
| `view-engine` | 1 |
| `decision-chamber` | 7 |
| `orchestration` | 3 |
| `audit` | 1 |
| `cli` | 1 |
| `benchmark` | 18 |
| `infra` | 5 |
| `security` | 3 |
| `doc` | 6 |

**Total issues: 103**

---

# Team Roles

| Role | Owns |
|------|------|
| **Research lead** | Formal model, evaluation design, research integrity, decision gate |
| **Runtime engineer** | Substrate core, interpreter, state engine, burn engine |
| **Security engineer** | Threat review, key hierarchy, residue inspection |
| **Systems engineer** | VM/microVM baselines and benchmarking environment |
| **UX / interaction engineer** | Minimal views and lifecycle legibility |

---

# Suggested Implementation Order

1. Repository, ADRs, and security assumptions (1–3)
2. Formal tuple-to-code mapping (4)
3. World engine (5–10)
4. Object engine and payload enforcement (11–17)
5. Policy engine (18–22)
6. Interpreter and lifecycle controller (23–26)
7. Capability system with epoch revocation (27–32)
8. Primitive algebra (33–42)
9. Key hierarchy and cryptographic burn (43–48)
10. Storage and memory cleanup (49–51)
11. Semantic burn and validation (52–55)
12. State engine and views (56–58)
13. Artifact vault with provenance minimization (59–62)
14. Decision Chamber grammar and convergence (63–68)
15. Orchestration: symbolic first, model optional (69–72)
16. End-to-end integration (73–75)
17. Baseline systems (76–80)
18. Residue and comprehension measurement (81–90)
19. Hypothesis testing (91–94)
20. Infrastructure, reproducibility, and final validation (95–103)

---

# Final Decision Gate Checklist

Before declaring Phase 0 complete, answer each question:

- [ ] Does the runtime enforce world law, or is it just a wrapper?
- [ ] Is the primitive algebra actually closed at runtime?
- [ ] Does burn destroy a typed world or just delete files?
- [ ] Is the vault the only cross-world channel?
- [ ] Do H1–H3 have preliminary directional results?
- [ ] Is the comparison fair to VM and microVM baselines?
- [ ] Would a neutral reviewer consider this a distinct runtime category?

If any answer is "no," Phase 0 is either incomplete or has falsified the hypothesis. Both are acceptable outcomes.
