# PRD — Chambers Isolation Phases

## Core thesis

A Chamber uses your machine. Your machine cannot extract what happened inside the Chamber.

**Inbound**: the chamber consumes CPU, memory, display, keyboard input. This is necessary. The chamber exists on the machine.

**Outbound**: no information escapes the chamber into the system, other processes, the network, or persistent storage. When the chamber burns, the key is destroyed and the content is unrecoverable. The machine knows a chamber existed. It does not know what was inside.

---

## What exists today

### Phase 0 — Substrate runtime (complete)

The core engine:
- World-first architecture: the chamber is the primary semantic unit
- Closed primitive algebra: 9 operations, no dynamic extension
- Typed objects with constrained payloads
- Preservation law: only explicitly sealed artifacts survive
- 6-layer burn: logical revocation → cryptographic erasure (K_w zeroized) → storage cleanup → memory cleanup → audit burn (Tier 2 events destroyed) → semantic residue measurement
- Artifact vault: sole cross-world channel
- Two-tier audit: Tier 1 (substrate-scoped, survives burn: WorldCreated + WorldDestroyed, max 2 entries) and Tier 2 (world-scoped, burned: phase transitions, convergence, sealing — destroyed on burn)
- Benchmark harness proving H1 (lower residue) and H3 (infeasible reconstruction) against VM baselines
- Empirical validation: 5 tests passed — audit tier destruction, content residue elimination, H1 benchmark (Chambers: 0 metadata vs VM: 3 vs microVM: 4), cross-world isolation, post-burn endpoint lockout

### Level 1 — Native application (complete)

The isolation shell:
- Native fullscreen app (tao + wry), not a browser tab
- Chamber takes over the entire screen
- No address bar, no back/forward, no browser history
- System cursor hidden; chamber renders its own pointer
- All system keyboard shortcuts blocked (Cmd+C/V/F/P/Z/S/Q)
- Right-click blocked
- All hardware APIs severed at JS level (camera, mic, bluetooth, USB, serial, MIDI, sensors)
- All storage APIs blocked (localStorage, IndexedDB, sessionStorage, Cache, Service Workers)
- All outbound network blocked (WebSocket, external fetch/XHR, EventSource)
- Clipboard API severed (read and write)
- Drag-and-drop blocked
- File picker blocked
- Spell check, autocorrect, autocomplete disabled
- Print, window.open, alert/confirm/prompt blocked
- Web Workers and SharedWorkers blocked
- Only allowed network: fetch to 127.0.0.1 (the substrate)
- Burn kills the app entirely — fullscreen vanishes, desktop returns
- Esc is the only escape hatch (triggers burn)

---

## Remaining outbound channels (application layer)

These are gaps that exist today and are closable without leaving user-space:

| Channel | Risk | Closable? |
|---------|------|-----------|
| Key material in swap | K_w could be paged to disk by macOS VM system | Yes — `mlock()` |
| Core dumps | Crash dump could contain plaintext state | Yes — `setrlimit(RLIMIT_CORE, 0)` |
| Process memory readable by debugger | Root or same-user process could attach | Partially — `ptrace` deny, but root bypasses |
| Temp files from WebKit | Webview engine may write to /tmp or cache dirs | Yes — audit + restrict with App Sandbox |
| macOS process accounting | OS logs that the app launched (process name, timestamps) | No — OS-level |
| Screenshot by another process | Another app could capture the screen | No — OS-level |

---

## Phase 2 — Application-layer hardening

No Apple Developer account required. All deliverables are user-space code.

**34 issues across 6 epics. Full issue list: `docs/Phase2_IssueList.md`**

### 2.1 Memory protection (7 issues)

**Goal**: Key material never touches disk. Plaintext state never survives process exit.

**Deliverables**:
- `mlock()` on K_w buffer and guard buffer — key material pinned in physical RAM, never paged to swap
- `madvise(MADV_DONTDUMP)` on sensitive memory regions — excluded from core dumps even if limits bypassed
- `setrlimit(RLIMIT_CORE, 0)` at process start — no core dumps on crash
- `zeroize` extended to all world state (Object, ObjectLink, ConvergenceReviewState, capability tokens) — not just K_w
- Integration test: memory scan after burn confirms zero plaintext remnants

**Acceptance**: after burn, attaching a debugger to the still-running process reveals no world content. After process exit, no swap file contains key material. Core dump signal produces no file.

