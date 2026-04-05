# Competitive Analysis — Chambers vs. Existing Privacy Systems

**Date:** 2026-04-05 (revised)

This document compares Chambers (Phase 0 + Level 1 + Phase 2) against existing privacy-preserving systems to understand where Chambers sits in the design space.

**This comparison illuminates design-space positioning, not competitive superiority.** Chambers is not a replacement for these tools. Different threat models require different tools. Tails provides anonymity. Qubes provides compartmentalization. Firecracker provides lightweight VM infrastructure. Chambers provides bounded ephemeral cognition with formal lifecycle law. They are complementary, not competing.

**Configuration note:** Chambers is compared in its Phase 2 hardened state. Competitors are compared in their default configurations unless noted. Some competitors (Qubes with anti-evil-maid, Tails on storage-free hardware, Firecracker with jailer) can be significantly hardened beyond their defaults. Where relevant, hardened configurations are noted.

---

## Systems Compared

| System | Category | What it does |
|--------|----------|-------------|
| **Chambers (Phase 2)** | World-first runtime | Sealed ephemeral worlds with typed objects, burn semantics, encrypted memory |
| **Tails** | Amnesic live OS | Boots from USB, routes through Tor, leaves no trace on host |
| **Qubes OS** | Compartmentalized VMs | Disposable qubes (VMs) for isolation, template-based |
| **GrapheneOS** | Hardened mobile OS | Hardened Android with sandboxing, exploit mitigations |
| **Firecracker** | MicroVM | Lightweight VMs for serverless (AWS Lambda), minimal attack surface |
| **gVisor** | Application kernel | User-space kernel that intercepts syscalls, sandboxes containers |
| **Docker (ephemeral)** | Container | Ephemeral containers with no persistent volume |
| **Whonix** | Privacy OS | Split gateway/workstation, Tor-routed, VM-based |
| **Standard VM** | Disposable VM | VirtualBox/VMware/QEMU snapshot → use → delete |
| **Private browsing** | Browser mode | Incognito/InPrivate — no history, no cookies after close |
| **AMD SEV / Intel TDX** | Confidential computing | Hardware memory encryption for VM guests — hypervisor cannot read guest RAM |
| **Intel SGX** | Trusted execution environment | Hardware enclaves — isolated memory region, encrypted, attestable |
| **ARM CCA** | Confidential computing | ARM Realms — hardware-isolated execution contexts on ARM |
| **Signal Protocol** | Secure messaging | Double Ratchet — forward-secret session keys, per-message key evolution |

---

## Axis 1: Semantic Residue After Destruction

*What recoverable information remains after the environment is destroyed?*

| System | Content residue | Structural residue | Metadata residue | Model/context residue | Overall |
|--------|----------------|-------------------|-----------------|----------------------|---------|
| **Chambers** | **Zero** — encrypted under K_w, key destroyed | **Zero** — graph encrypted, wiped | **2 events** (existed + destroyed) | **Zero** — inference state is world-scoped, burned | **0.0 score** |
| Tails | Low — RAM-only, but swap may leak | None if no persistence | Host BIOS timestamps, USB device history | N/A | Low |
| Qubes OS | Low — VM image deleted | Possible filesystem journal entries on host | Host dom0 logs, Xen traces | N/A | Low-Medium |
| GrapheneOS | Medium — app data may persist in flash wear-leveling | SQLite journals, app caches | System logs, usage stats | N/A | Medium |
| Firecracker | Low — guest memory freed | No disk (if RAM-only) | Host-side: jailer logs, cgroup entries, API socket traces | N/A | Low |
| gVisor | Low — process memory freed | Possible sentry logs | Host-side: runsc logs, gofer traces | N/A | Low-Medium |
| Docker | Medium — layer cache, build cache, overlay remnants | Image layers on host | Docker daemon logs, container metadata | N/A | Medium |
| Whonix | Low — similar to Qubes disposable | Gateway VM retains Tor circuit info | Host dom0 logs | N/A | Low-Medium |
| Standard VM | **Medium** — disk image deleted but host FS journal, swap, temp files remain | Block-level remnants on host disk | Hypervisor logs, creation timestamps, resource usage | N/A | **Medium** |
| Private browsing | **High** — DNS cache, OS page cache, favicon cache, GPU process memory, extensions | Tab structure in process memory | OS process accounting, network logs | N/A | **High** |

