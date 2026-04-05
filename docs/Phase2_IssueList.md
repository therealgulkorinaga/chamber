# Phase 2 — Application-Layer Hardening Issue List

No Apple Developer account required. All deliverables are user-space code.

**Total issues: 34**
**Epics: 6**
**Estimated duration: 2–3 weeks (single engineer)**

---

## Legend

- **P0**: blocking — must complete before epic is done
- **P1**: high — important but not blocking
- **P2**: optional — deferred if time-constrained
- **Depends**: must complete before this issue starts

---

# Epic 1: Memory Protection (Issues 1–7)

### Issue 1: mlock on K_w buffer
**Goal**: World key never paged to swap.
**Tasks**:
- Call `libc::mlock()` on the `WorldKey.key_bytes` buffer immediately after generation in `CryptoProvider::generate_world_key()`
- Verify with `mincore()` that the page is resident
**Crate**: `chambers-crypto`
**Priority**: P0
**Depends**: none
**Acceptance**: `WorldKey` buffer is locked in physical RAM. `sysctl vm.swapusage` shows no increase during chamber operation with mlock enabled.

### Issue 2: setrlimit to disable core dumps
**Goal**: Process crash never writes sensitive memory to disk.
**Tasks**:
- At process start (in `chambers-app/src/main.rs`), call `libc::setrlimit(RLIMIT_CORE, &rlimit { rlim_cur: 0, rlim_max: 0 })`
- Verify with `ulimit -c` equivalent
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: none
**Acceptance**: `kill -SIGABRT <pid>` produces no core file.

### Issue 3: madvise MADV_DONTDUMP on sensitive regions
**Goal**: Even if core dump limits are bypassed, sensitive regions are excluded.
**Tasks**:
- Call `libc::madvise(ptr, len, MADV_DONTDUMP)` on the guard buffer and K_w buffer
- Apply to any mmap'd encrypted object store region
**Crate**: `chambers-crypto`
**Priority**: P1
**Depends**: Issue 1

### Issue 4: Extend zeroize to all world state
**Goal**: All sensitive allocations are zeroed on drop, not just K_w.
**Tasks**:
- Add `#[derive(Zeroize, ZeroizeOnDrop)]` to `Object`, `ObjectLink`, `ConvergenceReviewState`
- Verify: after `StateEngine::destroy_world_state()`, the backing memory is zeroed before deallocation
- Add zeroize to capability token storage
**Crate**: `chambers-types`, `chambers-state`, `chambers-capability`
**Priority**: P0
**Depends**: none
**Acceptance**: Memory scan of freed heap after burn shows no plaintext object payloads.

### Issue 5: mlock on guard buffer
**Goal**: The decryption guard buffer (Phase 2 encrypted memory pool) is pinned in RAM.
**Tasks**:
- Allocate guard buffer with `mmap(MAP_PRIVATE | MAP_ANONYMOUS)`
- Immediately `mlock()` the region
- On burn: `memset` zero, `munlock`, `munmap`
**Crate**: `chambers-crypto`
**Priority**: P0
**Depends**: Issue 14 (guard buffer creation)
**Acceptance**: Guard buffer page is resident throughout chamber session.

### Issue 6: Memory protection integration test
**Tasks**:
- Test: create chamber, add objects, burn. Scan `/proc/self/maps` (or macOS equivalent) for locked pages.
- Test: trigger crash signal, verify no core dump written.
- Test: after burn, read freed memory regions — verify zeros.
**Crate**: `chambers-runtime` (tests)
**Priority**: P0
**Depends**: Issues 1–5

### Issue 7: Document memory protection architecture
**Tasks**:
- Architecture note covering: what's mlock'd, what's zeroized, what's DONTDUMP'd
- Threat matrix: what each mitigation addresses
**Deliverable**: `docs/memory-protection.md`
**Priority**: P1
**Depends**: Issue 6

---

# Epic 2: WebKit Cache Isolation (Issues 8–12)

