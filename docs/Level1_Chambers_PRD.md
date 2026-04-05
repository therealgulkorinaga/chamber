# PRD — Chambers Level 1
## Thin Research UI on Top of the Substrate (Hardened)

## 1. Document purpose

This PRD defines **Level 1** of Chambers: the first visual research layer on top of the Level 0 CLI substrate.

Level 1 is **not** a polished product, broad secure‑computing environment, or consumer interface. Its purpose is to make the Chamber model **visible, operable, and testable by humans** while **strictly preserving** the paper’s architectural constraints:

- world‑first computation
- persistence‑law‑first design
- typed objects with constrained payloads
- finite primitive algebra
- preservation law
- burn‑first lifecycle
- trusted‑substrate assumption

Level 1 exists to answer:

> **Can humans understand, operate, and reason about Chamber lifecycle and world‑state more clearly than they can with disposable VM workflows, without reintroducing application‑centric persistence or hidden state?**

## 2. Product thesis for Level 1

Level 1 is a **thin visual shell** over the substrate.

- It is **not** a second runtime.
- It is **not** a client‑side reinvention of world logic.
- It is **not** a rich application layer.

Its only job: let a user create a Chamber, inspect world‑state, see legal actions, review convergence, preserve the allowed artifact, burn the world, and inspect post‑burn residue.

**Core principle:** All meaningful state and all legal actions remain in the substrate; the UI only **renders** and **invokes** (via the adapter).

## 3. Goals

Level 1 must produce six things:

1. **Visual world operation** – operate one Chamber grammar without CLI.
2. **Lifecycle legibility** – show current phase, epoch, convergence proposal, preservation legality, burn status.
3. **Legal‑action visibility** – display only operations legal in current world‑state and phase.
4. **Finalization visibility** – expose the three‑part finalization model (propose → validate → authorise seal).
5. **Burn visibility** – show burn as an explicit event with stages and outcome.
6. **Residue inspection** – expose residue metrics and baseline comparisons for research.

## 4. Non‑goals (strictly enforced)

Level 1 will **not** include:

- Ordinary import / export of any kind.
- Collaboration or multi‑user support.
- Rich editing environment (no freeform document editing).
- Generic app execution or plugin architecture.
- Multiple chamber families (only Decision Chamber).
- Enterprise governance or fleet management.
- Polished consumer UX or “AI assistant” chat behaviour.
- Any client‑side persistence of chamber internals (beyond transient UI state).

## 5. Primary users

- Research engineer (builds and evaluates the UI shell and adapter).
- Security reviewer (audits for hidden state, bypasses, and truthful burn semantics).
- Systems researcher / design partner (evaluates lifecycle comprehension and world mental model).

These are early, serious users — not consumers.

## 6. Core design principles (hardened)

### 6.1 UI is subordinate to substrate law
- UI may render only what the substrate exposes.
- UI may invoke only legal substrate actions (via the adapter).
- UI may **never** mutate world‑state directly.

### 6.2 No hidden client‑side persistence
- No shadow object store, no local chamber history DB, no unsanctioned artifact cache.
- No browser‑side persistence of chamber internals by default (localStorage, IndexedDB, service workers, etc.) unless explicitly permitted and audited.

### 6.3 Legibility over beauty
- Clarity matters more than polish. The interface must feel sparse, explicit, and inspectable.

### 6.4 World‑first, not app‑first
- The UI presents a Chamber as a bounded world with lifecycle law, not as “just another tool”.

### 6.5 Minimalism
- Every added feature risks reintroducing ordinary computing habits. The UI must stay thin.

## 7. Scope of build

### 7.1 Required screens / surfaces

#### 7.1.1 Chamber creation surface
- Select grammar (only Decision Chamber enabled).
- Enter objective.
- Create new world.
- Show world ID and initial state.

#### 7.1.2 World‑state surface (main interface)
Must show:
- world ID, objective, lifecycle phase, epoch.
- Object list with types, lifecycle class, preservable flag, modified time.
- Object relationships (links).
- Current candidate artifact (if any).
- Unresolved contradictions / constraints.
- Allowed next operations (legal actions).

#### 7.1.3 Action surface
User can invoke only legal operations. Examples:
- create premise, link objects, challenge object, generate alternative, rank set, synthesise set, condense object, propose convergence, seal artifact, trigger burn.

