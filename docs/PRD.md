# PRD — Chambers Phase 0
## Research Substrate and Benchmark Harness

## 1. Document purpose

This PRD defines **Phase 0** of Chambers: the minimum research-grade substrate and benchmark harness required to test whether the Chamber model is materially different from disposable VM and constrained microVM baselines on the paper's stated axes: semantic residue minimization, lifecycle legibility, preservation narrowness, and legal execution surface.

Chambers is explicitly scoped under a **trusted-substrate assumption** and does not claim protection against lower-platform compromise, firmware compromise, DMA-class attacks, or malicious peripherals.

---

## 2. Product thesis for Phase 0

Phase 0 is **not** a user product. It is a **research substrate** and **measurement harness**.

Its job is to answer one question:

**Does a world-first runtime with typed objects, finite primitives, preservation law, and burn semantics reduce semantic residue relative to disposable VM-style baselines under bounded assumptions?**

Semantic residue is defined in the paper as recoverable interpretable information about non-preserved world-state after termination, beyond what the preservation law permits.

---

## 3. Goals

Phase 0 must produce five things.

### 3.1 Working substrate runtime

A working **substrate runtime** implementing the paper's core architecture:

- world engine
- object engine
- operation engine
- policy engine
- capability system
- state engine
- artifact vault
- burn engine
- sparse audit layer

### 3.2 Closed primitive enforcement mechanism

A **closed primitive enforcement mechanism** such that all world evolution occurs through a finite primitive algebra validated by a state-machine runtime for:

- type compatibility
- capability possession
- lifecycle legality
- preservation-law legality
- world-scope correctness

### 3.3 One reference chamber grammar

A **single reference chamber grammar**, using the paper's Decision Chamber as the first implementation target.

### 3.4 Burn implementation

A **burn implementation** that follows the paper's five-layer destruction model:

- logical burn
- cryptographic burn
- storage cleanup
- memory cleanup
- semantic burn

with cryptographic erasure of world-scoped keys as the primary burn primitive.

### 3.5 Benchmark harness

A **benchmark harness** that compares Chambers against at least:

- one disposable VM baseline
- one constrained microVM-style baseline

using the paper's falsifiable evaluation agenda and hypotheses H1–H3.

---

## 4. Non-goals

Phase 0 will **not** include:

- ordinary import/export
- collaboration
- multi-user features
- enterprise policy packs
- hardware attestation beyond what is minimally required for the chosen test environment
- generalized app execution
- plugin system
- broad chamber grammar library
- consumer UX polish

This is deliberate. The paper is narrow and implementation-light by design, and the first step is to validate the substrate, not to build a broad secure-computing product.

---

## 5. Primary users

Phase 0 serves three internal user types.

### 5.1 Research engineer
Builds and evaluates the substrate and benchmark harness.

### 5.2 Security reviewer
Inspects whether burn semantics, state partitioning, and capability narrowing actually behave as claimed.

### 5.3 Systems researcher / design partner
Runs bounded tasks inside the first chamber to assess lifecycle comprehension and residue outcomes.

These are not public end users. Phase 0 is pre-product.

---

## 6. Core product principles

Phase 0 must obey the paper's architecture.

### 6.1 World-first
The Chamber is the primary semantic unit, not the app or guest machine.

### 6.2 Finite primitive algebra
No undeclared behavior. No dynamic primitive creation.

### 6.3 Typed objects with constrained payloads
No opaque binary payloads, no external blobs, no Base64-style smuggling through text fields.

### 6.4 Preservation law is formal
Only approved artifact classes may survive.

### 6.5 Burn is multi-layered
Not session exit, not VM deletion, but destruction across logical, cryptographic, storage, memory, and semantic layers.

### 6.6 Trusted-substrate humility
The substrate is the central trust object. If it is compromised, Chambers collapses into managed theater.

---

## 7. Scope of build

### 7.1 Runtime scope

The substrate runtime must include the following components.

#### 7.1.1 World engine
Responsible for:

- world creation
- world ID allocation
- namespace isolation
- lifecycle phase tracking
- termination dispatch

**Requirement:** world IDs must never be reused after burn.

#### 7.1.2 Object engine
Responsible for:

- object creation
- schema validation
- lifecycle-class tagging
- transform-set binding
- preservable flag assignment

**Minimum object fields:**

- type
- world_id
- lifecycle_class
- payload
- transform_set
- preservable_flag
- capability_requirements

#### 7.1.3 Operation engine
Responsible for executing the finite primitive algebra.

**Initial primitive set:**

