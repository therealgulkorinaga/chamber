# Chambers Level 1 — Detailed Issue List (Hardened)
## Thin Research UI on Top of the Substrate

This issue list expands the Level 1 PRD into **implementation epics, technical issues, dependencies, risks, deliverables, and acceptance criteria** with a strong focus on **eliminating vulnerabilities** (hidden persistence, bypass, cross‑world leakage, UI drift).

Level 1 assumes Level 0 exists (substrate, interpreter, burn engine, vault, benchmark harness).  
Level 1 is a **visual and operational shell** – not a new runtime.

---

# 0. Program‑level success criteria (hardened)

Level 1 succeeds **only if** all of the following are true:

1. A user can create and operate a Decision Chamber without CLI.
2. Every meaningful state transition still routes through substrate law (adapter forwards, never decides).
3. Lifecycle phases are visible and understandable.
4. Legal actions are surfaced correctly and update with world‑state.
5. Preservation and burn are legible and cannot be bypassed.
6. Residue and benchmark outputs are inspectable through the UI.
7. The UI does **not** recreate application‑centric persistence or hidden state.
8. **No cross‑world leakage** – UI never shows objects from wrong world.
9. **Post‑burn freeze** – no action on a destroyed world succeeds.

---

# 1. Epic: Level 1 architecture and repo extension (hardened)

## 1.1 Define Level 1 system boundary (security critical)
**Objective**  
Write down exactly what belongs in the UI shell, what in the adapter, and what remains substrate‑only.  

**Tasks**  
- `level1_architecture.md` with explicit **forbidden patterns**:  
  - Adapter performing policy/capability checks.  
  - UI caching world state beyond a single render cycle.  
  - Any client‑side object graph modification.  
- ADR: “UI is subordinate to substrate law”.  

**Acceptance criteria**  
- No ambiguity about who owns world truth.  
- Security review confirms adapter has no policy logic.

## 1.2 Extend repository structure for Level 1
**Tasks**  
- Create `/ui`, `/adapter`, `/ui-tests`, `/ui-fixtures`, `/ux-research`.  

**Acceptance** – runtime and UI stay separate; no substrate logic copied into UI.

---

# 2. Epic: API adapter layer (hardened – no policy)

## 2.1 Define adapter contract (stateless forwarder)
**Required operations** (each maps 1:1 to substrate API):  
- create_world, get_world_state, get_object_graph, get_legal_actions, invoke_primitive, propose_convergence, validate_convergence, authorize_sealing, burn_world, get_residue_report, get_benchmark_report.  

**Security rule:** Adapter **must not** perform any policy or capability check. It only validates request format and forwards.  

**Acceptance** – Adapter code reviewed for policy logic; none found.

## 2.2 Build state fetch endpoints (read‑only)
**Tasks**  
- `getWorld`, `getObjects`, `getObjectGraph`, `getLifecycle`, `getCapabilities`.  

**Acceptance**  
- Responses are **read‑only views** over substrate state.  
- Data includes world ID and epoch – UI must verify epoch matches current view.  
- After burn, `getWorld` returns `WorldDestroyed` error.

## 2.3 Build action endpoints (with replayability)
**Tasks**  
- `getLegalActions` (returns list from substrate, not computed in adapter).  
- `invokePrimitive` – passes through, returns substrate result.  
- Error response format includes `error_code`, `message` (no payload leakage).  

**Acceptance** – Adapter cannot bypass interpreter; invalid actions return errors; all calls logged for replay.

## 2.4 Build finalisation endpoints
**Tasks**  
- `proposeConvergence`, `validateConvergence`, `authorizeSealing`, `getFinalizationState`.  

**Acceptance** – Model/UI cannot unilaterally preserve; substrate validation is final.

## 2.5 Build burn and residue endpoints
**Tasks**  
- `burnWorld`, `getBurnStatus`, `getResidueReport`, `getBenchmarkComparison`.  

**Acceptance** – Burn state updates correctly; residue output accessible after termination; no hidden reopen path.

---

# 3. Epic: UI shell foundation (no hidden state)

## 3.1 Select UI stack – local‑first, auditable
**Constraints**  
- Must allow complete disable of localStorage/IndexedDB.  
- No automatic cloud sync.  
- State management library (if any) must not persist by default.  

**Acceptance** – UI can run with all persistence APIs mocked/blocked.

## 3.2 Implement app shell (no chamber logic)
**Tasks**  
- Shell layout, routing, error boundary, loading states.  

**Acceptance** – Shell contains no chamber logic; recovers cleanly from adapter failures.

## 3.3 Define UI state discipline (enforced by tests)
**Allowed ephemeral UI state:** current selection, panel open/closed, viewport zoom, last fetch result (memory only), request status.  

**Forbidden:**  
- Shadow object store.  
- Local chamber history DB.  
- Unsanctioned artifact cache.  
- Hidden draft buffers.  
- Extra lifecycle tracker.  
- Browser persistence of chamber internals.  

**Acceptance** – Automated test verifies no writes to localStorage/IndexedDB during normal operation.

---

# 4. Epic: Chamber creation surface (no import path)

