# Chambers: Sealed Ephemeral Worlds for Private Cognition

## A World-Based Position Paper on Persistence-Law-First, Burn-First, Task-Bounded Computing

**Arko Ganguli**

*Revised manuscript — v3 with peer review corrections*

---

## Abstract

Privacy-preserving computing has advanced through amnesic live operating systems, compartmentalized virtualization, verified boot, sandboxing, and hardware-backed attestation. This paper proceeds under a trusted-substrate assumption: it does not claim protection against lower-platform compromise, firmware compromise, DMA-class attacks, or malicious peripherals. Its contribution is narrower.

Rather than treating the application or guest machine as the primary semantic unit, it treats a bounded world-state as the primary unit of persistence, reasoning, and destruction.

This paper proposes Chambers, a world-based, persistence-law-first model in which a permanent substrate generates sealed, temporary computational worlds for bounded tasks. Chambers does not eliminate software that plays an orchestration role; rather, it rejects the application as the primary semantic and persistence unit. A Chamber is defined by typed internal objects, trusted operations, explicit lifecycle constraints, preservation law, and burn semantics that destroy temporary state once a task has converged or aborted.

The paper does not claim universal superiority over disposable virtual machines. Its narrower claim is that Chambers may be over and above VM-based approaches on four axes: semantic residue minimization, lifecycle legibility, preservation narrowness, and legal execution surface.

**Legal execution surface** is defined operationally as the number of distinct operations the environment permits. Chambers: 9 (a closed primitive algebra). gVisor: ~200 (filtered syscalls). A disposable VM or container: unbounded (any code can execute inside the guest). A smaller legal execution surface means fewer channels through which residue can be created, leaked, or retained.

The architecture is meaningful even if the orchestration layer is implemented by a symbolic planner or a smaller model rather than a large language model.

**Implementation note (v3):** A working substrate runtime, benchmark harness, and native application shell have been built and tested under application-layer assumptions (Section 2.4 documents what is not claimed). Comparative trace analysis against real disposable VM and Docker baselines shows: (a) zero undeclared residue for Chambers versus incidental metadata retention for baselines; (b) structural infeasibility of reconstruction for Chambers versus feasible reconstruction for baselines; (c) a structural predictability advantage from explicit preservation law, pending validation with human participants. The implementation reveals additional architectural requirements — application-layer isolation, encrypted memory pools, chamber-born model instances, and hypervisor boot — documented in this revision.

**Keywords:** privacy-preserving systems; ephemeral computing; world-based computing; capability systems; secure deletion; cryptographic erasure; disposable environments; burn semantics; semantic residue; persistence-law-first computing.

---

## 1. Introduction

Most privacy systems improve computing by making persistent environments safer rather than by rejecting persistence itself. Tails presents a portable operating system intended to leave no trace on the host after shutdown; Qubes offers disposable and stateless guest environments; GrapheneOS hardens a general-purpose platform. These systems solve real problems, but they preserve a common assumption: the durable machine or reusable application environment remains the default computational unit.

These systems are important, but they share a common premise: the fundamental unit of computation remains the persistent machine or the reusable application environment. Even where a guest is disposable, the guest is still machine-shaped. Even where shutdown minimizes host traces, the ontology remains application-centric. Even where the platform is heavily hardened, applications, files, recovery state, caches, and continuity remain first-class design assumptions.

Unikernel approaches (MirageOS [21]) eliminate unnecessary OS components, reducing the persistence surface. Confidential computing (AMD SEV-SNP [23], Intel TDX [24], ARM CCA [25], Google Confidential VMs [22]) provides hardware-enforced memory encryption, preventing even the hypervisor from reading guest state. These are important advances, but they preserve the application-centric model — the unit of computation is still the program or VM, not the bounded task.

This paper starts from a narrower premise. Under a trusted-substrate assumption, it asks: what follows if the bounded world, not the application, is the primary semantic and persistence unit?

**Contribution.** The novelty is not in any individual component — typed objects [18], capability systems [11], secure deletion [15][16][17], hardware enclaves [23][24], and ephemeral execution [1][2][13] all have precedent. The novelty is their composition into a single runtime model where the world, not the application, is the primary unit of persistence and destruction, governed by explicit preservation law and evaluated in terms of semantic residue. No prior system combines: (a) a world-first execution unit; (b) a closed primitive algebra constraining all world evolution; (c) formal preservation law declaring what may survive; (d) task-bounded grammars defining permitted object types and lifecycle phases; (e) cryptographic burn with residue-oriented semantics. Each component exists in isolation; the composition does not.

---

## 2. Threat Model: Inbound Access, Outbound Isolation

*[New section — v2]*