**The UI must never show a generic toolbox that exceeds current policy and capability state.**

#### 7.1.4 Convergence and finalisation surface
Must show:
- Whether convergence has been proposed, passed validation, or failed (with reason).
- Whether preservation is legal.
- Whether human/policy authority must approve sealing.
- Available termination options.

#### 7.1.5 Burn surface
Must show burn stages (logical revocation, cryptographic burn, storage cleanup, memory cleanup) and final outcome (world destroyed, artifact preserved or not).

#### 7.1.6 Residue inspection surface
Must show at minimum:
- Surviving artifact classes.
- Retained metadata summary.
- Residue metrics (recoverable object fraction, edge fraction, etc.).
- Benchmark comparison outputs (if available).

### 7.2 Deferred (not in Level 1)
- Graph visualisation with drag‑and‑drop mutation (would risk becoming a shadow editor).
- Lifecycle timeline (helpful but can be built later without breaking security).
- Epoch history view (low priority, no security impact).

## 8. Technical architecture (hardened)

### 8.1 Layering

**Layer A – Substrate** (Level 0)  
World engine, object engine, interpreter, policy engine, capability system, state engine, artifact vault, burn engine, sparse audit layer.

**Layer B – API Adapter** (stateless, no policy enforcement)  
A thin service boundary that **forwards** requests to the substrate and returns responses.  
- **Must not** perform its own policy or capability checks.  
- **Must not** cache world state beyond a single request.  
- **Must** reject any request that does not include a valid world ID and (if required) capability token.  

**Layer C – UI Shell** (minimal, ephemeral state only)  
Renders substrate state and invokes substrate actions via the adapter.

### 8.2 Architectural constraints (hardened)

#### 8.2.1 No client‑side state authority
The client is **never** the source of truth for:
- object graph, lifecycle phase, capability state, artifact status, residue status.

#### 8.2.2 No direct mutation
Every UI action must route through the same interpreter / substrate API path that the CLI uses.  
The adapter **must not** offer a “bypass” primitive.

#### 8.2.3 Replayability
UI actions must be serialisable into replayable command traces for security auditing and residue analysis.

#### 8.2.4 Deterministic rendering
Given identical substrate state, the rendered view must be identical (no randomised UI elements that could hide state).

## 9. Functional requirements (hardened)

### FR1 – Create Chamber
User can create a Decision Chamber from UI by entering an objective.  
**Acceptance:** world created, valid world ID returned, initial state visible.

### FR2 – Inspect world‑state
User can inspect objective, lifecycle phase, epoch, object inventory, details, links, candidate artifact, unresolved contradictions.  
**Acceptance:** information reflects substrate state, not UI‑local caches; refreshes correctly after operations.

### FR3 – View legal actions only
UI shows only operations currently allowed.  
**Acceptance:** illegal actions are absent or clearly disabled; action list updates after epoch changes and lifecycle transitions.

### FR4 – Invoke operations
User can invoke legal primitive operations.  
**Acceptance:** every action routes through substrate API; invalid actions fail with interpretable errors; no direct client‑side mutation exists.

### FR5 – Convergence visibility
UI exposes convergence proposal, validation, and authorisation state.  
**Acceptance:** user can distinguish “proposed” from “validated”; invalid convergence states are explained; preservation authority is explicit.

### FR6 – Burn visibility
UI exposes burn progress and end state.  
**Acceptance:** burn is visibly distinct from normal closure; preserved artifact outcome is clear; destroyed state is explicit.

### FR7 – Residue visibility
UI can display post‑burn residue metrics and benchmark outputs.  
**Acceptance:** residue panel can be used without CLI; benchmark outputs are attributable to the specific run and are read‑only.

### FR8 – No cross‑world leakage
UI must never display objects or state from a different world ID.  
**Acceptance:** automated test attempts to load world A’s objects while viewing world B – must fail.

### FR9 – Post‑burn world freeze
After burn, the UI must not allow any action that would attempt to resume or mutate the destroyed world.  
**Acceptance:** any action on a burned world returns `WorldDestroyed` error.

## 10. Non‑functional requirements (hardened)

