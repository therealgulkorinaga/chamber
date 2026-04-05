# Security Assumptions — Phase 0

## What Phase 0 Claims

Chambers Phase 0 claims that under the assumptions below, a world-first runtime with typed objects, finite primitives, preservation law, and burn semantics reduces semantic residue relative to disposable VM-style baselines.

## Trust Boundary

The **substrate runtime** is the central trust object. Everything inside the substrate process is trusted. Everything outside is untrusted except as noted.

```
┌─────────────────────────────────────────┐
│           TRUSTED SUBSTRATE             │
│  ┌─────────┐ ┌──────────┐ ┌─────────┐  │
│  │ World   │ │ Burn     │ │ Vault   │  │
│  │ Engine  │ │ Engine   │ │         │  │
│  ├─────────┤ ├──────────┤ ├─────────┤  │
│  │ Object  │ │ Crypto   │ │ Audit   │  │
│  │ Engine  │ │ Provider │ │ Log     │  │
│  ├─────────┤ ├──────────┤ ├─────────┤  │
│  │ Policy  │ │ State    │ │ View    │  │
│  │ Engine  │ │ Engine   │ │ Layer   │  │
│  ├─────────┤ ├──────────┤ └─────────┘  │
│  │ Interp. │ │ Capabil. │              │
│  └─────────┘ └──────────┘              │
└─────────────────────────────────────────┘
         ▲ TRUST BOUNDARY ▲
┌─────────────────────────────────────────┐
│         UNTRUSTED / OUT OF SCOPE        │
│  - Operating system kernel              │
│  - Firmware / bootloader                │
│  - Hardware / peripherals               │
│  - Network                              │
│  - Other processes                      │
└─────────────────────────────────────────┘
```

## Explicit Assumptions

| # | Assumption | Rationale |
|---|-----------|-----------|
| A1 | The substrate binary is not compromised | No code integrity verification in Phase 0 |
| A2 | The host OS does not actively subvert the substrate | No kernel-level defenses |
| A3 | No DMA-class attacks during operation | No IOMMU enforcement |
| A4 | No malicious peripherals attached | No device attestation |
| A5 | No firmware compromise | No measured boot |
| A6 | Memory is not externally readable during operation | No cold-boot defense |
| A7 | The substrate's RNG is sound | Depends on OS-provided entropy |

## What Phase 0 Does NOT Solve

- **Lower-platform compromise**: If the OS, firmware, or hardware is compromised, all guarantees collapse.
- **Physical access attacks**: Cold-boot attacks, bus probing, or disk imaging after burn are not defended against.
- **Supply-chain compromise**: The substrate binary, its dependencies, and the Rust toolchain are trusted.
- **Side-channel attacks**: Timing, power analysis, or cache-based side channels are out of scope.
- **Network-based attacks**: Phase 0 operates locally with no network dependency during chamber execution.
- **Multi-user isolation**: Phase 0 is single-user.
- **Hardware attestation**: No TPM, SGX, or remote attestation.
- **General import/export**: No data enters or leaves a world except through the artifact vault.

## Known Risks and Mitigations

| Risk | Mitigation | Status |
|------|-----------|--------|
| Undeclared logging by dependencies | Dependency audit; no verbose logging in release | Documented |
| Over-retentive auditing | Audit layer logs only 6 sparse event types, no payloads | Implemented |
| Hidden OS-level caches | Acknowledged limitation; not mitigable without OS cooperation | Documented |
| Vault metadata as residue source | Minimal provenance; no world internals in vault | Implemented |
| Primitive interpreter bypass | Interpreter is sole public mutation path; no state-mutating helpers | Structural |
| Model context leakage | Orchestrator context is world-scoped; cleared on burn | Implemented |
| Key material in swap | `zeroize` crate used; `mlock` recommended for production | Partial |
| Update-path compromise | No auto-update in Phase 0 | N/A |

## Collapse Condition

If any assumption A1–A7 is violated, Chambers collapses into **managed theater** — the appearance of isolation and burn without the substance. This is stated openly because honest threat modeling requires it.