The original paper stated a "trusted-substrate assumption." Implementation reveals a cleaner framing.

### 2.1 The Inbound/Outbound Distinction

**Inbound access** is necessary and permitted. The chamber uses the machine — CPU, memory, display, keyboard input. The chamber exists *on* the machine. This is not a violation of isolation; it is the precondition for the chamber to function.

**Outbound access** is the threat. No information may escape the chamber through application-layer channels — the network, persistent storage, the clipboard, the filesystem, or blocked hardware APIs. When the chamber burns, the key is destroyed and the content is unrecoverable through those channels. Section 2.2 documents what is explicitly not claimed: OS-level visibility, framebuffer capture, keystroke interception, and DMA access to K_w remain outside the application-layer boundary.

The claim is therefore:

> **The machine knows a chamber existed. It does not know what was inside.**

This framing replaces the binary "trusted/untrusted" model with a directional one. The substrate consumes system resources (inbound). The substrate prevents information exfiltration (outbound). These are different problems requiring different solutions.

### 2.2 What the System Does Not Claim

The following are **not claimed** at any phase:

- **OS-level invisibility.** macOS logs that the app launched (process name, timestamps, resource usage). An observer can determine a chamber existed.
- **Framebuffer protection.** The rendered UI is visible to screen capture by co-resident processes. Displayed content is a residue channel for visual information.
- **Keystroke protection.** A keylogger captures user input. System-generated state (convergence analysis, synthesis results) is not captured, but typed content is.
- **DMA-proof memory.** K_w is in mlock'd RAM during operation. A DMA attacker who reads K_w can decrypt the encrypted memory pool. Hardware-grade memory encryption (SEV/TDX/SGX) provides stronger guarantees not achievable in user space.
- **Protection against root.** A root-level attacker can bypass ptrace(PT_DENY_ATTACH) and read process memory. Hardened Runtime (requiring code signing) mitigates this but is deferred to Phase 3.
- **Anonymity.** Chambers has no network and provides no identity protection. It is complementary to anonymity tools (e.g., Chambers inside Tails).

These limitations are architectural, not implementation gaps. They define the boundary of what application-layer isolation can achieve.

### 2.3 Outbound Channels and Their Status

Implementation identified the following outbound channels and their mitigations:

**Application-layer channels (closable in user space):**

| Channel | Mitigation | Status |
|---------|-----------|--------|
| Clipboard (system) | API severed; chamber-scoped clipboard | Implemented |
| Network (external) | fetch/XHR/WebSocket locked to localhost only | Implemented |
| File system | File picker, File API blocked | Implemented |
| Browser storage (localStorage, IndexedDB, Cache) | APIs overridden to throw | Implemented |
| Service Workers, Web Workers | Constructors blocked | Implemented |
| Drag-and-drop | Events prevented | Implemented |
| Camera, microphone, screen capture | getUserMedia, getDisplayMedia blocked | Implemented |
| Bluetooth, USB, Serial, MIDI | Navigator APIs blocked | Implemented |
| Notifications, Print, Share, Payment | APIs blocked | Implemented |
| Speech recognition/synthesis | APIs removed | Implemented |
| Sensors (accelerometer, gyroscope, ambient light) | Constructors removed | Implemented |
| System keyboard shortcuts (Cmd+C/V/F/P/Z/S) | Intercepted and prevented | Implemented |
| Right-click context menu | Event prevented | Implemented |
| Browser history, back/forward navigation | Native window, no browser | Implemented |
| Key material in swap | mlock | Phase 2 |
| Core dumps | setrlimit(RLIMIT_CORE, 0) | Phase 2 |
| WebKit temp files/cache | Ephemeral data store, post-burn cleanup | Phase 2 |
| Process attachment (debugger) | ptrace(PT_DENY_ATTACH) + Hardened Runtime | Phase 2 |

**OS-layer channels (require OS cooperation):**

| Channel | Mitigation | Status |
|---------|-----------|--------|
| Process accounting (OS logs app launch) | Not preventable; reveals only that app ran, not content | Accepted |
| Screenshot by another process | Not preventable from user space | Accepted |
| Swap encryption | FileVault enforcement | Phase 3 |

**Hardware-layer channels (require hardware trust):**

| Channel | Mitigation | Status |
|---------|-----------|--------|
| DMA observation | Encrypted memory pool (plaintext exists only in guard buffer for microseconds) | Phase 2-3 |
| Framebuffer capture | Shows rendered UI only, not structured object graph | Accepted |
| Keystroke interception | Captures user input only, not system-generated state | Accepted |

### 2.4 The Remote Access Structural Claim

