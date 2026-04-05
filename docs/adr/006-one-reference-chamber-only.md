# ADR-006: One Reference Chamber Only

## Status
Accepted

## Context
Building a broad grammar library before validating the substrate would be premature. The research question is whether the runtime model itself produces different outcomes — not whether many chamber types are useful.

## Decision
Phase 0 supports exactly **one** chamber grammar: the **Decision Chamber**. This grammar defines:

- 10 object types (decision_objective, premise, support_statement, constraint, risk, upside, contradiction, alternative, recommendation, decision_summary)
- Preservation law: only `decision_summary` may survive
- Three termination modes: converged-preserving, converged-total-burn, abort-burn
- Epoch-scoped capability narrowing (Active → ConvergenceReview → Finalization)

## Consequences
- The grammar system is designed for extensibility but only one grammar is implemented.
- All benchmarking uses the Decision Chamber.
- A broader grammar library is deferred to Phase 1+.
- If the Decision Chamber doesn't validate the thesis, more grammars won't help.