### Issue 8: Configure wry with ephemeral data store
**Goal**: WebKit creates no persistent storage.
**Tasks**:
- Research `wry` API for non-persistent WebKit data store (`WKWebsiteDataStore.nonPersistent()`)
- If available: configure in `WebViewBuilder`
- If not available: file upstream issue or bridge via ObjC FFI
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: none
**Risk**: May require wry PR or ObjC bridge if API not exposed.

### Issue 9: Disable WebKit caches
**Tasks**:
- Disable HTTP cache (set `URLCache` capacity to 0)
- Disable favicon cache
- Disable font cache
- Disable back-forward cache
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: Issue 8

### Issue 10: Redirect WebKit temp directory
**Tasks**:
- Set WebKit's data directory to a chamber-scoped tmpdir
- On burn, `rm -rf` the tmpdir
- Verify no files remain in default WebKit cache locations (`~/Library/WebKit/`, `~/Library/Caches/`)
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: Issue 8

### Issue 11: Post-burn filesystem scan test
**Tasks**:
- Before chamber: snapshot filesystem timestamps
- Run a chamber session (create, add objects, burn)
- After burn: `find / -newer <timestamp> -user $(whoami)` excluding known safe paths
- Verify zero chamber-related files created
**Crate**: `chambers-app` (tests)
**Priority**: P0
**Depends**: Issues 9, 10
**Acceptance**: No WebKit artifacts found on disk after burn.

### Issue 12: Document WebKit isolation
**Deliverable**: `docs/webkit-isolation.md`
**Priority**: P2
**Depends**: Issue 11

---

# Epic 3: Anti-Debugging (Issues 13–16)

### Issue 13: ptrace PT_DENY_ATTACH at process start
**Goal**: No debugger can attach to the running process.
**Tasks**:
- In `main()`, before any chamber logic: `libc::ptrace(PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0)`
- Wrap in a platform-specific block (`#[cfg(target_os = "macos")]`)
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: none
**Acceptance**: `lldb -p <pid>` returns "attach failed" / "Operation not permitted".

### Issue 14: Debugger detection with emergency burn
**Goal**: If something attaches despite ptrace deny, trigger burn immediately.
**Tasks**:
- Periodically check `sysctl(CTL_KERN, KERN_PROC, KERN_PROC_PID)` for `P_TRACED` flag
- If detected: trigger abort burn on all active chambers, then exit
- Check interval: every 500ms in a background thread
**Crate**: `chambers-app`
**Priority**: P1
**Depends**: Issue 13
**Risk**: False positives from system monitoring tools. Needs testing.

### Issue 15: Anti-debugging integration test
**Tasks**:
- Test: launch chambers, attempt `lldb -p <pid>` from another terminal — verify failure
- Test: attempt `dtrace -p <pid>` — verify failure
- Test: attempt `DYLD_INSERT_LIBRARIES` injection (expected: works without Hardened Runtime, documented as Phase 3 gap)
**Crate**: `chambers-app` (tests)
**Priority**: P0
**Depends**: Issue 13

### Issue 16: Document anti-debugging and known gaps
**Tasks**:
- Document: ptrace works unsigned, Hardened Runtime needs dev account (Phase 3)
- Document: root can still bypass ptrace on macOS
- Document: DYLD injection is the Phase 3 gap
**Deliverable**: `docs/anti-debugging.md`
**Priority**: P1
**Depends**: Issue 15

---

# Epic 4: Chamber Clipboard (Issues 17–21)

### Issue 17: In-memory chamber clipboard
**Goal**: Chamber has its own clipboard that doesn't touch the system pasteboard.
**Tasks**:
- Create `ChamberClipboard` struct: `content: Option<String>`, world-scoped, zeroized on drop
- Store in the app state, keyed by world ID
- On burn: clipboard is zeroed and removed
**Crate**: `chambers-app`
**Priority**: P0
**Depends**: none

### Issue 18: Intercept Cmd+C in chamber webview
**Tasks**:
- The JS keyboard handler already blocks Cmd+C. Extend it to:
  1. Get the current text selection
  2. POST the selected text to `/app/clipboard/copy` (stores in ChamberClipboard, not system)
  3. Visual feedback: brief flash or "copied" indicator