Even under full hardware compromise, remote access into a running chamber is impossible. The chamber has no inbound network listener on any external interface, no RPC endpoint, no IPC channel, no shared memory, no signal handler, and no named pipe. Input is accepted only from the local event loop (physical keyboard and pointer).

A compromised machine can passively observe the chamber (DMA read, framebuffer capture, keystroke interception) but cannot inject commands or extract data remotely. After burn, passive observation yields only ciphertext encrypted under a destroyed key.

This claim is structural. It requires no OS cooperation and no hardware trust. There is no inbound door to open.

---

## 3. Core Model

*[Retained from v1 with minor refinements]*

### 3.1 World as Primary Unit

A Chamber C is formally a tuple C = (O, T, P, K, V, L, R) where O is the objective, T is a typed object set, P is a finite primitive algebra, K is a capability system, V is a view set, L is a lifecycle law, and R is a preservation law. This tuple defines a bounded, temporary world.

### 3.2 Typed Objects with Constrained Payloads

Objects within a world are typed according to a grammar. Permitted payload classes include bounded scalar text, bounded structured text, symbolic labels, and bounded relational references. Opaque binary payloads, external-blob payloads, and encodings intended to smuggle arbitrary binary content — such as embedding arbitrary files as Base64 text — are excluded at the schema level.

### 3.3 Primitive Algebra

The primitive algebra P is a closed set. In the simplest form it contains creation, linking, challenge, contradiction generation, ranking, synthesis, condensation, sealing, and burn invocation. The algebra is finite and substrate-defined; it is not extended on the fly by the orchestrator. The paper's world-first claim depends on this closure property.

**Implementation note (v2):** The implemented algebra contains nine primitives: create_object, link_objects, challenge_object, generate_alternative, rank_set, synthesize_set, condense_object, seal_artifact, trigger_burn. The Primitive enum in the implementation has no extensibility variant. All world evolution passes through a closed interpreter with a five-check validation pipeline: world-scope correctness, type compatibility, capability possession, lifecycle legality, and preservation-law legality.

### 3.4 Preservation Law

The preservation law R maps object classes to survivor status. Formally, R partitions object classes into preservable and non-preservable sets, with the additional constraint that some preservable classes may become preservable only after lifecycle conditions are satisfied. Preservation is therefore a legality question, not merely a storage choice.

### 3.5 Burn

Burn is not equivalent to process exit or deletion of a guest image. Burn is the composite event that terminates world existence across logical, cryptographic, storage, memory, and semantic layers. Burn is defined over world-state, not over infrastructure only.

### 3.6 Semantic Residue

A system state S after termination exhibits semantic residue with respect to a world W if an evaluator with post-termination access to S can recover interpretable information about non-preserved world-state beyond what the preservation law permits.

The taxonomy of semantic residue includes: content residue (surviving payload fragments), structural residue (surviving graph topology), behavioral residue (evidence of which operations were performed), metadata residue (timestamps, counts, lifecycle traces), and model residue (orchestrator-side KV-cache, prompt context remnants).

### 3.7 Convergence and Finalization Authority

Convergence is defined as a grammar-relative property of a world-state such that: (i) the world contains an artifact candidate permitted by the preservation law; (ii) all mandatory object classes have been resolved, discharged, or marked irreducibly open; and (iii) no preservation-blocking contradiction or dependency remains.

Finalization is modeled as a three-part process. First, the finalizer proposes convergence. Second, the substrate validates the proposal. Third, a human or policy authority authorizes sealing. The model rejects unilateral model authority over preservation.

A world may terminate in one of three modes: converged-preserving termination, converged-total-burn termination, or abort burn. Abort burn is a first-class lifecycle outcome.

---

## 4. Chambers vs. Disposable VMs — Revised

*[Updated from v1 with implementation findings]*

| Axis | Disposable VM | Chamber |
|------|--------------|---------|
| Unit of destruction | Infrastructure object: guest image, virtual disk | Typed world graph with known-temporary entities |
| Legal execution surface | Broad general-purpose semantics inside guest | Finite primitive algebra over typed objects only |
| Preservation boundary | Anything could in principle be saved | Survival is a narrow, formal exception |
| Mental model | Computer inside the computer | Bounded world with explicit lifecycle law |
| Comparison metric | Isolation strength, hypervisor maturity | Semantic residue, lifecycle legibility |
| Host relationship | Guest within hypervisor within host OS | World within dedicated substrate runtime |

**Implementation finding (v2):** A seventh delta emerged from implementation:

| Axis | Disposable VM | Chamber |
|------|--------------|---------|
| **Security model** | **Security-by-cleanup**: run stuff, delete the container, hope nothing leaked | **Security-by-construction**: the environment has no capability to leak. No filesystem, no network, no shell. The only thing that can happen is chamber law. |