### NFR1 – Thinness
The UI must remain a thin shell; no parallel logic engine.

### NFR2 – Local‑only operation
Normal Level 1 operation must not require remote services (except optional benchmark data export).

### NFR3 – No hidden client persistence
Audit of localStorage, IndexedDB, service workers, and crash recovery must confirm no chamber internals persist by default.

### NFR4 – Inspectability
A researcher can understand how each displayed field maps to substrate state.

### NFR5 – Debuggability without leakage
Errors must be explicit enough for research but must not leak object payloads or world internals by default.

### NFR6 – Adapter idempotency
Replaying the same adapter request twice must produce the same substrate transition (no accidental double‑burn).

## 11. Level 1 UX posture

The interface should feel like:
- a world console
- a chamber debugger
- a lifecycle instrument panel
- a research control surface

**Not** a polished consumer app.

## 12. AI / planner requirements for Level 1

Same as Level 0. Preferred order: symbolic planner → small model → LLM only if necessary.  
If an LLM is used:
- World‑scoped inference context must be treated as world‑state and burned.
- Model weights remain substrate‑scoped.
- UI must not create a second independent conversational memory outside the chamber.

## 13. Data model impacts

Level 1 adds only **presentation‑layer ephemeral state**:

**Allowed UI‑local ephemeral state:**
- current selection
- viewport state
- expanded/collapsed panel state
- recent fetch result (in memory only)
- request status (loading, error)

**Forbidden UI‑local persistence:**
- separate object store
- shadow lifecycle tracker
- extra draft buffer of chamber internals
- cached artifact copies beyond what policy explicitly allows
- hidden event logs of world internals

## 14. Research goals for Level 1

- **Lifecycle comprehension** – can users explain phase, convergence, survival, burn?
- **Preservation comprehension** – do users correctly predict what survives?
- **World mental model** – do users understand the system as a bounded world with explicit law, not just another app?

## 15. Milestones

| Milestone | Deliverables | Exit criteria |
|-----------|--------------|----------------|
| L1.1 – API adapter | Create world, fetch state, legal actions, invoke, convergence, burn, residue endpoints | All substrate actions callable without CLI; adapter does no policy checks |
| L1.2 – Minimal shell | UI shell, creation screen, world summary, object list, action panel | One Decision Chamber can be created and operated without CLI |
| L1.3 – Finalisation & burn UI | Convergence review, finalisation, preserve/burn pathway, burn progress | Users can distinguish proposal, validation, authorisation, termination |
| L1.4 – Residue & benchmark | Residue panel, post‑burn summary, baseline comparison view | Researcher can inspect residue outcomes through UI |
| L1.5 – Usability research pass | Test script, comprehension tasks, comparison protocol | Level 1 can support human‑facing evaluation of H2 |

## 16. Acceptance criteria for Level 1 (hardened)

Level 1 is complete **only if**:

1. A user can create and operate a Decision Chamber without CLI.
2. Every meaningful action still routes through substrate law (adapter audit passes).
3. Lifecycle phases are clearly visible and accurate.
4. Convergence and preservation state are legible.
5. Burn is visible, explicit, and cannot be bypassed.
6. Residue information is inspectable through the UI.
7. **No client‑side persistence of chamber internals** is found (audit passes).
8. **No cross‑world leakage** test passes.
9. **Post‑burn freeze** test passes.
10. The UI never presents a “generic app” mental model (research pass confirms).

## 17. Risks & mitigations

| Risk | Mitigation |
|------|-------------|
| UI drift into product theatre | Strict non‑goals; weekly architecture review |
| Hidden client persistence | Automated audit of storage APIs; manual review |
| Parallel logic engine | Adapter does **no** policy checks; all law in substrate |
| Misleading mental model | UX research pass with specific falsification questions |
| Overbuilding (graph drag‑and‑drop) | Deferred to Level 2+; not allowed in Level 1 |

## 18. Deferred to later levels

- Ordinary import / export
- Multiple chamber families
- Collaboration
- Enterprise governance
- Polished ritual UX
- Graph editing

## 19. One‑line outcome

If Level 1 works, you have a **human‑operable chamber runtime** that makes the architecture visible enough to test lifecycle legibility and semantic‑residue claims without collapsing back into ordinary application software.