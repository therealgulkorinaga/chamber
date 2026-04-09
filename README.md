# Chambers

**Sealed Ephemeral Worlds for Private Cognition**

A runtime where thinking rooms self-destruct. You enter a chamber, do your reasoning, seal one decision, and burn everything else. The machine knows a chamber existed. It does not know what was inside.

> [Read the paper](Chambers%20Position%20Paper.pdf)

---

## What is this

Chambers inverts how computing handles privacy. Instead of making persistent environments safer, it rejects persistence itself. The bounded world — not the application — is the primary unit of computation and destruction.

A Chamber is defined by:
- **Typed objects** (facts, constraints, risks, alternatives, decisions)
- **A closed primitive algebra** (9 operations — no arbitrary code execution)
- **Preservation law** (only one object class can survive burn — declared in advance)
- **Cryptographic burn** (destroy the key, the ciphertext becomes meaningless)
- **Encrypted memory** (objects are ciphertext in RAM — plaintext exists for microseconds)

## Architecture

```
Native App (fullscreen, no browser)
  All system shortcuts blocked, custom pointer, clipboard isolated
  Camera/mic/bluetooth/USB/sensors/network — all severed
  ┌─────────────────────────────────────────────────┐
  │  Runtime                                         │
  │  World Engine → Object Engine → State Engine     │
  │  Interpreter (5-check validation pipeline)       │
  │  Policy Engine ← Grammar (Decision Chamber)     │
  │  Capability System (epoch-scoped, narrowing)     │
  │  Burn Engine (6-layer destruction)               │
  │  Artifact Vault (sole cross-world channel)       │
  │  Audit (2-tier: existence events survive,        │
  │         world events burned)                     │
  ├─────────────────────────────────────────────────┤
  │  Encrypted Memory Pool                           │
  │  AES-256-GCM under K_w per world                │
  │  Guard buffer: 8KB, mlock'd, zeroed per use     │
  │  Scoped access: with_object(id, |pt| { ... })   │
  └─────────────────────────────────────────────────┘
```

## The 6-Layer Burn

1. **Logical** — capabilities revoked, handles invalidated
2. **Cryptographic** — K_w zeroed from memory, wrapped key deleted
3. **Storage** — world-scoped records removed
4. **Memory** — in-memory structures zeroed
5. **Audit** — world-scoped events destroyed (only "existed" + "destroyed" survive)
6. **Semantic** — residue measurement confirms nothing recoverable

## What's built

| Layer | What | Status |
|-------|------|--------|
| **Phase 0** | Substrate runtime, 9 primitives, burn engine, benchmark harness | Complete |
| **Level 1** | Native fullscreen app (tao + wry), fullscreen takeover, system isolation | Complete |
| **Phase 2** | mlock, core dump disable, ptrace deny, encrypted memory pool, chamber clipboard, WebKit incognito | Complete |
| **Phase 3** | Secure Enclave, App Sandbox, Hypervisor boot, chamber-born LLM | Planned |

### Tests

44 tests, 0 failures. Including:
- Encrypted store roundtrip + wrong-key rejection
- Full preserve+burn E2E with content leak search
- Cross-world crypto isolation
- Post-burn endpoint lockout
- Two-tier audit event destruction
- Determinism (replay produces same state)
- Real baseline benchmarks (filesystem + Docker)

### Benchmark results (real baselines)

| Condition | Content Residue | Metadata | Reconstruction |
|-----------|----------------|----------|----------------|
| **Chambers** | **Zero** | **0 undeclared** | **Infeasible** |
| Disposable VM | Zero | 2 entries | Feasible |
| Docker microVM | Zero | 5.3 entries | Feasible |

## Quick start

### Native app (fullscreen, isolated)

```bash
cargo run --release -p chambers-app
```

Takes over the screen. Click **Load Demo** to enter a pre-populated chamber. **Burn** to destroy. **Esc** to exit.

### CLI

```bash
cargo run -p chambers-cli
```

### HTTP adapter (for development)

```bash
cargo run -p chambers-adapter
# Open http://127.0.0.1:3000
```

### Benchmark

```bash
cargo run --release -p chambers-benchmark -- 5
```

### Tests

```bash
cargo test
```

## How it works

1. **Open a chamber** — choose a grammar (Decision Chamber), enter your question
2. **Add entries** — facts, constraints, alternatives, risks, upsides
3. **Connect them** — link risks to alternatives, supports to premises
4. **Advance stages** — exploring → reviewing → finalizing
5. **Seal your decision** — the one thing that survives (requires human authorization)
6. **Burn** — chamber destroyed, key zeroed, app exits, desktop returns
7. **Archive** — your sealed decision lives in the vault. Everything else is gone.

## Project structure

```
crates/
  chambers-types/       Core data model
  chambers-crypto/      Key hierarchy, AES-256-GCM, mlock, guard buffer
  chambers-world/       World engine, lifecycle state machine
  chambers-object/      Schema validation, payload enforcement
  chambers-policy/      Grammar loading, preservation law
  chambers-capability/  Epoch-scoped tokens, narrowing
  chambers-state/       Encrypted object/link store
  chambers-operation/   9 primitive operations
  chambers-interpreter/ 5-check validation pipeline
  chambers-burn/        6-layer destruction
  chambers-vault/       Artifact vault (sole cross-world channel)
  chambers-audit/       2-tier audit (substrate + world events)
  chambers-view/        Read-only projections
  chambers-orchestrator/ Rules-based symbolic planner
  chambers-runtime/     Top-level wiring
  chambers-cli/         Interactive CLI
  chambers-adapter/     HTTP API (axum)
  chambers-app/         Native fullscreen app (tao + wry)
  chambers-benchmark/   Real baselines, H1-H3 testing
docs/
  PRD_Chambers_Isolation_Phases.md
  Phase2_IssueList.md
  competitive-analysis.md
  empirical-test-results.md
  phase2-empirical-results.md
  final-validation-report.md
  real-baseline-results.md
  security-assumptions.md
  formal-mapping.md
  adr/                 Architecture Decision Records
grammars/
  decision_chamber.json
ui/
  index.html           Browser-based UI (development only)
```

## Security model

**Inbound**: the chamber uses the machine (CPU, memory, display, keyboard). Necessary.

**Outbound**: nothing escapes through application-layer channels.

| Channel | Status |
|---------|--------|
| Clipboard | Isolated (chamber-scoped, zeroed on burn) |
| Network | Blocked (localhost only) |
| File system | Blocked |
| Camera/mic/bluetooth/USB | Blocked |
| Storage APIs | Blocked (WebKit incognito) |
| System shortcuts | Blocked |
| Core dumps | Disabled |
| Debugger attachment | Denied (ptrace) |
| Memory | Encrypted (AES-256-GCM, plaintext in guard buffer only) |

**Not claimed**: OS-level process visibility, framebuffer capture, keystroke interception, DMA access to K_w. See [security assumptions](docs/security-assumptions.md).

## Paper

The position paper is included: [Chambers Position Paper.pdf](Chambers%20Position%20Paper.pdf)

## License

MIT