This distinction is critical. A disposable VM is generic — it can do anything, so it must be cleaned up after. A chamber is purpose-built — it can only do what the grammar allows, so there is nothing to clean up beyond the cryptographic erasure.

---

## 5. Technical Architecture

*[Retained from v1]*

### 5.1 Layered Architecture

The architecture is understood in six layers: Layer 0 (immutable roots), Layer 1 (mutable platform), Layer 2 (verified substrate image), Layer 3 (chamber services), Layer 4 (orchestration layer), and Layer 5 (world instances).

### 5.2 Substrate Components

Ten substrate services define, render, constrain, and destroy worlds: World Engine, Object Engine, Operation Engine, Policy Engine, Capability System, State Engine, View Engine, Artifact Vault, Audit Layer, and Burn Engine.

### 5.3 Epoch-Based Capability Revocation

Capabilities are modeled as both world-scoped and epoch-scoped. A world epoch is a monotonic lifecycle index. Capability tokens are valid only while both the world identifier and epoch identifier remain current. When the lifecycle controller advances the world from one phase to another, it may increment the epoch and invalidate capabilities not re-issued under the new phase.

### 5.4 Burn Engine — Six-Layer Destruction

*[Updated from v1 — original paper described 5 layers. Implementation added a 6th (audit burn) after the audit layer was identified as a metadata residue channel.]*

Cryptographic burn is the primary security primitive. Physical overwrite is secondary hygiene.

1. **Logical** — Capability revocation, handle invalidation, namespace retirement, graph reachability breakage.
2. **Cryptographic** — Destroy K_w. Invalidate unwrap path from K_s. Retained ciphertext becomes unrecoverable.
3. **Storage** — Delete or dereference temporary objects, indexes, caches, live-world blocks.
4. **Memory** — Zero or drop in-memory world-state and transient orchestrator context.
5. **Audit burn** — Destroy world-scoped audit events (Tier 2: phase transitions, convergence, sealing records). Only substrate-scoped events survive (Tier 1: WorldCreated + WorldDestroyed — 2 entries revealing existence only).
6. **Semantic** — Prevent reconstruction of non-preserved world-state from whatever remains.

Reference key hierarchy: at world creation, the substrate generates a fresh world key K_w, optionally wrapped under a substrate-held sealing key K_s. Live world-state is encrypted under K_w; artifact sealing uses a distinct artifact key K_a. Burn destroys K_w and invalidates any unwrap path before storage hygiene completes.

---

## 6. The Native Execution Requirement

*[New section — v2]*

### 6.1 Why a Browser Tab Defeats the Architecture

Early implementation used a browser-based UI. This revealed that the browser is an outbound channel:

- **Browser history** records that the chamber existed and what URLs were visited
- **Back/forward buttons** allow navigation to a state that should have been destroyed
- **Browser cache** persists page content to disk
- **Browser extensions** have access to page content
- **Developer tools** expose the DOM, network requests, and JavaScript state
- **Browser process memory** is shared with other tabs
- **Tab restore** on crash recreates destroyed state

The chamber cannot run inside a browser. It must run as a native application with its own window, its own process, and its own lifecycle.

### 6.2 The System Cursor Problem

Implementation revealed that system-level affordances are residue channels. The macOS cursor, keyboard shortcuts, right-click menu, spell check, and autocorrect all belong to the host system. They cross the chamber boundary in both directions — they inject system behavior into the chamber (spell check, autocorrect) and extract chamber content to the system (clipboard via Cmd+C, screenshots via Cmd+Shift+3).

The solution is to sever all system affordances inside the chamber:

- System cursor hidden; chamber renders its own pointer via DOM
- All system keyboard shortcuts intercepted and blocked
- Right-click context menu prevented
- Spell check, autocorrect, autocomplete disabled
- Drag-and-drop from outside prevented
- The chamber presents its own interaction primitives that exist only inside the world

This is a UX-security finding that does not appear in the prior literature on privacy-preserving systems: **system-level interaction affordances are trust boundary violations**.

### 6.3 Fullscreen Isolation

The chamber takes over the entire screen when opened. No dock, no menu bar, no other windows visible. The design intent is that the user perceives the chamber as a bounded environment, not as another application. When the chamber burns, the fullscreen vanishes and the desktop returns. The transition is immediate. Note that OS-level processes (Activity Monitor, screen capture by other apps) can still observe the chamber's existence; the fullscreen model addresses the user's mental model, not the OS's awareness.

---

## 7. Encrypted Memory Pool

*[New section — v2]*

### 7.1 The During-Operation Exposure Problem

