# ADR-001: World-First Architecture

## Status
Accepted

## Context
Traditional computing models treat the application or guest VM as the primary semantic unit. Data persists by default, and deletion is an afterthought. Chambers inverts this: the **world** (chamber) is the primary semantic unit, not the app, process, or guest machine.

## Decision
All runtime behavior is organized around world lifecycle. Objects, capabilities, links, and state exist only within a world's namespace. No object is addressable outside its owning world. The world is created, lives through defined epochs, and is destroyed through a formal burn sequence.

## Consequences
- Every engine (object, capability, state, operation) is world-scoped.
- Cross-world communication is impossible except through the artifact vault.
- World IDs are never reused after burn.
- There is no global object namespace.