**Finding:** Chambers is the only system that achieves zero content and structural residue through cryptographic erasure. All others rely on deletion (filesystem remove, memory free) which is best-effort on modern storage.

---

## Axis 2: Lifecycle Legibility

*Can the user understand and predict what will survive vs. be destroyed?*

| System | Lifecycle model | User can predict survivors? | Preservation is explicit? |
|--------|----------------|---------------------------|--------------------------|
| **Chambers** | **Formal**: Created → Active → Convergence → Finalization → Burn. Preservation law declared in grammar. | **Yes** — grammar declares exactly what can survive (only `decision_summary`) | **Yes** — requires human authorization to seal |
| Tails | Boot → use → shutdown = everything gone | Mostly — but users don't know what Tor circuits are cached in RAM | No — everything is equally ephemeral |
| Qubes OS | Create disposable → use → close = VM deleted | Somewhat — users know the VM is gone, but don't know what dom0 retains | No — no selective preservation |
| GrapheneOS | Install app → use → uninstall = app data removed | No — users don't know about flash wear-leveling, SQLite WAL, or system caches | No |
| Firecracker | API call → VM runs → API call → VM destroyed | Yes for developers, no for end users | No |
| gVisor | Container starts → runs → stops → removed | Similar to Docker — developers understand, users don't | No |
| Docker | Container starts → runs → stops → rm | Developers understand layers; users do not | No |
| Whonix | Boot → use → shutdown | Similar to Tails + Qubes complexity | No |
| Standard VM | Create → snapshot → use → revert/delete | Users think "delete VM = gone" but don't know about host traces | No |
| Private browsing | Open window → browse → close window | Users think "incognito = invisible" — widely misunderstood | No |

**Finding:** Chambers is the only system with a formal lifecycle model where the user can precisely predict what survives. Every other system has implicit, opaque destruction semantics.

---

## Axis 3: Preservation Narrowness

*How narrow is the channel for data to survive destruction?*

| System | What can survive | Channel width |
|--------|-----------------|--------------|
| **Chambers** | **Only** explicitly sealed artifacts of a declared preservable class, with human authorization | **1 object class, 1 channel (vault), requires authorization** |
| Tails | Optionally: persistent volume (if configured) — arbitrary files | Wide (any file, if persistence enabled) |
| Qubes OS | Anything in a non-disposable qube; clipboard contents | Wide |
| GrapheneOS | Any app data, system settings, downloads | Very wide |
| Firecracker | Whatever the API caller captures from stdout/logs | Medium (API-defined) |
| gVisor | Whatever the container runtime captures | Medium |
| Docker | Named volumes, bind mounts, image layers | Wide |
| Whonix | Gateway state, any non-disposable qube data | Wide |
| Standard VM | Exported files, screenshots, host-side copies | Wide (user-dependent) |
| Private browsing | Downloaded files, bookmarks (in some browsers), passwords (if saved) | Medium |

**Finding:** Chambers has the narrowest preservation channel of any system. One object class, one authorized path, substrate-enforced.

---

## Axis 4: Legal Execution Surface

*How constrained is what can happen inside the environment?*