The original paper describes cryptographic burn: destroy K_w after termination, making ciphertext unrecoverable. This addresses post-burn security. It does not address during-operation exposure.

During chamber operation, world state exists in plaintext in RAM. A DMA attacker, a memory-scanning process, or a cold-boot attack can read this plaintext for the entire duration of the session.

### 7.2 Decrypt-on-Read Architecture

The solution is to keep world state encrypted in RAM at all times. Plaintext exists only in a single guarded buffer for the microsecond of active use.

**Encrypted object store:** replaces the plaintext HashMap with an encrypted store. Every object is serialized, encrypted with K_w (AES-256-GCM), and stored as a ciphertext blob. The in-memory object graph holds only encrypted data.

**Guard buffer:** a single page-aligned, mlock'd buffer (4KB). When an operation reads an object: decrypt into the guard buffer, use the plaintext, zeroize the buffer. Only one object is ever decrypted at a time.

**Scoped access API:** the Rust borrow checker enforces the decrypt-use-zero cycle at compile time. The plaintext reference cannot escape the closure. There is no way to hold decrypted data outside the guarded scope.

```
world.with_object_decrypted(object_id, |plaintext| {
    // Use it here. Cannot return, store, or copy the reference.
    // When this closure returns, the guard buffer is zeroed.
});
```

### 7.3 Impact on Residue

With the encrypted memory pool, a DMA snapshot of process memory shows only ciphertext and at most one object fragment in the guard buffer (if captured at the exact microsecond of active decryption). The exposure window drops from "entire session duration" to "microseconds per access." After burn, K_w is zeroed and the entire encrypted store becomes unrecoverable, even from a full memory image taken before burn.

---

## 8. AI as a Constrained Orchestrator — Revised

*[Significantly expanded from v1]*

### 8.1 Original Position (retained)

The orchestration layer is software. Chambers does not claim the absence of software performing an orchestration-like role; rather, it claims that orchestration software is not the primary semantic and persistence unit. The orchestrator maps objectives to admissible world evolution under substrate law.

The architecture is meaningful even if the orchestration layer is replaced by a symbolic planner or a smaller model rather than a large language model.

### 8.2 Chamber-Born Model Instances

*[New — v2]*

Implementation introduces a model that is born inside the chamber and dies with it.

Every time a chamber opens, a model instance is created inside the chamber's execution context. The model's weights are loaded into memory. Its context window, KV cache, and attention state exist only inside the chamber's encrypted memory space. When the chamber burns, the model dies with it. The next chamber gets a fresh model with zero memory of prior sessions.

**What is substrate-scoped (not burned):**
- Model weights (GGUF file). These are infrastructure — the same model is loaded into every chamber. Weights contain no chamber-specific information.

**What is world-scoped (burned):**
- KV cache — the model's working memory of this conversation
- Attention state — what the model is attending to
- Sampling state — active generation parameters
- Context window — the accumulated prompt and generation
- Any chain-of-thought or scratchpad buffers

All world-scoped inference state is allocated inside the encrypted memory pool, encrypted under K_w, decrypted only in the guard buffer during active inference, and zeroed on burn.

### 8.3 What Separates This from Cloud LLMs

| | Cloud LLM | Chamber-born LLM |
|---|---|---|
| Memory | Persists across sessions | Born blank, dies on burn |
| Context | Stored on provider servers, potentially logged, potentially trained on | Encrypted in chamber RAM, zeroed on burn |
| Weights | Provider-controlled | Substrate-scoped, loaded locally |
| What survives | Everything — conversation, model state, provider logs | Only explicitly sealed artifacts |
| Cross-session leakage | Possible — context windows, training data, cross-user contamination | Impossible — model instance is dead, next chamber gets clean birth |
| Provider sees your data | Yes | No provider exists. No server. No log. |

### 8.4 Model Context as World-State (formal requirement)

If an LLM is used as the orchestrator:
- Context window = world-state. Encrypted under K_w, burned with the world.
- KV cache = world-state. Same treatment.
- Model weights ≠ world-state. Substrate-scoped infrastructure. Not burned.
- No hidden scratch state outside world law.
- No cross-chamber learning. The model does not fine-tune or accumulate feedback from chamber content.

---

## 9. Chamber Boot

*[New section — v2]*

### 9.1 The Problem with Running Inside the Host OS

A chamber that runs as a regular macOS process shares the operating environment with hundreds of other processes. Spotlight can index its activity. Activity Monitor shows its resource usage. The window server, clipboard daemon, and IOKit device registry are shared. The chamber is pretending to be isolated while sitting in a shared environment.

### 9.2 The Hypervisor Boot Model

