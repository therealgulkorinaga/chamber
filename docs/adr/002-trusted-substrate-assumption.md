# ADR-002: Trusted Substrate Assumption

## Status
Accepted

## Context
Chambers must define a clear trust boundary. Claiming protection against all threat classes would be dishonest and architecturally unsound.

## Decision
The substrate runtime is the central trust object. All security guarantees assume:

- The substrate itself is not compromised.
- No lower-platform compromise (firmware, bootloader, DMA attacks).
- No malicious peripherals.
- No supply-chain compromise of the substrate binary.

If the substrate is compromised, all guarantees collapse. This is stated explicitly rather than hidden.

## Consequences
- Phase 0 does not include hardware attestation.
- Phase 0 does not defend against kernel-level attacks.
- All burn guarantees are conditional on the substrate being trustworthy.
- This is documented in the security assumptions document and stated in the PRD.
