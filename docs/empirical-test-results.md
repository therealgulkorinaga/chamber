# Empirical Test Results — Chambers Phase 0 + Level 1

**Date:** 2026-04-05
**Build:** Release (Rust 1.94.1, Apple Silicon)
**Tests conducted:** 5
**Tests passed:** 5

---

## Test 1: Audit Tier Destruction on Burn

**Objective:** Verify that world-scoped audit events (Tier 2) are destroyed on burn, and only substrate-scoped events (Tier 1) survive.

**Method:**
1. Create a chamber
2. Perform operations generating Tier 2 events (create objects, advance phases)
3. Count audit events before burn
4. Execute abort burn
5. Count audit events after burn
6. Verify only Tier 1 events remain

**Results:**

| Metric | Before burn | After burn |
|--------|------------|------------|
| Total audit events | 4 | 2 |
| Tier 1 (substrate-scoped) | 1 (WorldCreated) | 2 (WorldCreated + BurnCompleted) |
| Tier 2 (world-scoped) | 3 (PhaseTransition ×3) | 0 |

**Residue report:**
```
substrate_event_count: 2
world_events_surviving: 0
audit_leaks_internals: false
residue_score: 0.0
```

**Verdict:** PASSED. Tier 2 events are destroyed on burn. An observer post-burn learns only that a world existed and was destroyed. No phase transition history, convergence records, or operational metadata survives.

---

## Test 2: Content Residue After Preserve+Burn

**Objective:** Verify that sensitive content from non-preserved objects is unrecoverable after burn, even when one artifact is preserved.

**Method:**
1. Create a chamber with sensitive content:
   - Premise: "TOP SECRET: Project Lazarus budget is $4.2M"
   - Risk: "CONFIDENTIAL: CEO plans to resign in Q3"
   - Alternative: "SECRET OPTION: Acquire CompetitorX for $50M"
   - Decision summary: "Proceed with acquisition" (sealed as artifact)
2. Advance to finalization, seal the decision summary
3. Execute converged-preserving burn
4. Search all surviving data (audit events + vault) for 6 sensitive terms
5. Attempt to access world state, graph, and summary post-burn

**Search terms:** "TOP SECRET", "CONFIDENTIAL", "SECRET OPTION", "Lazarus", "CompetitorX", "resign"

**Results:**

| Search target | "TOP SECRET" | "CONFIDENTIAL" | "SECRET OPTION" | "Lazarus" | "CompetitorX" | "resign" |
|--------------|-------------|----------------|-----------------|-----------|--------------|---------|
| Audit events | Not found | Not found | Not found | Not found | Not found | Not found |
| Vault | Not found | Not found | Not found | Not found | Not found | Not found |

**Post-burn endpoint responses:**
```
GET /api/worlds/{id}          → {"error":"world not found"}
GET /api/worlds/{id}/summary  → {"error":"world not found"}
GET /api/worlds/{id}/graph    → {"error":"world not found"}
```

**Vault contents (sole survivor):**
```json
{"decision": "Proceed with acquisition", "rationale": "Strategic fit"}
```

**Residue report:**
```
state_engine_has_world: false
crypto_key_destroyed: true
substrate_event_count: 2
world_events_surviving: 0
audit_leaks_internals: false
residue_score: 0.0
```

**Verdict:** PASSED. Zero sensitive content found in any surviving data channel. The 3 non-preserved objects (premise, risk, alternative) are gone. Only the sealed decision summary survives in the vault, containing only the decision text — no references to the sensitive premises, risks, or alternatives that led to it.

---

## Test 3: H1 Benchmark — Semantic Residue Comparison

**Objective:** Compare post-destruction metadata residue across Chambers, disposable VM, and constrained microVM baselines with the two-tier audit fix applied.

**Method:**
1. Run canonical decision task (cloud provider selection for HIPAA workloads) 5 times in each condition
2. Measure: recoverable object fraction, recoverable edge fraction, surviving metadata count, reconstruction time

**Results:**