| System | What can execute | Constraint model |
|--------|-----------------|-----------------|
| **Chambers** | **9 primitives only**. No shell, no arbitrary code, no file I/O, no network. Grammar defines permitted operations per lifecycle phase. | **Closed algebra, substrate-enforced** |
| Tails | Full Linux desktop — any application, any syscall | Open — general-purpose OS |
| Qubes OS | Full OS per qube — any application | Open per qube |
| GrapheneOS | Any Android app (with hardened sandbox) | Open — app sandbox only |
| Firecracker | Full Linux kernel in guest — any code | Open inside guest |
| gVisor | Filtered syscalls (~200 supported) — any compatible code | Semi-constrained (syscall filter) |
| Docker | Full Linux userspace — any code | Open (unless seccomp/AppArmor) |
| Whonix | Full desktop environment + Tor | Open |
| Standard VM | Full OS — anything | Open |
| Private browsing | Full web platform — JS, WebAssembly, WebRTC, etc. | Open (browser sandbox only) |

**Finding:** Chambers is the only system with a closed, finite execution surface. All others allow arbitrary computation.

---

## Axis 5: Memory Protection (During Operation)

*Is data encrypted in RAM while the environment is active?*

| System | RAM encryption | Key management | DMA protection |
|--------|---------------|----------------|----------------|
| **Chambers (Phase 2)** | **Yes** — all objects/links encrypted under K_w in RAM. Plaintext only in guard buffer for microseconds. | K_w per world, mlock'd, zeroized on burn | IOMMU (Apple Silicon) + encrypted store limits exposure |
| Tails | No — plaintext in RAM | N/A | No specific protection |
| Qubes OS | No — plaintext in guest RAM | Xen manages guest pages | Xen IOMMU for PCI passthrough |
| GrapheneOS | No application-level — relies on hardware (Titan M, ARM MTE) | Hardware-managed | ARM SMMU |
| Firecracker | No — plaintext in guest RAM | KVM manages pages | IOMMU if configured |
| gVisor | No — sentry process has plaintext | N/A | No |
| Docker | No | N/A | No |
| Whonix | No | N/A | Same as Qubes |
| Standard VM | No | Hypervisor manages pages | Depends on hypervisor + IOMMU config |
| Private browsing | No | N/A | No |
| **AMD SEV / Intel TDX** | **Yes** — hardware encrypts ALL guest memory. Hypervisor cannot read guest RAM. | Firmware-managed, hardware-enforced. Key never in software. | Full — hardware IOMMU + memory encryption |
| **Intel SGX** | **Yes** — enclave memory encrypted by CPU. | Hardware-managed per-enclave. | Full within enclave boundary |
| **ARM CCA** | **Yes** — Realm memory encrypted by hardware. | Hardware-managed. | Full within Realm |
| **Signal Protocol** | N/A (messaging, not computing) | Per-message key ratchet. Old keys deleted. | N/A |

**Finding:** Chambers is the only *software-only* system that encrypts data in RAM during operation. However, this is a weaker guarantee than hardware-backed encryption:

| Property | Chambers (Phase 2) | AMD SEV / Intel TDX | Intel SGX |
|----------|-------------------|--------------------|-----------| 
| What's encrypted | Objects/links (not K_w itself) | ALL guest memory | Enclave memory only |
| Key location | K_w in mlock'd process RAM — readable by DMA | Firmware/hardware — never in software | CPU-internal — never in RAM |
| Can hypervisor/host read it? | Root can bypass ptrace and read K_w | **No** — hardware enforced | **No** — hardware enforced |
| Attestation | None (Phase 3) | Yes — remote attestation | Yes — remote attestation |

**Honest assessment:** Chambers' encrypted memory pool is defense-in-depth, not a hardware-grade guarantee. A root-level attacker who bypasses ptrace can read K_w from process memory and decrypt everything. SEV/TDX/SGX prevent this at the hardware level. Phase 3 (Secure Enclave for K_s) narrows this gap but does not close it fully — K_w still exists in RAM during operation.

---

## Axis 6: Clipboard Isolation

