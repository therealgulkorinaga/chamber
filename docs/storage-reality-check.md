# Storage and Flash Reality Check (Issue 99)

## What Burn Claims

Chambers' burn semantics claim:
1. **Logical revocation**: capabilities invalidated, handles unusable.
2. **Cryptographic erasure**: `K_w` destroyed; ciphertext unrecoverable under trusted-substrate assumption.
3. **Storage cleanup**: world-scoped records deleted from application layer.
4. **Memory cleanup**: in-memory structures zeroed and dropped.

## What Burn Does NOT Guarantee

### Flash/SSD behavior
Modern SSDs use wear-leveling, garbage collection, and over-provisioning. When the application layer deletes a file:
- The flash translation layer (FTL) marks the logical block as unused.
- The physical NAND pages may retain the old data until overwritten by wear-leveling.
- TRIM/discard commands may or may not be honored by the drive firmware.

**Consequence**: After storage cleanup, the ciphertext encrypted under `K_w` may physically persist on the SSD. This is acceptable under the trusted-substrate model because:
- The ciphertext is encrypted with AES-256-GCM under `K_w`.
- `K_w` has been zeroed from memory and its wrapped form deleted.
- Without `K_w`, the ciphertext is computationally infeasible to decrypt.

### Swap and virtual memory
If the OS swaps the substrate process to disk:
- Key material (`K_w`) could be written to swap space.
- `zeroize` clears memory before deallocation, but cannot prevent the OS from paging out before that point.

**Mitigation** (recommended for production, not enforced in Phase 0):
- Use `mlock()` to pin key material in physical RAM.
- Disable swap on the host.
- Use encrypted swap.

### Core dumps
If the process crashes, a core dump may contain:
- `K_w` (if not yet zeroed).
- Plaintext world state (if not yet encrypted at rest).

**Mitigation**: Disable core dumps (`ulimit -c 0` or `prctl(PR_SET_DUMPABLE, 0)`).

## Honest Summary

| Claim | Reality |
|-------|---------|
| Ciphertext unrecoverable | True, IF `K_w` is genuinely destroyed and attacker cannot recover it from swap/dump |
| File deleted | Application-level delete. Physical NAND may retain data. |
| Memory zeroed | `zeroize` crate zeros before drop. OS may have already paged to swap. |
| No residue | Zero recoverable *through the substrate API*. OS-level forensics may find ciphertext or metadata. |

Burn claims remain faithful to actual storage behavior when scoped to the trusted-substrate assumption. We do not imply guaranteed physical overwrite.