| Condition | Obj Fraction | Edge Fraction | Metadata Count | Reconstruction Time |
|-----------|-------------|---------------|----------------|-------------------|
| **Chambers** | 0.0000 | 0.0000 | **0** | **∞ (infeasible)** |
| Disposable VM | 0.0000 | 0.0000 | 3 | 300s |
| Constrained microVM | 0.0000 | 0.0000 | 4 | 600s |

**Hypothesis H1 (lower recoverable semantic residue):** SUPPORTED

Chambers achieves zero on all residue metrics. Both baselines retain OS-level metadata (process timestamps, directory records, memory usage counters). The differentiator is metadata: Chambers produces 0 surviving metadata entries (Tier 2 events burned), while baselines produce 3-4.

**Hypothesis H3 (fewer reconstructable traces):** SUPPORTED

Chambers reconstruction is infeasible — K_w is destroyed, all world state was encrypted under K_w. Baseline reconstruction requires forensic tools but is achievable in finite time.

**Verdict:** PASSED. H1 and H3 supported. The two-tier audit fix resolves the previously identified metadata residue problem.

---

## Test 4: Cross-World Isolation

**Objective:** Verify that two concurrent chambers share no state, and that burning one does not affect the other.

**Method:**
1. Create World A with distinctive content ("WORLD_A_SECRET_DATA_xyz123")
2. Create World B
3. Query World B's graph — verify World A's content is absent
4. Burn World A
5. Verify World B is still accessible and unaffected

**Results:**

| Check | Expected | Actual |
|-------|----------|--------|
| World B nodes containing World A content | 0 | 0 |
| World B accessible after World A burn | Yes | Yes |
| World B phase after World A burn | Active | Active |

**Verdict:** PASSED. Zero cross-world leakage. Worlds are fully isolated. Burning one world has no effect on another.

---

## Test 5: Post-Burn Endpoint Lockout

**Objective:** Verify that after burn, every data endpoint is inaccessible and only the minimal substrate audit survives.

**Method:**
1. Create a chamber, add an object
2. Burn the chamber
3. Attempt to access every API endpoint for the burned world
4. Verify residue report

**Results:**

| Endpoint | Post-burn response |
|----------|-------------------|
| `GET /api/worlds/{id}` | ERROR (world not found) |
| `GET /api/worlds/{id}/summary` | ERROR (world not found) |
| `GET /api/worlds/{id}/graph` | ERROR (world not found) |
| `GET /api/worlds/{id}/convergence` | ERROR (world not found) |
| `GET /api/worlds/{id}/legal-actions` | ERROR (world not found) |
| `GET /api/worlds/{id}/objects` | ERROR (world not found) |
| `GET /api/worlds/{id}/audit` | 2 events (Tier 1 only) |
| `GET /api/worlds/{id}/residue` | Residue score: 0.0 |

**Residue report:**
```
residue_score: 0.0
state_engine_has_world: false
crypto_key_exists: false
crypto_key_destroyed: true
substrate_event_count: 2
world_events_surviving: 0
audit_leaks_internals: false
```

**Verdict:** PASSED. All 6 data endpoints return errors. The world is completely inaccessible. Only 2 substrate-scoped audit events survive (WorldCreated, BurnCompleted). The crypto key is confirmed destroyed.

---

## Summary

| Test | What it proves | Result |
|------|---------------|--------|
| 1. Audit tier destruction | Tier 2 events burned. 4→2 post-burn. | PASSED |
| 2. Content residue | 6 sensitive terms searched post-burn. Zero found. | PASSED |
| 3. H1 benchmark | Chambers: 0 metadata. VM: 3. MicroVM: 4. | PASSED |
| 4. Cross-world isolation | Zero leakage. Burn of one world doesn't affect another. | PASSED |
| 5. Post-burn lockout | All data endpoints return error. Residue score 0.0. | PASSED |

**Key finding:** The two-tier audit architecture resolves the previously identified H1 failure. World-scoped metadata (phase transitions, convergence records, seal events) is now destroyed on burn. Only 2 substrate-scoped events survive: "a world was created" and "a world was destroyed." This is less metadata than both baselines (3 for disposable VM, 4 for microVM).

**Open item:** H2 (lifecycle comprehension) remains inconclusive — requires user study with the comprehension test harness.