**Crate**: `chambers-app` (JS + Rust endpoint)
**Priority**: P0
**Depends**: Issue 17

### Issue 19: Intercept Cmd+V in chamber webview
**Tasks**:
- Extend the JS keyboard handler:
  1. On Cmd+V in an input/textarea: GET `/app/clipboard/paste`
  2. Insert the returned text at cursor position
  3. If clipboard is empty: do nothing
- The system pasteboard is never read
**Crate**: `chambers-app` (JS + Rust endpoint)
**Priority**: P0
**Depends**: Issue 17

### Issue 20: Clipboard isolation test
**Tasks**:
- Test: copy text in chamber, burn chamber, Cmd+V in TextEdit — verify nothing pasted (system clipboard unchanged)
- Test: copy text in TextEdit before opening chamber, Cmd+V inside chamber — verify nothing pasted (system clipboard not read)
- Test: copy inside chamber, paste inside same chamber — verify it works
- Test: copy in chamber A, paste in chamber B — verify it does NOT work (clipboard is world-scoped)
**Crate**: `chambers-app` (tests)
**Priority**: P0
**Depends**: Issues 18, 19

### Issue 21: Clipboard zeroed on burn
**Tasks**:
- Verify: after burn, ChamberClipboard for that world is zeroed and removed from memory
- Scan process memory for clipboard content after burn
**Crate**: `chambers-app` (tests)
**Priority**: P1
**Depends**: Issue 20

---

# Epic 5: Encrypted Memory Pool (Issues 22–30)

This is the largest engineering effort in Phase 2.

### Issue 22: EncryptedObject type
**Goal**: In-memory representation of an encrypted object.
**Tasks**:
- Define `EncryptedObject { object_id: ObjectId, ciphertext: Vec<u8>, nonce: [u8; 12] }`
- Implement `encrypt(object: &Object, key: &WorldKey) -> EncryptedObject`
- Implement `decrypt(encrypted: &EncryptedObject, key: &WorldKey) -> Object`
- Round-trip test: encrypt → decrypt → compare
**Crate**: `chambers-crypto`
**Priority**: P0
**Depends**: none

### Issue 23: EncryptedLink type
**Tasks**:
- Define `EncryptedLink { ciphertext: Vec<u8>, nonce: [u8; 12] }`
- Encrypt/decrypt `ObjectLink` structs
**Crate**: `chambers-crypto`
**Priority**: P0
**Depends**: Issue 22

### Issue 24: Guard buffer allocation
**Tasks**:
- Allocate a single page-aligned buffer (8KB) via `mmap(MAP_PRIVATE | MAP_ANONYMOUS)`
- `mlock()` immediately
- `madvise(MADV_DONTDUMP)`
- Provide `GuardBuffer::write()` and `GuardBuffer::zero()` methods
- `zero()` called after every use
**Crate**: `chambers-crypto`
**Priority**: P0
**Depends**: Issue 5

### Issue 25: Scoped access API
**Goal**: Compile-time enforcement that plaintext cannot escape the guard buffer.
**Tasks**:
- Implement `EncryptedWorldState::with_object<F, R>(id, f: F) -> R where F: FnOnce(&Object) -> R`
  1. Decrypt into guard buffer
  2. Deserialize
  3. Call closure with reference
  4. Zero guard buffer
  5. Return closure result (must be `Copy` or owned, not a reference)
- Implement `with_object_mut<F>(id, f: F)` for modifications (re-encrypts after)
**Crate**: `chambers-state`, `chambers-crypto`
**Priority**: P0
**Depends**: Issues 22, 24

