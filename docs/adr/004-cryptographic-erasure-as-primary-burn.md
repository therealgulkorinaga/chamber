# ADR-004: Cryptographic Erasure as Primary Burn Primitive

## Status
Accepted

## Context
Physical overwrite of data on modern storage (SSD, flash) is unreliable due to wear-leveling, garbage collection, and over-provisioning. Deleting files does not guarantee data destruction at the physical layer.

## Decision
The primary burn mechanism is **cryptographic erasure**: destroy the world-scoped encryption key `K_w`. All world state is encrypted at rest under `K_w` using AES-256-GCM. When `K_w` is zeroed from memory and its wrapped form deleted, the remaining ciphertext is unrecoverable under the trusted-substrate assumption.

Burn sequence (in order):
1. **Logical burn** — revoke all capabilities, invalidate handles
2. **Cryptographic burn** — zeroize `K_w`, delete wrapped key
3. **Storage cleanup** — delete world-scoped files/records (best effort)
4. **Memory cleanup** — zero in-memory structures
5. **Semantic measurement** — assess residue for benchmarking

Storage and memory cleanup are defense-in-depth, not the primary guarantee.

## Consequences
- Ciphertext may remain on disk after burn — this is acceptable.
- The guarantee is: ciphertext without `K_w` is unrecoverable.
- `K_w` uses `zeroize` crate for secure memory clearing.
- Key material never touches disk unencrypted.