| System | Clipboard crosses boundary? | Isolated clipboard? |
|--------|---------------------------|-------------------|
| **Chambers** | **No** — system pasteboard never read or written. Chamber has its own clipboard, zeroed on burn. | **Yes** |
| Tails | Yes — X11 clipboard shared between apps | No |
| Qubes OS | Controlled — inter-qube clipboard requires explicit Ctrl+Shift+C/V | Partial |
| GrapheneOS | Yes — system clipboard shared | No |
| Firecracker | N/A (no GUI) | N/A |
| gVisor | N/A (no GUI) | N/A |
| Docker | N/A (no GUI typically) | N/A |
| Whonix | Same as Qubes | Partial |
| Standard VM | Guest tools may share clipboard | Depends on config |
| Private browsing | **Yes** — clipboard shared with all tabs and apps | No |

**Finding:** Chambers and Qubes are the only systems with clipboard boundary control. Chambers is stricter — the system clipboard is never touched. Qubes requires manual copy between qubes.

---

## Axis 7: Hardware API Isolation

*Can the environment access camera, microphone, bluetooth, USB, sensors?*

| System | Camera/Mic | Bluetooth/USB | Sensors | Network |
|--------|-----------|---------------|---------|---------|
| **Chambers** | **Blocked** | **Blocked** | **Blocked** | **Localhost only** |
| Tails | Available (user controls) | Available | Available | Tor-routed |
| Qubes OS | Per-qube device assignment | Per-qube | Per-qube | Per-qube firewall |
| GrapheneOS | Per-app permission | Per-app | Per-app | Available |
| Firecracker | Not exposed (no device passthrough by default) | Not exposed | Not exposed | virtio-net only |
| gVisor | Filtered syscalls — most devices blocked | Blocked | Blocked | Available (filtered) |
| Docker | Available (unless restricted) | Available (unless restricted) | Available | Available |
| Whonix | Available in workstation qube | Available | Available | Tor-routed |
| Standard VM | Available if guest tools installed | USB passthrough available | Not typically | Available |
| Private browsing | **Available** — same permissions as normal tabs | Available | Available | Available |

**Finding:** Chambers and Firecracker are the most restrictive. Chambers blocks everything at the application layer. Firecracker doesn't expose devices by design (no guest tools). The key difference: Firecracker is a VM infrastructure tool; Chambers is a user-facing application.

---

## Axis 8: Debugger / Inspection Resistance

| System | Can external process inspect memory? | Anti-debugging? |
|--------|-------------------------------------|----------------|
| **Chambers (Phase 2)** | **ptrace denied** + encrypted memory (even if inspected, sees ciphertext) | Yes — PT_DENY_ATTACH + core dumps disabled |
| Tails | Root can inspect any process | No |
| Qubes OS | dom0 can inspect guest RAM | No (by design — dom0 is trusted) |
| GrapheneOS | Root is restricted (verified boot) | Partial — hardened against root exploits |
| Firecracker | Host can inspect guest pages via /proc/pid/mem | No |
| gVisor | Host can inspect sentry process | No |
| Docker | Host can inspect container processes | No |
| Whonix | Same as Qubes | No |
| Standard VM | Host can read guest memory via hypervisor | No |
| Private browsing | Any process can inspect browser memory | No |

**Finding:** Chambers is the only system that both denies debugger attachment AND encrypts memory so that even if inspection succeeds (root bypass), the data is ciphertext.

---

## Axis 9: Cryptographic Erasure

*Is destruction based on key destruction (cryptographic) or data deletion (filesystem)?*