True isolation requires that opening a chamber is a boot, not an app launch. Using Apple's Hypervisor.framework, the chamber spawns a minimal VM containing:

- A microkernel (just enough to manage memory and run the substrate)
- The substrate runtime (statically linked Rust binary)
- An inference engine (llama.cpp, statically linked)
- Model weights (read-only)
- Framebuffer output and keyboard/mouse input
- Nothing else — no shell, no filesystem, no network stack, no sshd, no package manager

The host-VM communication channels are:
- **Framebuffer** (VM → host): display output only
- **Keyboard/mouse** (host → VM): input events only
- **Virtio control** (host → VM): start/stop signals only

No shared memory, no shared filesystem, no network bridge, no clipboard sharing. The VM is an opaque box that accepts keystrokes and emits pixels.

### 9.3 Security-by-Construction vs. Security-by-Cleanup

A disposable VM is security-by-cleanup: run stuff in a generic environment, delete the container, hope nothing leaked.

A chamber boot is security-by-construction: the VM has no capability to leak. There is no shell to escape to, no filesystem to write to, no network to exfiltrate through. The only thing that can happen inside is chamber law. There is nothing to clean up beyond the cryptographic erasure because there was never anywhere for data to go.

### 9.4 Burn = VM Destruction

1. Substrate executes six-layer burn inside the VM (logical → cryptographic → storage → memory → audit burn → semantic)
2. K_w is zeroed inside the VM
3. VM sends "burn complete" signal via virtio control
4. Host tells the hypervisor to destroy the VM
5. All VM memory pages are freed (returned to host physical memory pool)
6. All VM memory pages are freed. No disk image was created. Note: the host retains awareness that a VM was launched (process accounting, hypervisor logs) — "no trace" refers to world content, not existence metadata

---

## 10. Comparative Trace Analysis

*[Updated v3 — reframed from "hypothesis test" to "comparative trace analysis" per peer review. Baselines updated from simulated to real infrastructure.]*

### 10.1 Methodology

The substrate was compared against two real baselines using a canonical decision task (cloud provider selection for HIPAA workloads), 3 runs per condition:

- **Disposable VM baseline**: real temp directory on APFS, real file I/O, real `rm -rf` deletion, real post-deletion residue scan (macOS unified log, filesystem metadata, Spotlight, content grep).
- **Docker microVM baseline**: real ephemeral Docker container (`alpine:latest`, `--rm`, `--network none`, `--memory 64m`, `--read-only`, `--tmpfs /data`). Real post-destruction scan (Docker event log, daemon metadata, unified log, image layer cache).

All conditions performed the same task with the same input data. Residue was measured after destruction.

**Scope note.** The design-space analysis (Sections 4, 9) positions Chambers against 13 systems across 11 axes. The empirical section tests against 2 baselines. This asymmetry is intentional: the positioning is design-space analysis (where does Chambers sit relative to the field?), while the empirical section tests the most operationally relevant baselines (filesystem deletion and container destruction, which are the most common real-world "ephemeral computing" patterns). Testing against Qubes disposables, Firecracker, gVisor, and Tails would strengthen the empirical section but requires infrastructure beyond the current test environment. The positioning claims are architectural arguments; only the empirical findings are presented as measured results.

**Observer effect note**: the unified log scanner (`log show`) is itself logged by the macOS unified log, creating one metadata entry per scan. This observer-effect entry is documented but excluded from intrinsic residue counts. The scanner was checked for secondary traces (Spotlight, .DS_Store, filesystem journals) — none were found.

### 10.2 Results

| Condition | Content Residue | Structural Residue | Existence-Level Metadata | Reconstruction |
|-----------|----------------|-------------------|-------------------------|----------------|
| **Chambers** | Zero (encrypted, K_w destroyed) | Zero (graph burned) | 2 (declared: WorldCreated + WorldDestroyed) | Infeasible (AES-256, key destroyed) |
| Disposable VM | Zero (files deleted) | Zero (directory removed) | 2 (1 intrinsic + 1 observer effect) | Feasible from surviving traces |
| Docker microVM | Zero (tmpfs freed) | Zero (container removed) | 5.3 (~4.3 intrinsic + 1 observer effect) | Feasible from surviving traces |

### 10.3 Findings

**Finding 1: Content and structural residue.** All three conditions achieve zero content and structural residue at the object level. Files are deleted, objects are destroyed, graph structures are removed. This is not a differentiator — all three approaches successfully destroy their data.

**Finding 2: Existence-level metadata (predictability, not count).** All three conditions retain existence-level metadata — traces revealing "something happened" without revealing content. Chambers retains 2 events (WorldCreated + WorldDestroyed). The VM retains 1 intrinsic trace (/tmp modification timestamp). Docker retains ~4.3 intrinsic traces (event log lifecycle + metadata store + image cache).