### Issue 26: Replace WorldState with EncryptedWorldState
**Goal**: The in-memory object store holds only ciphertext.
**Tasks**:
- New struct `EncryptedWorldState { objects: HashMap<ObjectId, EncryptedObject>, links: Vec<EncryptedLink>, ... }`
- Migration: `StateEngine` creates `EncryptedWorldState` per world, keyed with `WorldId`
- `StateEngine` needs access to `CryptoProvider` for encrypt/decrypt
- All existing `with_world_state()` / `with_world_state_mut()` calls must be updated to use the scoped access pattern
**Crate**: `chambers-state`
**Priority**: P0
**Depends**: Issues 22, 23, 25
**Estimate**: Large — this touches every engine that reads objects.

### Issue 27: Update OperationEngine for encrypted state
**Tasks**:
- Every operation (create, link, challenge, rank, synthesize, condense, seal) must:
  1. Decrypt target object(s) via scoped access
  2. Perform the operation
  3. Re-encrypt the result
  4. Zero the guard buffer
- `exec_create_object`: serialize + encrypt new object, store as EncryptedObject
- `exec_link_objects`: decrypt both objects to verify existence, create EncryptedLink
- `exec_seal_artifact`: decrypt to verify preservability, create artifact (artifact payload is plaintext in vault — it's a survivor)
**Crate**: `chambers-operation`
**Priority**: P0
**Depends**: Issue 26

### Issue 28: Update ViewEngine for encrypted state
**Tasks**:
- `conversation_view()`: iterate encrypted objects, decrypt one at a time, extract display text, zero, move to next
- `graph_view()`: same — decrypt nodes one at a time, build GraphNode list
- `summary_view()`: count encrypted objects by decrypting metadata only (or store unencrypted type/class index)
- The webview receives rendered HTML strings, not structured objects
**Crate**: `chambers-view`
**Priority**: P0
**Depends**: Issue 26
**Design decision**: Consider storing a plaintext index of `(ObjectId, object_type, lifecycle_class, preservable)` alongside the encrypted store. This avoids decrypting every object just to count types. The index reveals type distribution but not content. Document this as an accepted metadata exposure.

### Issue 29: Update Interpreter for encrypted state
**Tasks**:
- Validation checks (type compatibility, preservation law) need to decrypt target objects briefly
- Update `check_type_compatibility`, `check_preservation_law`, `verify_objects_in_world` to use scoped access
**Crate**: `chambers-interpreter`
**Priority**: P0
**Depends**: Issue 26

### Issue 30: Encrypted memory pool integration tests
**Tasks**:
- Test: create object, verify it's stored as ciphertext in WorldState (not plaintext)
- Test: read object via scoped API, verify plaintext returned correctly
- Test: after scope exits, verify guard buffer is zeroed
- Test: burn world, scan HashMap — verify all values are ciphertext (or removed)
- Test: decrypt without K_w (after burn) — verify failure
- Performance test: create 500 objects, refresh all views, measure total time (must be < 5ms overhead)
**Crate**: `chambers-runtime` (tests)
**Priority**: P0
**Depends**: Issues 27, 28, 29

---

# Epic 6: DMA Mitigations (Issues 31–34)

### Issue 31: Memory page isolation for encrypted store
**Tasks**:
- Allocate the encrypted object store's backing memory via `mmap(MAP_PRIVATE | MAP_ANONYMOUS)` instead of default heap
- `mlock()` the entire region
- `madvise(MADV_DONTDUMP)`
- On burn: `memset` zero the entire region, then `munlock`, `munmap`
**Crate**: `chambers-state`, `chambers-crypto`
**Priority**: P1
**Depends**: Issue 26

### Issue 32: Apple Silicon IOMMU documentation
**Tasks**:
- Document: Apple Silicon IOMMU blocks external DMA by default
- Document: Thunderbolt/USB-C DMA attacks are mitigated on M-series
- Document: remaining risk is compromised internal controllers with pre-authorized DMA
- Reference Apple Platform Security Guide
**Deliverable**: `docs/dma-mitigations.md`
**Priority**: P1
**Depends**: none

### Issue 33: Scatter-and-blind architecture (design only)
**Tasks**:
- Design document: how to split encrypted objects across non-contiguous pages with XOR blinding
- Threat model validation: is continuous DMA observation realistic for target users?
- Decision: implement in Phase 2 or defer?
**Deliverable**: `docs/scatter-blind-design.md`
**Priority**: P2
**Depends**: Issue 32

### Issue 34: DMA resilience empirical test
**Tasks**:
- Scan process memory (via `/proc/self/mem` equivalent or a debug harness) during chamber operation
- Search for known plaintext strings (object payloads) outside the guard buffer
- Measure: how many bytes of plaintext exist at any given moment?
- Expected: only guard buffer (8KB max) contains plaintext, and only during active decryption
**Crate**: `chambers-runtime` (tests)
**Priority**: P0
**Depends**: Issue 30

---

# Phase 2 Exit Criteria

Phase 2 is complete when ALL of the following are true:

1. Key material (K_w) is mlock'd and never paged to swap
2. Core dumps are disabled
3. All world state is zeroed on drop (not just K_w)
4. WebKit creates no persistent artifacts; post-burn filesystem scan is clean
5. ptrace(PT_DENY_ATTACH) prevents debugger attachment
6. Chamber clipboard is world-scoped; system clipboard is never touched
7. All objects and links are encrypted in RAM under K_w
8. Plaintext exists only in the guard buffer for microseconds per access
9. Guard buffer is mlock'd, DONTDUMP'd, and zeroed after every use
10. All existing tests (35) still pass
11. DMA memory scan finds no plaintext outside the guard buffer
12. Post-burn memory scan finds no plaintext anywhere

---

# Dependency Graph

```
Issue 1 (mlock K_w)─────────┐
Issue 2 (core dump)          │
Issue 3 (MADV_DONTDUMP)──┐  │
Issue 4 (zeroize all)     │  │
                          ├──┼── Issue 6 (memory test) ── Issue 7 (doc)
Issue 5 (mlock guard)─────┘  │
                              │
Issue 8 (wry ephemeral)──┐   │
Issue 9 (disable caches)─┤   │
Issue 10 (tmpdir)─────────┤   │
                          └── Issue 11 (filesystem test) ── Issue 12 (doc)
                              │
Issue 13 (ptrace)─────────── Issue 15 (debug test) ── Issue 16 (doc)
Issue 14 (detection)──────┘
                              │
Issue 17 (clipboard struct)──┐│
Issue 18 (Cmd+C)─────────────┤│
Issue 19 (Cmd+V)─────────────┤│
                              ├── Issue 20 (clipboard test) ── Issue 21 (zero test)
                              │
Issue 22 (EncryptedObject)───┐│
Issue 23 (EncryptedLink)─────┤│
Issue 24 (guard buffer)──────┤│
Issue 25 (scoped API)────────┤│
                              ├── Issue 26 (EncryptedWorldState)
                              │         │
                              │    ┌────┼────┐
                              │    │    │    │
                              │  Iss.27 28  29 (operation, view, interpreter)
                              │    │    │    │
                              │    └────┼────┘
                              │         │
                              │    Issue 30 (integration tests)
                              │         │
Issue 31 (mmap isolation)─────┘         │
Issue 32 (IOMMU doc)                    │
Issue 33 (scatter design)               │
                                        │
                               Issue 34 (DMA empirical test)
```

---

# Suggested Implementation Order

1. **Quick wins (day 1)**: Issues 1, 2, 3, 13 — mlock, core dump, MADV_DONTDUMP, ptrace. Four small changes, immediate security gain.
2. **Clipboard (days 2–3)**: Issues 17, 18, 19, 20, 21 — chamber clipboard. Self-contained feature.
3. **WebKit isolation (days 3–4)**: Issues 8, 9, 10, 11 — depends on wry API research.
4. **Encrypted memory pool (days 5–12)**: Issues 22–30 — the big one. Start with EncryptedObject/Guard buffer, then migrate WorldState, then update all engines.
5. **DMA and docs (days 13–14)**: Issues 31–34 — mmap isolation, documentation, empirical test.
6. **Integration and verification (day 15)**: Issue 6, 15, full test suite, final empirical validation.