- create_object
- link_objects
- challenge_object
- generate_alternative
- rank_set
- synthesize_set
- condense_object
- seal_artifact
- trigger_burn

#### 7.1.4 Policy engine
Responsible for:

- permitted object classes
- permitted primitive calls
- permitted views
- permitted state transitions
- preservation-law checks
- termination-law checks

#### 7.1.5 Capability system
Responsible for:

- world-scoped capability tokens
- epoch-scoped narrowing
- mid-world revocation through epoch advancement

#### 7.1.6 State engine
Responsible for:

- live object graph
- lifecycle phase
- capability graph
- temporary render state
- convergence review state

#### 7.1.7 View layer
Phase 0 only needs the minimum set:

- conversation view
- graph view
- summary view
- burn view

#### 7.1.8 Artifact vault
Responsible for:

- storing approved survivors only
- minimizing provenance metadata
- acting as the sole authorized cross-world channel

#### 7.1.9 Audit layer
Responsible for sparse lifecycle logging only:

- world created
- convergence proposed
- validation result
- artifact sealed or not
- burn completed

Must not store world internals by default.

#### 7.1.10 Burn engine
Must implement:

- logical burn
- cryptographic burn
- storage cleanup
- memory cleanup
- semantic non-reconstructability objective

---

### 7.2 Chamber scope

Phase 0 supports **one** chamber grammar only.

#### 7.2.1 Decision Chamber
Use the paper's reference chamber grammar:

- objective class: `decision_objective`
- object classes:
  - `premise`
  - `support_statement`
  - `constraint`
  - `risk`
  - `upside`
  - `contradiction`
  - `alternative`
  - `recommendation`
  - `decision_summary`
- allowed views:
  - conversation
  - graph
  - summary
  - burn
- preservation law: only `decision_summary` may survive
- termination law:
  - preserve one `decision_summary` and burn all else
  - or burn all with no survivor

---

## 8. Functional requirements

### 8.1 World creation

The system must:

- instantiate a world from a selected grammar
- create a fresh world ID
- create a fresh world-scoped key `K_w`
- bind the world to a lifecycle controller
- initialize a scoped capability graph

**Acceptance criteria:**
- no object from a prior world is addressable in the new world namespace

---

### 8.2 Primitive enforcement

All world evolution must occur through transition requests to the closed interpreter.

The runtime must reject any request that fails:

- type compatibility
- capability possession
- lifecycle legality
- preservation-law legality
- world-scope correctness

**Acceptance criteria:**
- undeclared operations cannot execute
- invalid cross-world object references fail hard
- preservation of non-preservable classes is blocked

---

### 8.3 Capability narrowing

Capabilities must be:

- world-scoped
- epoch-scoped
- invalidated on epoch advance unless reissued under tighter policy

**Required phases:**
- exploratory epoch
- convergence epoch
- finalization epoch

**Acceptance criteria:**
- operations legal in exploratory phase are not automatically legal in finalization phase

---

### 8.4 Convergence and finalization

Phase 0 must implement the paper's three-part finalization model:

1. finalizer proposes convergence
2. substrate validates convergence against grammar and preservation law
3. policy or human authority authorizes sealing

**Termination modes:**
- converged-preserving termination
- converged-total-burn termination
- abort burn

**Acceptance criteria:**
- the model alone cannot unilaterally preserve an artifact
- abort burn leaves no artifact

---

### 8.5 Burn

Phase 0 burn must perform, in order:

1. logical revocation
2. cryptographic erasure of `K_w`
3. invalidation of unwrap path from `K_s` to `K_w`
4. storage cleanup
5. memory cleanup
6. semantic residue measurement pass

**Acceptance criteria:**
- retained ciphertext remains but is unrecoverable under trusted-substrate assumptions after `K_w` destruction
- temporary world graph is no longer traversable
- in-memory world context is dropped or zeroed

---

## 9. Non-functional requirements

### 9.1 Determinism of substrate law
Given the same valid transition request and same world-state, the interpreter should produce the same substrate-level result.

### 9.2 Minimal TCB growth
No unnecessary services in Phase 0. The substrate is the central trust object and must stay small.

### 9.3 Observability without over-retention
The system must emit enough signals for debugging and residue evaluation without storing world internals by default.

### 9.4 Local operation
Phase 0 should run locally with no dependency on network services during normal chamber execution.

---

## 10. AI / planner requirements

The paper is clear that the architecture remains meaningful even if the orchestration layer is replaced by a symbolic planner or smaller model, and that orchestration is not the primary semantic unit. Phase 0 should therefore default to the **simplest orchestration path** that can drive the chamber.