| System | Erasure method | Key destroyed? | Ciphertext retained? |
|--------|---------------|----------------|---------------------|
| **Chambers** | **Cryptographic** — K_w destroyed, ciphertext unrecoverable | **Yes** — zeroized from RAM | Yes (harmless without key) |
| Tails | Filesystem — RAM contents lost on shutdown (no disk) | N/A | N/A |
| Qubes OS | Filesystem — VM image deleted | N/A (LUKS per-qube, but key not per-session) | Possible block remnants |
| GrapheneOS | Filesystem — app data directory deleted | N/A | Flash wear-leveling retains blocks |
| Firecracker | Memory — guest pages freed by KVM | N/A | No disk to retain |
| gVisor | Memory — process killed | N/A | No |
| Docker | Filesystem — overlay layers removed | N/A | Possible host disk remnants |
| Whonix | Same as Qubes | N/A | Possible |
| Standard VM | Filesystem — VM image deleted | N/A | **Host disk retains deleted blocks** |
| Private browsing | Filesystem — cache/cookies deleted from browser profile | N/A | **Browser profile dir, DNS cache, OS cache retain data** |

**Finding:** Chambers is the only system using cryptographic erasure as the primary destruction primitive. All others use filesystem deletion, which is unreliable on modern storage (SSD wear-leveling, filesystem journals, OS caches).

---

## Axis 10: Remote Access Resistance

*Can the environment be accessed remotely, even if the machine is compromised?*

| System | Inbound remote access possible? | Why |
|--------|-------------------------------|-----|
| **Chambers** | **No** — no inbound listener, no RPC, no IPC, no shared memory. Structural property. | No network socket, no service, no protocol |
| Tails | Yes — if malware gains execution, it can open a connection (despite Tor routing) | Full OS with network stack |
| Qubes OS | Yes — per-qube, if the qube has network | Full OS per qube |
| GrapheneOS | Yes — any app with network permission | Full mobile OS |
| Firecracker | Yes — guest has virtio-net | Network available |
| gVisor | Yes — filtered but functional network | Network available |
| Docker | Yes — containers typically have network | Network available |
| Whonix | Yes — through Tor (by design) | Full network via Tor |
| Standard VM | Yes — if guest has network adapter | Network available |
| Private browsing | **Yes** — full network, WebRTC, WebSocket | Full browser network stack |

**Finding:** Chambers is the only system where remote access is structurally impossible. Every other system has a network stack that could be exploited.

---

## Axis 11: Post-Destruction Reconstruction Time

*How long does it take an evaluator to reconstruct what happened inside?*

| System | Reconstruction feasibility | Estimated time |
|--------|--------------------------|----------------|
| **Chambers** | **Infeasible** — encrypted data, key destroyed | **∞** |
| Tails | Difficult — RAM gone, but USB device history, BIOS timestamps | Hours-days (forensic) |
| Qubes OS | Moderate — dom0 logs, Xen traces, disk journal | Hours |
| GrapheneOS | Moderate — flash remnants, system logs, app artifacts | Hours |
| Firecracker | Difficult if RAM-only — but host logs exist | Hours |
| gVisor | Moderate — sentry logs, runsc traces | Hours |
| Docker | Easy — image layers, build cache, daemon logs | **Minutes** |
| Whonix | Moderate — similar to Qubes | Hours |
| Standard VM | Easy-Moderate — host disk journal, temp files, swap | **Minutes-Hours** |
| Private browsing | Easy — DNS cache, OS cache, favicon DB, GPU memory | **Minutes** |

**Finding:** Chambers is the only system where reconstruction is computationally infeasible (requires breaking AES-256). All others allow reconstruction with forensic tools and sufficient time.

---

## Composite Scorecard

Scale: 0 (worst) → 5 (best) per axis. Scores reflect the paper's own acknowledged limitations, not aspirational claims.

**Chambers scoring rationale:**
- Memory: 4 not 5 — objects encrypted but K_w itself is plaintext in mlock'd RAM. Root/DMA can read K_w.
- Hardware: 4 not 5 — APIs blocked but framebuffer capture and keystroke interception are accepted channels.
- Debug resist: 3 not 5 — ptrace denied but root can bypass. Hardened Runtime deferred to Phase 3.
- Remote resist: 5 — structural property, no network listener exists. Not bypassable.
- Residue: 5 — zero content/structural residue, 2 metadata events (accepted by design).