### 2.2 WebKit cache isolation (5 issues)

**Goal**: The webview engine creates no persistent artifacts outside the substrate's control.

**Deliverables**:
- Configure wry/WebKit with ephemeral (non-persistent) data store (`WKWebsiteDataStore.nonPersistent()`)
- Set WebKit cache directory to a chamber-scoped tmpdir
- Disable HTTP cache, favicon cache, font cache, back-forward cache
- On burn, explicitly delete any WebKit temp directories
- Post-burn filesystem scan: `find / -newer <burn-timestamp>` returns nothing chamber-related

**Acceptance**: no WebKit artifacts on disk after burn. Zero files created outside substrate control.

**Risk**: wry may not expose WebKit ephemeral data store API. Mitigation: ObjC bridge or upstream PR.

### 2.3 Anti-debugging (4 issues)

**Goal**: Prevent external processes from reading chamber process memory.

**Deliverables**:
- `ptrace(PT_DENY_ATTACH)` at process start — blocks lldb, dtrace, process attachment (works unsigned)
- Debugger detection loop (500ms interval): check `P_TRACED` flag via sysctl, trigger emergency burn if detected
- Integration test: `lldb -p <pid>` returns "Operation not permitted"

**Acceptance**: debugger attachment fails. Dtrace probing fails.

**Known gap**: Hardened Runtime (DYLD_INSERT_LIBRARIES injection protection) requires Apple Developer account — deferred to Phase 3. Root-level attacker can still bypass ptrace. Documented.

### 2.4 Chamber clipboard (5 issues)

**Goal**: The chamber has its own clipboard that burns with the world. No data crosses to the system pasteboard.

**Deliverables**:
- `ChamberClipboard` struct: in-memory, world-scoped, zeroized on drop
- Cmd+C inside chamber: captures selection → stores in ChamberClipboard via HTTP endpoint (not system pasteboard)
- Cmd+V inside chamber: reads from ChamberClipboard → inserts at cursor (system pasteboard never read)
- Clipboard is zeroed and removed on burn
- Cross-chamber test: copy in chamber A, paste in chamber B — fails (clipboard is world-scoped)

**Acceptance**: copy text in chamber, burn, Cmd+V in TextEdit — nothing. System clipboard unchanged throughout.

### 2.5 Encrypted memory pool (9 issues — largest effort)

**Goal**: World state is ciphertext in RAM at all times. Plaintext exists only for microseconds in a single guarded buffer. A DMA snapshot of process memory yields nothing readable.

**Architecture**:

```
Phase 0/L1 (exposed):                    Phase 2 (hardened):
  RAM:                                      RAM:
    Object A: plaintext  ← exposed            Object A: ciphertext
    Object B: plaintext  ← exposed            Object B: ciphertext
    Object C: plaintext  ← exposed            Object C: ciphertext
    K_w: plaintext                             K_w: mlock'd
                                               Guard Buffer (8KB, mlock'd):
                                                 one object at a time
                                                 zeroed after every use
```

**Deliverables**:
- `EncryptedObject` and `EncryptedLink` types — ciphertext blobs with nonce (AES-256-GCM)
- Guard buffer — 8KB, page-aligned, mlock'd, MADV_DONTDUMP, zeroed after every use
- Scoped access API — `with_object(id, |plaintext| { ... })` enforced by Rust borrow checker. Plaintext cannot escape the closure.
- `EncryptedWorldState` replacing `WorldState` — objects and links stored as ciphertext
- All engines updated: OperationEngine, ViewEngine, Interpreter — all use scoped access
- Render pipeline change: view engine decrypts one object at a time, extracts display text, zeros, moves to next
- Integration tests: memory scan during operation shows no plaintext outside guard buffer

**Performance**: AES-256-GCM on Apple Silicon hardware: ~10ns per encrypt/decrypt. 500 objects × 50 ops/sec = 0.5ms overhead. Imperceptible.

**What this buys**: DMA exposure window drops from entire session to microseconds per access. Pre-burn memory dump is useless without K_w.

### 2.6 DMA-specific mitigations (4 issues)

**Deliverables**:
- Memory page isolation: `mmap(MAP_PRIVATE | MAP_ANONYMOUS)` + `mlock` + `MADV_DONTDUMP` for encrypted store. On burn: `memset` zero entire region before `munmap`.
- Apple Silicon IOMMU documentation: external DMA blocked by default on M-series.
- Scatter-and-blind design doc (optional, P2): split encrypted objects across non-contiguous pages with XOR blinding. Design only; implementation gated on threat model validation.
- DMA empirical test: scan process memory during chamber operation, search for known plaintext strings outside guard buffer. Expected: zero.