The difference is not primarily in count but in **predictability**. Chambers' metadata is declared in advance by the grammar — the user knows exactly what will remain before the chamber is created. The baselines' metadata is incidental and unpredictable — a user cannot enumerate what the host OS or Docker daemon will retain. Tier 1 events contain no operation counts, object counts, or per-operation timestamps — only that a world existed and was destroyed.

This is a lifecycle-legibility advantage, not a residue-count advantage.

**Finding 3: Reconstruction feasibility.** Chambers reconstruction is structurally infeasible — K_w is destroyed, and all world state was encrypted under K_w (AES-256-GCM). Without the key, the retained ciphertext is computationally unrecoverable regardless of time or resources. Baseline reconstruction is feasible in principle from surviving traces (filesystem timestamps, Docker event logs), though no reconstruction protocol was executed to measure actual time.

**Finding 4: Structural predictability of preservation (H2 structural analysis).** An automated analysis with synthetic participants at five knowledge levels shows that Chambers' explicit grammar enables users at the InformedUser level and above to achieve perfect prediction accuracy (F1 = 1.000) for what survives burn. No equivalent declaration exists for VM or Docker baselines — users cannot predict what host-level traces the OS or daemon will retain, and even Expert-level participants report lower confidence. This is a structural property of formal preservation law, not a measured human outcome. **Validation with human participants is pending.**

**Finding 5: Docker residue exceeds simulation.** The simulated microVM baseline predicted 4 metadata entries; real Docker produced 5.3 — the Docker daemon's event logging system (create → attach → start → die → destroy) is a genuine residue channel the simulation missed. The base image cache (`alpine:latest`, 13.6MB) persists indefinitely on the host.

**Baseline proxy sufficiency note.** Real Firecracker microVM baselines require Linux/KVM and were not feasible on the test hardware (macOS/Apple Silicon). Docker with `--rm --network none --read-only --tmpfs /data` is a reasonable proxy: both provide ephemeral execution with no persistent storage. The key difference is that Firecracker has a smaller host-side footprint (no Docker daemon, no image layer cache). A real Firecracker baseline would likely show fewer metadata entries than Docker (closer to the VM baseline) but more than Chambers (the host still retains VM lifecycle metadata). This remains future work.

---

## 11. Limitations

The model is narrow. Without ordinary import/export and without a general application model, it addresses only a bounded class of privacy-sensitive computation. The substrate is persistent and therefore the central object of trust. The orchestration section remains contingent on implementation discipline.

**Additional limitations identified in implementation (v3):**

- **OS-level visibility cannot be prevented.** macOS logs that the app launched. Activity Monitor shows resource usage during the session. These reveal that a chamber existed, though not what was inside (see Section 2.2 for full "not claimed" list).
- **Framebuffer capture is not preventable from user space.** A co-resident process with screen capture permission can see the rendered UI.
- **Keystroke interception is not preventable from user space.** A keylogger captures what the user typed. System-generated state is not captured.
- **Apple Silicon mitigates but does not eliminate DMA.** The IOMMU blocks external DMA devices, but a compromised internal controller could observe K_w in mlock'd memory. The DMA exposure window has not been measured with hardware attack equipment (e.g., PCILeech) — this is future work.
- **The encrypted memory pool is software-enforced, not hardware-enforced.** A root-level attacker who reads K_w from process memory can decrypt the encrypted store. Hardware-grade memory encryption (SEV/TDX/SGX) provides stronger guarantees not achievable in user space.
- **Model weights are not burned.** They are substrate-scoped infrastructure. Weights are static and pre-trained; they should not contain chamber-specific information.
- **Baseline measurements use proxies.** Docker containers serve as microVM proxies; real Firecracker baselines require Linux/KVM and were not feasible on test hardware. Reconstruction times are estimates from surviving traces, not measured reconstruction attempts.
- **H2 (lifecycle comprehension) is structurally validated, not empirically validated.** Synthetic participants demonstrate the structural advantage of formal preservation law; human validation is pending.
- **"Zero undeclared residue" is the accurate claim, not "zero residue."** Chambers retains 2 declared existence-level events (WorldCreated + WorldDestroyed). These are by design and documented in the grammar.

---

## 12. Conclusion — Revised

Chambers proposes a different unit of privacy-preserving computation. Rather than securing reusable app-centric environments more aggressively, it treats a bounded temporary world as the primary semantic unit. The substrate defines what may exist, what may happen, what may survive, and how destruction occurs. The orchestrator may arrange the world, but it may not invent its laws.