**Competitor notes:**
- Qubes (hardened): with LUKS per-qube + anti-evil-maid + minimal dom0 logging, scores higher than default.
- Tails: strong within its threat model (anonymity + amnesia) — scored 0 on axes it doesn't target, not axes it fails at.
- Firecracker (with jailer): scores higher than default configuration.

| System | Residue | Legibility | Preservation | Execution | Memory | Clipboard | Hardware | Debug resist | Crypto erase | Remote resist | Recon resist | **Total /55** |
|--------|---------|-----------|-------------|-----------|--------|-----------|----------|-------------|-------------|--------------|-------------|--------------|
| **Chambers** | **5** | **5** | **5** | **5** | **4** | **5** | **4** | **3** | **5** | **5** | **5** | **51** |
| AMD SEV/TDX | 3 | 1 | 1 | 1 | **5** | 0 | 0 | 4 | 1 | 0 | 2 | 18 |
| Intel SGX | 3 | 1 | 1 | 2 | **5** | 0 | 0 | 5 | 1 | 0 | 3 | 21 |
| Firecracker | 4 | 3 | 3 | 1 | 0 | 5 | 5 | 0 | 2 | 0 | 3 | 26 |
| gVisor | 3 | 2 | 2 | 3 | 0 | 5 | 4 | 0 | 0 | 0 | 2 | 21 |
| Qubes OS | 3 | 3 | 2 | 1 | 0 | 3 | 3 | 0 | 1 | 1 | 2 | 19 |
| Qubes (hardened) | 4 | 3 | 2 | 1 | 0 | 3 | 3 | 1 | 2 | 1 | 3 | 23 |
| Tails | 4 | 3 | 2 | 1 | 0 | 0 | 0 | 0 | 2 | 1 | 3 | 16 |
| Whonix | 3 | 2 | 2 | 1 | 0 | 3 | 0 | 0 | 1 | 0 | 2 | 14 |
| GrapheneOS | 2 | 2 | 1 | 1 | 1 | 0 | 2 | 2 | 0 | 0 | 2 | 13 |
| Docker | 2 | 2 | 1 | 1 | 0 | 5 | 1 | 0 | 0 | 0 | 1 | 13 |
| Standard VM | 2 | 2 | 1 | 1 | 0 | 2 | 1 | 0 | 0 | 0 | 1 | 10 |
| Private browsing | 1 | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 |

**Note:** SEV/TDX and SGX score 5/5 on Memory — they provide hardware-enforced encryption that Chambers cannot match in software. Chambers scores higher overall because it addresses axes that hardware encryption doesn't (lifecycle legibility, preservation law, execution surface). These are complementary approaches: Phase 3 Hypervisor boot + Secure Enclave would combine Chambers' architectural properties with hardware-backed memory isolation.

---

## Key Insights

### 1. Chambers occupies a distinct position, not a superior one

Chambers is not competing with these systems — it is solving a different problem. Tails, Qubes, and GrapheneOS are **operating systems** for general privacy. Firecracker and gVisor are **infrastructure** for isolation. SEV/TDX/SGX provide **hardware-backed confidential computing**. Docker is a **deployment tool**. Chambers is a **runtime for bounded private cognition**.

The comparison is useful for understanding where Chambers sits in the design space, not for claiming it replaces these tools. Chambers scores highest on axes that other systems don't address (lifecycle legibility, preservation narrowness, execution surface). It scores lower than hardware solutions on memory protection. These are different tools for different threat models.

**Complementarity:** Chambers could run inside Tails (for anonymity + ephemeral cognition), inside a Qubes disposable (for OS-level compartmentalization), or on hardware with SEV/TDX (for hardware-backed memory encryption). The isolation models are stackable, not competing.

### 2. Cryptographic erasure is the differentiator vs. software systems