**Recommended implementation order:**
1. symbolic planner or rules-based orchestrator
2. small model if needed
3. LLM only if necessary

If an LLM is used:

- world-scoped inference context must be treated as world-state
- model weights belong to the substrate and are not burned per-world
- no hidden scratch state outside world law is allowed

---

## 11. Data model

### 11.1 World
**Fields:**
- world_id
- grammar_id
- objective
- lifecycle_phase
- epoch
- world_key_ref
- artifact_key_ref
- created_at
- terminated_at
- termination_mode

### 11.2 Object
**Fields:**
- object_id
- world_id
- type
- lifecycle_class
- payload
- transform_set
- preservable_flag
- created_at
- last_modified_at

### 11.3 Capability token
**Fields:**
- token_id
- world_id
- epoch
- principal
- permitted_operation
- permitted_object_types
- issued_at
- expires_at
- revoked_flag

### 11.4 Artifact
**Fields:**
- artifact_id
- source_world_id
- artifact_class
- sealed_at
- provenance_metadata_min
- vault_policy_class

---

## 12. Benchmark harness

Phase 0 must include benchmark tooling to compare Chambers with:

- disposable VM baseline
- constrained microVM baseline

The evaluation agenda from the paper gives three central hypotheses:

- **H1:** lower recoverable semantic residue than disposable VM baseline
- **H2:** better user prediction of what survives and burns
- **H3:** fewer reconstructable intermediate reasoning traces

### 12.1 Metrics

Measure at least:

- recoverable object fraction of non-preserved world-state
- recoverable graph-edge fraction
- surviving metadata count
- time required to reconstruct intermediate reasoning traces
- user prediction accuracy about what survives and what burns

### 12.2 Falsifiers

Phase 0 is considered to have failed its research thesis if:

- Chambers leaves comparable semantic residue to a disposable VM baseline
- users do not understand Chamber lifecycle more accurately than the baseline
- substrate-side retention recreates equivalent intermediate traces

---

## 13. Security assumptions and risks

### 13.1 Assumptions

- trusted substrate
- no lower-platform compromise during test runs
- no malicious peripherals
- no DMA-class boot attacks
- no substrate supply-chain compromise during Phase 0 evaluation

### 13.2 Known risks

- undeclared logging
- over-retentive auditing
- hidden caches
- vault leakage
- primitive-interpreter bypass
- model-context leakage
- update-path compromise

---

## 14. Milestones

### Milestone 1 — substrate skeleton
**Deliver:**
- world engine
- object engine
- policy engine
- interpreter shell
- lifecycle controller

**Exit criteria:**
- worlds can be created and destroyed
- object schemas validate correctly

---

### Milestone 2 — primitive enforcement
**Deliver:**
- finite primitive algebra
- request validation pipeline
- capability checks
- epoch advancement

**Exit criteria:**
- invalid transitions blocked
- no undeclared operations possible through public runtime path

---

### Milestone 3 — burn engine
**Deliver:**
- key hierarchy
- `K_w` generation
- burn sequence
- storage cleanup hooks
- memory cleanup hooks

**Exit criteria:**
- burn runs end-to-end
- post-burn world cannot be reopened

---

### Milestone 4 — Decision Chamber
**Deliver:**
- one grammar
- one convergence checker
- one summary artifact path
- abort burn path

**Exit criteria:**
- one full chamber run from creation to preserve+burn
- one full chamber run from creation to abort burn

---

### Milestone 5 — benchmark harness
**Deliver:**
- disposable VM baseline
- constrained microVM baseline
- residue instrumentation
- lifecycle-comprehension test rig

**Exit criteria:**
- H1–H3 can be tested using reproducible runs

---

## 15. Acceptance criteria for Phase 0

Phase 0 is complete when:

- the substrate runtime exists and enforces chamber law
- the Decision Chamber runs end-to-end
- burn semantics work at the implementation level claimed
- one preserved artifact class is supported
- no ordinary import/export exists beyond artifact survival into the vault
- the benchmark harness can compare Chambers against at least one disposable VM and one constrained microVM baseline
- H1–H3 are testable with collected data

---

## 16. Open questions deferred to later phases

Not Phase 0:

- general import/export
- multi-chamber orchestration
- institutional vault governance
- hardware attestation and whitelist productization
- enterprise fleet management
- broader grammar libraries
- public release polish

---

## 17. One-line outcome

If Phase 0 works, you will have proven whether Chambers is a real runtime category or just a well-written idea. That is the right first product objective under the paper's own logic.