Comparative trace analysis against real baselines shows: zero undeclared residue for Chambers versus incidental metadata retention for baselines; structural infeasibility of reconstruction for Chambers versus feasible reconstruction for baselines; and a structural predictability advantage from explicit preservation law (pending human validation).

Implementation also reveals requirements the original paper did not anticipate:

1. **The chamber must not run inside a browser.** Browser history, cache, extensions, and shared process space are outbound channels. The chamber requires its own native execution context.
2. **System-level interaction affordances are trust boundary violations.** The cursor, keyboard shortcuts, spell check, and clipboard belong to the host system. The chamber must sever them and present its own.
3. **Plaintext should not persist in RAM for the session lifetime.** The encrypted memory pool with decrypt-on-read and a guarded buffer reduces the DMA exposure window from the entire session to microseconds per object access.
4. **The chamber should boot, not launch.** A hypervisor-based execution context (VM with no filesystem, no network, no shell) provides security-by-construction rather than security-by-cleanup.
5. **An LLM can be born inside the chamber and die with it.** Model inference state is world-scoped, encrypted under K_w, and zeroed on burn. No cross-chamber memory. No cloud dependency. No provider sees the data.

The paper's claim remains intentionally narrow. Chambers is not a universal replacement for VMs, amnesic operating systems, or hardened platforms. It occupies a distinct position in the design space: bounded ephemeral cognition with formal lifecycle law. It is complementary to, not competitive with, existing privacy tools — Chambers could run inside a Tails session (for anonymity), inside a Qubes disposable (for OS compartmentalization), or on hardware with SEV/TDX (for hardware-backed memory encryption). The comparative trace analysis suggests genuine advantages on the axes the architecture targets: lifecycle legibility, preservation predictability, and semantic residue structure.

---

## References

[1] Tails. Official documentation. https://tails.net/
[2] Qubes OS Documentation. Disposable qubes. https://doc.qubes-os.org/en/latest/user/how-to-guides/how-to-use-disposables.html
[3] GrapheneOS Features Overview. https://grapheneos.org/features
[4] NIST SP 800-193, Platform Firmware Resiliency Guidelines. https://csrc.nist.gov/pubs/sp/800/193/final
[5] NIST SP 800-218, Secure Software Development Framework. https://csrc.nist.gov/pubs/sp/800/218/final
[6] Apple Platform Security, Hardware Security Overview. https://support.apple.com/en-ie/guide/security/secf020d1074/web
[7] Apple Platform Security, Secure Enclave. https://support.apple.com/en-ie/guide/security/sec59b0b31ff/web
[8] Microsoft, Windows UEFI Firmware Update Platform. https://learn.microsoft.com/en-us/windows-hardware/drivers/bringup/windows-uefi-firmware-update-platform
[9] Microsoft, Kernel DMA Protection for Thunderbolt. https://learn.microsoft.com/en-us/windows/security/hardware-security/kernel-dma-protection-for-thunderbolt
[10] Intel SA-00075 and mitigation guidance. https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00075.html
[11] Watson et al. "Capsicum: practical capabilities for UNIX." USENIX Security 2010.
[12] Murray et al. "seL4: From General Purpose to a Proof of Information Flow Enforcement." IEEE S&P 2013.
[13] Agache et al. "Firecracker: Lightweight Virtualization for Serverless Applications." NSDI 2020.
[14] gVisor Documentation and Architecture Guide. https://gvisor.dev/docs/
[15] Peterson et al. "Secure Deletion for a Versioning File System." FAST 2005.
[16] Reardon et al. "Data Node Encrypted File System: Efficient Secure Deletion for Flash Memory." USENIX Security 2012.
[17] Wei et al. "Reliably Erasing Data From Flash-Based Solid State Drives." FAST 2011.
[18] Dennis and Van Horn. "Programming Semantics for Multiprogrammed Computations." CACM 1966.
[19] Signal Protocol. Double Ratchet Algorithm. https://signal.org/docs/specifications/doubleratchet/
[20] Apple Hypervisor.framework documentation. https://developer.apple.com/documentation/hypervisor
[21] MirageOS. Library operating system for constructing unikernels. https://mirage.io/
[22] Google Cloud Confidential VMs. SEV-backed encrypted VM instances. https://cloud.google.com/confidential-computing
[23] AMD SEV-SNP. Secure Encrypted Virtualization — Secure Nested Paging. https://www.amd.com/en/developer/sev.html
[24] Intel TDX. Trust Domain Extensions. https://www.intel.com/content/www/us/en/developer/tools/trust-domain-extensions/overview.html
[25] ARM CCA. Confidential Compute Architecture. https://www.arm.com/architecture/security-features/arm-confidential-compute-architecture
