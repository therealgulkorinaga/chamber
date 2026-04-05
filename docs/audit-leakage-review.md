# Audit Leakage Review — Orchestration and Vault

## Orchestration Layer (Issue 72)

### Architecture
The `SymbolicOrchestrator` is stateless — it holds only a reference to the `Runtime` and a `Principal`. It has:

- No internal object cache
- No scratch buffers
- No logging of its own
- No persistent state between runs

All orchestrator actions route through `TransitionRequest` → `Interpreter` → `OperationEngine`. The orchestrator cannot bypass the interpreter's 5-check validation pipeline.

### Verified: No hidden persistence

| Check | Result |
|-------|--------|
| Orchestrator struct fields after run | `&Runtime` reference + `Principal` only |
| Internal buffers | None — local variables dropped at function exit |
| Logging | Orchestrator emits no log statements |
| Error paths | Errors propagated as `OrchestratorError`, no payload data in error messages |
| Stack traces | Standard Rust — no payload content in debug output |
| Debug mode | No `#[cfg(debug_assertions)]` blocks that store extra state |

### Remaining risks

- **Stack residue**: Rust drops local variables when they go out of scope, but does not zero memory. For Phase 0 (research substrate), this is acceptable under the trusted-substrate assumption. Production would need `zeroize` for stack-allocated buffers.

## Vault Layer (Issue 62)

### What the vault stores

Each sealed artifact contains:
- `artifact_id` — UUID
- `source_world_id` — UUID (identifies which world, but not its contents)
- `artifact_class` — string (e.g., "decision_summary")
- `payload` — the decision summary itself (this is the intended survivor)
- `sealed_at` — timestamp
- `provenance_metadata` — grammar_id, objective_summary, world_created_at, world_terminated_at
- `vault_policy_class` — string

### What the vault does NOT store

- Object IDs of non-preserved objects
- Link structure (graph edges)
- Intermediate reasoning (premises, alternatives, risks, upsides)
- Challenge records
- Capability tokens
- Object payloads other than the sealed artifact

### Cross-world inference from vault

An observer with vault access can infer:
- That a world existed (from `source_world_id`)
- What grammar was used (from `grammar_id`)
- When the world lived (from timestamps)
- The final decision (from `payload`)

An observer **cannot** infer:
- What alternatives were considered
- What risks were identified
- What premises led to the decision
- How many objects existed
- The reasoning chain

### Verdict

The vault's residue boundary is well-defined. The accepted residue (that a world existed and produced a decision) is inherent to having an artifact vault at all. The unaccepted residue (world internals) is effectively excluded.
