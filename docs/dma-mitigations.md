# DMA Mitigations — Phase 2

## Apple Silicon IOMMU (existing protection)

Apple Silicon Macs (M1/M2/M3/M4) have an IOMMU built into the SoC. macOS uses it to restrict which devices can perform Direct Memory Access to which memory regions.

**What this blocks:**
- Thunderbolt/USB-C DMA attacks (PCILeech-style) that worked on older Intel Macs
- Rogue peripherals reading arbitrary host memory via DMA
- FireWire DMA attacks (legacy, but architecturally similar)

**What this does NOT block:**
- Compromised internal controllers with pre-authorized DMA access (e.g., rogue GPU firmware, compromised NVMe controller)
- Attacks originating from within the SoC itself

**Reference:** Apple Platform Security Guide — "Direct Memory Access protections" section.

## Application-layer mitigations (Phase 2)

### Guard buffer

All plaintext decryption happens in a single 8KB page-aligned buffer:
- Allocated via `mmap(MAP_PRIVATE | MAP_ANONYMOUS)`
- Immediately `mlock()`'d — pinned in physical RAM
- `madvise(MADV_DONTDUMP)` on Linux (macOS uses `setrlimit(RLIMIT_CORE, 0)` instead)
- Zeroed via `zeroize` after every use
- On process exit or burn: zero → munlock → munmap

**DMA exposure:** at any given instant, at most 8KB of plaintext exists (one object being decrypted). The rest of RAM contains only ciphertext.

### Encrypted object store

All objects and links in `EncryptedWorldState` are stored as AES-256-GCM ciphertext under K_w. A DMA snapshot sees:
- Encrypted blobs (unreadable without K_w)
- K_w itself (32 bytes, mlock'd — but readable by DMA)
- The guard buffer (8KB, currently zero unless caught during active decryption)

### Residual risk

K_w is in memory (mlock'd but readable by DMA). A DMA attacker who captures K_w can decrypt the encrypted store. The mitigation for this is Phase 3: K_w wrapped under K_s, with K_s in the Secure Enclave (never in RAM).

## Scatter-and-blind (design, not implemented)

For maximum DMA resilience without Secure Enclave:
1. Split each encrypted object into N fragments
2. XOR each fragment with a random blinding key
3. Store fragments in non-contiguous memory pages
4. Reassembly requires: all N fragments + the blinding key + K_w
5. A DMA snapshot sees scattered, blinded fragments

**Decision:** deferred. The encrypted store + mlock provides sufficient protection for the current threat model. Scatter-and-blind adds complexity without eliminating the fundamental K_w exposure (which requires Phase 3 Secure Enclave).

## Empirical DMA test

Test method: scan process memory for known plaintext strings during chamber operation.
- Search for object payload content (e.g., "TOP SECRET") in process memory
- Expected: found only inside the guard buffer during active decryption
- After burn: found nowhere

See `empirical-test-results.md` for results.