## 4.1 Build new chamber screen
**Tasks**  
- Grammar selector (only Decision Chamber).  
- Objective input (validate length, encoding).  
- Create button → calls adapter.  

**Acceptance** – World created successfully; world ID shown; no hidden file upload or blob injection.

## 4.2 Creation validation
**Tasks**  
- Reject objective that is empty, too long (>10KB), or contains binary/control chars beyond plain text.  

**Acceptance** – Malformed input rejected clearly; no hidden import path.

---

# 5. Epic: World‑state surface (no stale or cross‑world data)

## 5.1 Build world summary panel
**Must show:** world ID, grammar, objective, lifecycle phase, epoch, candidate artifact status, termination mode (if ended).  

**Acceptance** – Updates live from adapter; reflects burn and convergence correctly.

## 5.2 Build object inventory panel
**Must show:** object ID, type, lifecycle class, preservable flag, modified time, selection state.  

**Acceptance** – Objects only from active substrate state; no pseudo‑objects.

## 5.3 Build object detail panel
**Must show:** payload (as rendered schema), outgoing links, incoming links, legal transformations, capability‑relevant actions.  

**Acceptance** – Payload shown does not exceed schema/size bounds; no binary dump.

## 5.4 Graph view (read‑only, no mutation)
**Must show:** nodes by type, typed links, contradiction visibility, candidate artifact position.  

**Strict rule:** No drag‑and‑drop editing. Graph is derived **only** from substrate.  

**Acceptance** – Graph toolkit does not store extra local state beyond ephemeral UI viewport.

---

# 6. Epic: Legal action surface (dynamic, substrate‑driven)

## 6.1 Build action panel
**Tasks**  
- Fetch legal actions from adapter (not computed in UI).  
- Render action buttons; disable/hide unavailable actions.  

**Acceptance** – User never sees a generic toolbox that exceeds current law; panel updates after each transition.

## 6.2 Primitive invocation flow
**Tasks**  
- Build forms for primitive parameters.  
- Map UI form → adapter request.  
- Show validation errors; refresh world‑state on success.  

**Acceptance** – No direct state mutation path exists in UI; invalid calls fail gracefully.

## 6.3 Error semantics (no leakage)
**Tasks**  
- Define user‑facing error classes: `PolicyError`, `LifecycleError`, `CapabilityError`, `SchemaError`.  
- Log errors **without** object payloads or world internals.  

**Acceptance** – User can understand why action failed; logs are not hidden residue channels.

---

# 7. Epic: Convergence and finalisation UI (explicit authority)

## 7.1 Convergence proposal display
**Must show:** proposal present/absent, candidate artifact, unresolved blocking conditions, mandatory object‑class completion status.  

**Acceptance** – Proposal distinct from validated finalisation.

## 7.2 Validation state display
**Must show:** validation passed/failed, failed conditions, preservation legality, blocking contradictions.  

**Acceptance** – User can tell whether issue lies in grammar‑relative structure or policy.

## 7.3 Authorisation surface
**Tasks** – Show whether sealing requires approval; render approve/deny; show consequences.  

**Acceptance** – Model is not presented as unilateral preservation authority.

## 7.4 Termination‑mode display
**Must show:** converged‑preserving, converged‑total‑burn, abort burn.  

**Acceptance** – Modes are understandable and distinct; no ambiguity about what survives.

---

# 8. Epic: Burn interface (explicit, irreversible)

## 8.1 Build burn status surface
**Must show:** logical burn status, key destruction status, storage cleanup, memory cleanup, destroyed flag, artifact preserved or not.  

**Acceptance** – User can tell burn is in progress and when world is irreversibly gone.

## 8.2 Burn completion state
**Must show:** world destroyed, artifact ID (if preserved), residue report link, benchmark comparison link.  

**Acceptance** – Destroyed world cannot be resumed; no stale active‑world view persists.

## 8.3 Abort‑burn handling
**Tasks** – Render abort‑burn as separate path; show no artifact preserved.  

**Acceptance** – Aborted world does not look like failed navigation; termination semantics clear.

---

# 9. Epic: Residue and benchmark visibility (research use)

## 9.1 Residue panel
**Must show:** surviving artifact classes, retained metadata summary, recoverable object fraction, recoverable edge fraction, trace reconstruction indicators (if available).  

**Acceptance** – Panel maps directly to benchmark instrumentation; does not infer extra meaning.

## 9.2 Baseline comparison panel
**Must show:** Chamber run metrics vs disposable VM vs constrained microVM; hypothesis mapping (H1/H2/H3).  

**Acceptance** – Comparison output attributable to concrete run ID; no overclaim.

## 9.3 Export for researchers (controlled)
**Tasks** – Allow export of benchmark report only (JSON/CSV) – **not** chamber internals.  

**Acceptance** – Export does not violate sealed‑world semantics; boundary documented.

---

# 10. Epic: UI‑side security and persistence audit (critical)

## 10.1 Browser/local persistence audit
**Tasks** – Audit localStorage, sessionStorage, IndexedDB, service workers, HTTP cache, crash recovery.  