---

### Phase 2 exit criteria

Phase 2 is complete when ALL of the following are true:

1. K_w is mlock'd — never paged to swap
2. Core dumps disabled — crash produces no file
3. All world state zeroed on drop — not just K_w
4. WebKit creates no persistent artifacts — post-burn filesystem scan is clean
5. ptrace blocks debugger attachment — lldb/dtrace fail
6. Chamber clipboard is world-scoped — system pasteboard never touched
7. All objects and links encrypted in RAM under K_w — ciphertext only
8. Plaintext exists only in guard buffer for microseconds per access
9. Guard buffer is mlock'd, DONTDUMP'd, and zeroed after every use
10. All existing tests (35) still pass
11. DMA memory scan finds no plaintext outside guard buffer
12. Post-burn memory scan finds no plaintext anywhere

After Phase 2, the outbound isolation claim is:

> At the application layer, no information escapes the chamber to the host system. Key material is locked in RAM and zeroed on burn. World state is encrypted in memory at all times — plaintext exists only in a single guarded buffer for microseconds per access. The process creates no persistent artifacts. The system clipboard is untouched. External process attachment is denied. DMA observation yields only ciphertext. The only information the host retains is that the app was launched (process name and timestamps in OS logs).

---

## Phase 3 — OS-layer cooperation

### 3.1 Apple Developer account prerequisites

Phase 3 requires an Apple Developer account ($99/year, individual). This unlocks: code signing, Hardened Runtime, App Sandbox enforcement, Secure Enclave access, Hypervisor.framework entitlements, and notarization.

### 3.2 macOS App Sandbox (requires code signing)

**Goal**: The process itself cannot access files, network, or hardware even if JS-level blocks are bypassed.

**Deliverables**:
- Enable macOS App Sandbox entitlements:
  - No file system access (except substrate data directory)
  - No network access (except localhost loopback)
  - No camera, microphone, USB, Bluetooth entitlements
  - No printing
  - No Apple Events / IPC
- Sign the binary with restricted entitlements
- Hardened Runtime enabled (prevents code injection, DYLD_INSERT_LIBRARIES, etc.)

**Acceptance**: attempting to access any blocked resource from native code (not just JS) returns a sandbox violation. `DYLD_INSERT_LIBRARIES` injection is blocked.

### 3.3 Secure Enclave key storage

**Goal**: K_s (substrate key) never exists in main memory. It lives in the Secure Enclave.