Every software system in this comparison relies on deletion. Chambers relies on key destruction. This is why reconstruction time is infinite for Chambers and finite for everything else. The encrypted memory pool (Phase 2) extends this from post-burn to during-operation.

However, this is a software-level guarantee. A root-level attacker who reads K_w from process memory before burn can decrypt everything. Hardware solutions (SEV/TDX/SGX) provide stronger guarantees by keeping keys in hardware. Phase 3 (Secure Enclave) narrows this gap.

### 3. The closed execution surface is unique

No other system constrains what can happen inside to a finite algebra. Even gVisor, which filters syscalls, still allows hundreds of operations. Chambers allows 9. This is the "legal execution surface" axis — smaller surface means fewer residue channels. This property is independent of the underlying isolation technology and would be preserved even if Chambers ran inside a VM or TEE.

### 4. Firecracker is the closest software competitor

Firecracker scores highest among the software baselines because it shares key properties: no persistent storage (RAM-only possible), no device access, minimal surface. The difference is that Firecracker is generic infrastructure — it doesn't know about typed objects, preservation law, or burn semantics. It's security-by-cleanup. Chambers is security-by-construction.

### 5. Hardware TEEs are the strongest on memory protection

AMD SEV, Intel TDX, Intel SGX, and ARM CCA provide guarantees that Chambers cannot match in software: hardware-enforced memory encryption with keys that never exist in software-accessible memory. A compromised hypervisor cannot read guest RAM. Chambers' encrypted memory pool is defense-in-depth — it raises the bar significantly but does not reach the hardware-enforced level. Phase 3 (Hypervisor.framework + Secure Enclave) moves toward this but remains weaker than dedicated confidential computing hardware.

### 6. Signal Protocol analogy is precise

The paper references Signal's Double Ratchet as an analogy for burn semantics. The comparison holds:

| Property | Signal | Chambers |
|----------|--------|----------|
| Key scope | Per-message ratchet | Per-world K_w |
| Key destruction | Old keys deleted after ratchet advance | K_w zeroized on burn |
| Forward secrecy | Compromise of current key doesn't reveal past messages | Compromise of substrate doesn't reveal past worlds (K_w is gone) |
| What survives | Delivered messages on recipient device | Sealed artifacts in vault |

The difference: Signal's forward secrecy is per-message (fine-grained). Chambers' is per-world (coarse-grained). Signal preserves all delivered messages. Chambers preserves only explicitly sealed artifacts.

### 7. Private browsing is theater

Private browsing scores lowest because it provides almost no real isolation. DNS cache, OS page cache, favicon database, GPU process memory, extension state — all persist after the window closes. Users believe they're invisible. They're not.

---

## What Chambers Cannot Do (and these systems can)

| Capability | Chambers | Systems that can |
|-----------|----------|-----------------|
| General-purpose computing | No — 9 primitives only | All except Firecracker |
| Run arbitrary applications | No | All OS-based systems |
| Network access | No (localhost only) | All |
| File I/O | No | All |
| Multi-user | No | Qubes, Tails, GrapheneOS |
| Hardware attestation | No (Phase 3) | GrapheneOS (Titan M), Qubes (Anti-Evil Maid) |
| Anonymity / Tor routing | No | Tails, Whonix |
| App ecosystem | No | GrapheneOS, Qubes |

Chambers is deliberately narrow. It trades generality for provable isolation within its bounded scope.

**Why no anonymity?** Chambers does not route traffic through Tor or provide network anonymity because the chamber has no network. There is nothing to anonymize. However, Chambers is complementary to anonymity tools: running Chambers inside a Tails session would provide both anonymity (Tails handles the network layer) and ephemeral cognition (Chambers handles the world layer). The isolation models are orthogonal.

**Why no hardware attestation?** Chambers Phase 2 operates entirely in user space without Apple Developer account. Hardware attestation (Secure Enclave, Hypervisor.framework) requires Phase 3. This is a sequencing choice, not an architectural limitation.