**Acceptance** – No chamber internals persist locally by default; any unavoidable persistence documented and minimised.

## 10.2 Frontend logging audit
**Tasks** – Inspect console logs, network debugging traces, exception payloads, analytics hooks (none allowed).  

**Acceptance** – No object payloads or world internals leak via logs by default.

## 10.3 Adapter security audit
**Tasks** – Inspect direct state mutation paths, authz enforcement, stale world access after burn, epoch mismatch handling.  

**Acceptance** – Adapter cannot perform actions substrate would reject; stale world/epoch actions fail consistently.

## 10.4 Cross‑world isolation test
**Task** – Automated test: create world A, world B; UI tries to load world B’s objects while viewing world A → must fail.  

**Acceptance** – Test passes.

## 10.5 Post‑burn freeze test
**Task** – Burn world; then attempt to invoke any primitive via UI → must return `WorldDestroyed`.  

**Acceptance** – Test passes.

---

# 11. Epic: UX research instrumentation (no vulnerability)

## 11.1 Lifecycle comprehension study rig
**Tasks** – Scripted tasks, survey prompts, ask participants what survives/burns, identify lifecycle phase, compare with baseline.  

**Acceptance** – Study can be run on internal cohort; outputs map to H2.

## 11.2 Mental‑model study
**Tasks** – Interviews, think‑aloud, categorisation of user descriptions.  

**Acceptance** – Outputs can support or weaken world‑first claim.

## 11.3 Usability defect triage
**Categories:** substrate bug, representation issue, lifecycle confusion, legal‑action discoverability, over‑complexity.  

**Acceptance** – Issues are triaged, not ignored.

---

# 12. Epic: Diagrams and visible system law

## 12.1 Lifecycle state machine visualisation
**Tasks** – Render phases, highlight current, show valid next transitions, show terminal states.  

**Acceptance** – Improves comprehension without adding extra semantics.

## 12.2 Burn hierarchy visualisation
**Tasks** – Render logical/cryptographic/storage/memory/semantic burn stages, show completion, show artifact status.  

**Acceptance** – Users distinguish key destruction from storage cleanup; burn not perceived as simple UI closure.

---

# 13. Epic: Minimal styling and interaction discipline

## 13.1 Styling system
**Requirements** – Sparse layout, high legibility, low ornamentation, research‑tool feel, no “assistant chat” styling.  

**Acceptance** – Interface does not frame Chambers as a generic productivity product.

## 13.2 Interaction discipline
**Preferred verbs:** create world, inspect object, invoke primitive, propose convergence, validate, seal artifact, burn.  

**Avoid:** open file, save draft, install tool, new document, sync.  

**Acceptance** – Language reinforces world‑first model.

---

# 14. Epic: Performance and determinism

## 14.1 UI performance budget
**Tasks** – Define render budget (<200ms for state update), test with moderate chamber size (≤500 objects).  

**Acceptance** – Level 1 remains usable for research scale.

## 14.2 Deterministic replay from UI traces
**Tasks** – Log adapter calls in replayable format; build trace replayer; compare replayed state to original.  

**Acceptance** – UI session can be reconstructed as substrate actions; divergence is detectable.

---

# 15. Epic: Documentation (security & operation)

## 15.1 Level 1 architecture note
**Deliverables** – Architecture overview, boundaries, data flow, allowed UI‑local state, forbidden persistence.

## 15.2 Research operator guide
**Deliverables** – Chamber creation, object inspection, convergence review, burn, residue review, comparison‑study steps.

## 15.3 Security assumptions note
**Include** – Trusted substrate assumption, UI does not solve lower‑platform compromise, local browser caveats, no ordinary import/export.

---

# 16. Dependencies

**Hard** – Level 0 substrate completed, stable primitive interpreter, burn engine working, residue instrumentation, benchmark baselines.

**Soft** – Small internal researcher cohort, local UI stack, optional model access.

---

# 17. Team roles

- Runtime engineer – adapter boundaries, substrate integration.  
- Frontend engineer – UI shell, state discipline, visual surfaces.  
- Security engineer – client persistence audit, adapter audit, burn correctness.  
- Research lead – lifecycle comprehension studies, chamber‑vs‑baseline evaluation.  
- Systems engineer – benchmark integration, result presentation.

---

# 18. Final Level 1 deliverables

1. Thin local UI shell  
2. Substrate adapter API (stateless forwarder)  
3. Chamber creation surface  
4. World‑state surface  
5. Legal‑action surface  
6. Convergence/finalisation surface  
7. Burn surface  
8. Residue/benchmark surface  
9. Persistence‑audited frontend  
10. Internal usability/research pass  

---

# 19. Final decision gate

Proceed to next level **only if**:

- Users can operate a chamber without CLI.  
- UI does **not** recreate hidden persistence (audit passes).  
- Lifecycle legibility is meaningfully improved.  
- Level 1 still feels like a visual shell over substrate, not an app reabsorbing the architecture.  
- Residue and benchmark outputs remain inspectable and honest.  
- **Cross‑world isolation and post‑burn freeze tests pass.**  

If any of these fail, Level 1 has drifted too far and must be corrected.