**Deliverables**:
- Use Apple Security framework / Keychain with Secure Enclave backing
- K_s generated inside the Secure Enclave, never exported
- K_w wrapping/unwrapping operations performed by the Secure Enclave
- K_w itself is still in main memory (mlock'd) during chamber operation, but its wrapping key (K_s) is hardware-protected

**Acceptance**: even with full RAM dump, K_s is not recoverable because it never left the Secure Enclave.

### 3.4 Encrypted swap

**Goal**: If the OS does page chamber memory to disk, it's encrypted with a key the OS controls.

**Deliverables**:
- Verify FileVault is enabled (macOS full-disk encryption)
- Recommend / enforce encrypted swap (`sysctl vm.swapusage` + verify encryption)
- Document: if FileVault is off, swap is a residue channel

**Acceptance**: documented requirement. Installer checks FileVault status.

### 3.5 Chamber boot (Hypervisor.framework)

**Goal**: Opening a chamber is a boot, not an app launch. The chamber runs in a purpose-built VM that has no filesystem, no network stack, no shell, no purpose other than being the substrate. The host cannot inspect chamber memory. Burn destroys the VM.

**Why this is different from a generic disposable VM**:

A disposable VM (what we benchmarked against in Phase 0) is a generic environment running generic software. You spin up Linux, run scripts, delete the VM. The VM doesn't know about chambers, burn semantics, or preservation law. It's security-by-cleanup — delete the container, hope nothing leaked.

A chamber boot is security-by-construction. The VM is the substrate. It has no other capability. There is no shell to escape to, no filesystem to write to, no network to exfiltrate through. The only thing that can happen inside it is chamber law.

| | Generic disposable VM | Chamber boot |
|---|---|---|
| What runs inside | Full OS, shell, filesystem, apps | Bare substrate runtime only |
| Network | Has a network stack | No network stack exists |
| Storage | Disk image, filesystem, logs | RAM only — no persistent storage |
| At "delete" | VM image deleted, metadata may survive | K_w zeroed, VM memory freed by hypervisor. Nothing was ever on disk. |
| Who enforces rules | Nobody | Substrate — closed primitives, preservation law, burn semantics |
| Encryption | Optional | Mandatory — all state encrypted under K_w |
| Residue after destruction | Host metadata, potentially recoverable disk blocks | Nothing — memory freed, key destroyed, no disk was ever used |

**Deliverables**:

**3.3.1 VM image**

A minimal bootable image containing only:
- Linux microkernel (or custom minimal kernel) — just enough to manage memory and run the substrate
- The Chambers substrate runtime (Rust binary, statically linked)
- llama.cpp inference engine (statically linked, no Python, no dependencies)
- Model weights file (loaded at boot, see 3.4)
- Framebuffer driver (display output to host)
- virtio-input driver (keyboard/mouse from host)
- Nothing else — no shell, no package manager, no sshd, no cron, no syslog, no filesystem beyond initramfs

Total image size: ~50MB (kernel) + substrate binary (~10MB) + inference engine (~5MB) = ~65MB without weights.

**3.3.2 Boot sequence**

When the user clicks "Open Chamber":
1. Host allocates VM memory (8-32GB depending on model size)
2. Hypervisor.framework creates the VM with: no network, no disk, no USB, no Bluetooth — only framebuffer out and keyboard/mouse in
3. VM boots the microkernel (~200ms on Apple Silicon)
4. Substrate initializes, generates K_w, creates the world
5. Chamber UI renders to the VM's framebuffer
6. Host displays the framebuffer fullscreen — the chamber has "booted"
7. Total time from click to ready: 1-3 seconds

**3.3.3 Host-VM communication**

The only channels between host and VM:
- **Framebuffer** (VM → host): display output. Read-only from host perspective.
- **Keyboard/mouse** (host → VM): input events. Write-only from host perspective.
- **Virtio control** (host → VM): start/stop signals only.

No shared memory. No shared filesystem. No network bridge. No clipboard sharing. No drag-and-drop. The VM is an opaque box that accepts keystrokes and emits pixels.

**3.3.4 Burn sequence (VM destruction)**

1. Substrate executes 5-layer burn inside the VM (logical → crypto → storage → memory → semantic)
2. K_w is zeroed inside the VM
3. VM sends "burn complete" signal via virtio control
4. Host tells Hypervisor.framework to destroy the VM
5. Hypervisor frees all VM memory pages (returned to host physical memory pool)
6. Host returns to normal desktop
7. The VM never existed from the host's perspective — no disk image, no log files, no memory trace

**Acceptance**:
- Host-side memory scan after VM destruction finds no chamber content
- No files created on host filesystem during chamber session
- VM boot to ready in < 3 seconds on M-series Mac
- VM has no outbound network capability (verified by attempting connection from inside)

---

### 3.6 Chamber-born LLM

**Goal**: Every chamber gets its own model instance. Born at chamber boot. Dies at burn. No memory across chambers. No context leakage. No hidden state.

**Architecture**:

```
┌──────────────────── VM boundary ────────────────────┐
│                                                      │
│  Model weights (read-only, substrate-scoped)         │
│  ┌─────────────────────────────────────────────┐     │
│  │  Llama/Mistral/Phi weights (GGUF, ~4-8GB)   │     │
│  │  Loaded at boot. Same weights every time.    │     │
│  │  Not burned (infrastructure, like CPU ISA).  │     │
│  └─────────────────────────────────────────────┘     │
│                                                      │
│  Inference state (world-scoped, burned)              │
│  ┌─────────────────────────────────────────────┐     │
│  │  KV cache          — encrypted under K_w     │     │
│  │  Attention buffers — encrypted under K_w     │     │
│  │  Sampling state    — encrypted under K_w     │     │
│  │  Context window    — encrypted under K_w     │     │
│  │  Scratchpad        — encrypted under K_w     │     │
│  │                                               │     │
│  │  ALL of this is zeroed on burn.               │     │
│  │  The next chamber gets a blank model.         │     │
│  └─────────────────────────────────────────────┘     │
│                                                      │
│  Substrate runtime                                   │
│  ┌─────────────────────────────────────────────┐     │
│  │  Interpreter (enforces chamber law on model) │     │
│  │  World engine, object engine, burn engine    │     │
│  │  The model submits TransitionRequests like    │     │
│  │  any other principal. It cannot bypass the    │     │
│  │  interpreter.                                 │     │
│  └─────────────────────────────────────────────┘     │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**What is substrate-scoped (NOT burned)**:
- Model weights (GGUF file). These are infrastructure. The same model is loaded into every chamber. Burning the weights would mean re-downloading them every session. Weights are read-only and contain no chamber-specific information.

**What is world-scoped (burned)**:
- KV cache — the model's working memory of this conversation
- Attention state — what the model is "paying attention to"
- Sampling state — temperature, top-p, repetition penalties in effect
- Context window — the accumulated prompt + generation so far
- Any scratchpad or chain-of-thought buffers
- Generated tokens before they're submitted as objects

All world-scoped state is allocated inside the encrypted memory pool (Phase 2). Encrypted under K_w. Decrypted only in the guard buffer during active inference. Zeroed on burn.

**What the model can do**:
- Read chamber entries (decrypted one at a time through the guard buffer)
- Generate new entries (submitted through the interpreter, validated by the primitive algebra)
- Propose object creation, links, challenges, synthesis, condensation
- Propose convergence
- Act as the orchestrator (replacing the symbolic planner from Phase 0)

**What the model cannot do**:
- Create undeclared primitives (closed algebra, interpreter rejects)
- Bypass the interpreter (no direct state mutation path)
- Preserve what the preservation law doesn't allow (policy engine enforces)
- Unilaterally seal an artifact (requires human authorization)
- Access the network, filesystem, clipboard, or any hardware (VM has none)
- Remember anything after burn (inference state is world-scoped, key is destroyed)
- Communicate with models in other chambers (no cross-VM channel)

**Model selection for Apple Silicon**:

| Model | Size | RAM needed | Quality | Fit |
|---|---|---|---|---|
| Phi-3 Mini 4B (Q4) | ~2.5GB | 4GB | Good for structured reasoning | Best for 8GB Macs |
| Mistral 7B (Q4) | ~4GB | 8GB | Strong general capability | Good default |
| Llama 3 8B (Q4) | ~4.5GB | 8GB | Strong reasoning | Good default |
| Llama 3 13B (Q4) | ~7GB | 16GB | Excellent reasoning | Best quality |
| Mixtral 8x7B (Q4) | ~26GB | 32GB | Near-frontier | High-end Macs only |

Inference engine: llama.cpp, compiled for Apple Silicon with Metal acceleration. Runs entirely on the GPU/ANE — no CPU fallback, no external API calls.

**The birth-and-death cycle**:

1. User opens chamber → VM boots → model weights loaded (read-only mmap) → K_w generated → world created
2. Model inference begins — generates initial analysis of the objective, proposes entries
3. User and model collaborate — user adds entries, model synthesizes, challenges, ranks
4. Convergence — model proposes decision summary, user authorizes sealing
5. Burn — inference state zeroed, K_w destroyed, VM killed, model is dead
6. Next chamber — fresh VM, fresh K_w, clean model instance. Zero memory of what happened.

**Acceptance**:
- Model generates valid TransitionRequests that pass interpreter validation
- Model cannot create objects of types not in the grammar
- Model cannot seal artifacts without human authorization
- After burn, no inference state (KV cache, context, attention) is recoverable
- Model in chamber A has zero knowledge of chamber B
- Inference runs at > 10 tokens/second on target hardware (M-series, 16GB+)

---

### 3.7 Model context as world-state (formal treatment)

The paper requires that if an LLM is used, its inference context must be treated as world-state. This means:

- **Context window = world-state**. It's encrypted under K_w, stored in the encrypted memory pool, and burned with the world.
- **KV cache = world-state**. Same treatment.
- **Model weights ≠ world-state**. They're substrate-scoped infrastructure. Not burned. Not specific to any chamber.
- **No hidden scratch state**. If the model uses chain-of-thought, scratchpads, or internal buffers, all of it is world-scoped and encrypted. There is no "thinking space" outside chamber law.
- **No cross-chamber learning**. The model does not fine-tune, accumulate RLHF feedback, or update its weights based on chamber content. Every chamber gets the same base model.

This is what separates a chamber-born model from a cloud LLM:
- A cloud LLM's provider sees your prompts, stores your conversations, potentially trains on them.
- A chamber-born model's entire cognitive state is encrypted inside the chamber and destroyed on burn. There is no provider. There is no server. There is no log.

---

### Phase 3 exit criteria

> The substrate key is hardware-protected (Secure Enclave). Disk is encrypted (FileVault). Chamber execution happens inside a purpose-built VM with no network, no filesystem, no shell (Hypervisor.framework). A local LLM is born inside each chamber — its inference state is world-scoped, encrypted under K_w, and destroyed on burn. The host knows a VM ran. It does not know what was inside, what the model thought, or what decision was made. The model has no memory of prior chambers.

---

## Phase 4 — Platform

This is where the product expands. Isolation is proven. Now scale it.

- Multiple chamber grammars (research, negotiation, triage, planning, legal review, etc.)
- Multi-chamber orchestration (chambers that spawn sub-chambers, each with their own model instance and burn cycle)
- Import/export with explicit policy controls (data enters a chamber through a policy gate; data leaves only as sealed artifacts)
- Institutional vault governance (who can access which artifacts, retention policies, audit trails)
- Enterprise fleet management (managed chambers across an organization, compliance reporting)
- Model selection per grammar (different grammars may benefit from different model sizes/specializations)
- Consumer UX (the research tool becomes a product)

Phase 4 only happens if Phases 2-3 prove the isolation model works and the chamber-born LLM is useful enough to justify the architecture.

---

## Phase summary

| Phase | Focus | Key deliverable | Threat boundary |
|-------|-------|-----------------|-----------------|
| **0** (done) | Substrate | Runtime, burn engine, crypto, benchmark | Trusted substrate assumption |
| **L1** (done) | Application shell | Native app, JS isolation, fullscreen takeover | Application-layer block of all outbound APIs |
| **2** | Application hardening (no dev account) | mlock, core dump disable, WebKit cache isolation, ptrace anti-debug, chamber clipboard, encrypted memory pool, DMA resilience | No application-layer outbound channel; DMA observation yields only ciphertext |
| **3** | Apple cooperation + chamber-born LLM | Dev account, App Sandbox, Hardened Runtime, Secure Enclave, encrypted swap, hypervisor boot, local model born and killed per chamber | OS-enforced sandbox, hardware-backed keys, VM isolation, model burned per chamber |
| **4** | Product expansion | Multi-grammar, multi-chamber, enterprise, consumer | Same isolation, broader functionality, model selection per grammar |

---

## Remote access claim

Even under full hardware compromise, remote access into a running chamber is impossible. The chamber has no inbound network listener on any external interface, no RPC endpoint, no IPC channel, no shared memory, no signal handler, and no named pipe. Input is accepted only from the local event loop (physical keyboard and pointer).

A compromised machine can passively observe the chamber (DMA read, framebuffer capture, keystroke interception) but cannot inject commands or extract data remotely.

From Phase 2 onward, passive observation is also degraded:
- **DMA observation**: yields only ciphertext. World state is encrypted in RAM. Plaintext exists in a single guard buffer for microseconds per access. A DMA snapshot captures at most one object fragment before it's zeroed.
- **Framebuffer capture**: shows the rendered UI (display text). Does not reveal the structured object graph, link topology, or non-displayed payload fields.
- **Keystroke interception**: captures what the user typed. Does not capture system-generated state (convergence analysis, link inference, synthesis results).
- **After burn**: all passive observation becomes worthless. K_w is destroyed. Ciphertext is unrecoverable. The framebuffer is black. The process is dead.

This claim has two layers:
1. **Structural** (all phases): there is no inbound door. No OS cooperation needed.
2. **Cryptographic** (Phase 2+): even passive observation yields only ciphertext. Requires encrypted memory pool.

---

## One-line thesis per phase

- **Phase 0**: Can a world-first runtime with burn semantics reduce semantic residue? (Yes.)
- **Level 1**: Can a human operate the chamber without reintroducing system-level leakage? (Yes, at the application layer.)
- **Phase 2**: Can we close every application-layer outbound channel? (mlock, sandbox, anti-debug, cache wipe.)
- **Phase 3**: Can we give the chamber its own execution context and its own intelligence? (Hypervisor boot, Secure Enclave, chamber-born LLM that dies on burn.)
- **Phase 4**: Does this model scale to a real product? (Multiple grammars, multi-chamber orchestration, enterprise, consumer.)
