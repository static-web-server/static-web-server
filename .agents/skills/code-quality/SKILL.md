---
name: code-quality
description: Ensure high-quality Rust code in the Static Web Server (SWS) project
---

# Code Quality Guide

Load this skill when writing, reviewing, or refactoring Rust code in the SWS project.

**When to load**: editing any file under `src/`, adding a new module, changing error handling, touching async code, or reviewing a PR that modifies Rust source.

## Core Engineering Principles

* **Correctness above all.** Build production-grade software, not prototypes. Prioritize correctness, reliability, and maintainability over expedient shortcuts.

* **Every change requires a test.** Every code change must be accompanied by a test that fails before the change and passes after it. No behavioral change is complete without objective verification.

* **Enforce invariants explicitly.** Critical assumptions and invariants must be asserted, not silently ignored. Fail fast on invalid states rather than masking defects with defensive conditionals that obscure root causes.

* **Own regressions end-to-end.** Any test failures introduced by your change are your responsibility to investigate and resolve. Do not defer by comparing against another branch or attempting to prove the failure is pre-existing. Diagnose the failure, identify the root cause, and either fix it or provide conclusive evidence that it is unrelated.

* **Evidence over assumptions.** Every debugging hypothesis must be validated with reproducible evidence. Never speculate, infer causality without proof, or implement fixes based on unverified assumptions. Root-cause analysis must be grounded in observable facts.

* **Optimize only after measurement.** Performance work must be driven by profiling, benchmarks, or measurable evidence. Do not trade correctness or maintainability for speculative micro-optimizations.

## API Design

* Design APIs that make invalid states impossible or difficult to represent.
* Encode invariants in the type system whenever practical instead of relying on runtime validation.
* Public APIs should have clear ownership semantics and minimal surprises.
* Favor composability over specialization.

## State & Concurrency

* Minimize mutable state. Keep state transitions explicit and deterministic.
* Prefer ownership and message passing over shared mutable state. Synchronize shared state explicitly.
* Avoid holding locks across `.await`. Acquire locks for the shortest possible duration.
* Keep async critical sections small. Never block asynchronous executors (see `rust-backend` skill: no `block_on` in async context).
* Ensure task cancellation leaves the system in a valid state.

## Maintainability

* Prefer straightforward code over clever code.
* Eliminate duplication through sound abstractions, not indirection.
* Keep functions focused on a single responsibility. Keep modules cohesive.
* Refactor when complexity increases instead of layering additional special cases.
* Code should be understandable without external explanation.

## Explicitness & Predictability

* Make control flow, ownership, and lifetimes obvious.
* Prefer explicit conversions over implicit behavior.
* Identical inputs must produce identical outputs unless randomness or external state is explicitly part of the contract.
* Avoid hidden side effects and surprising defaults.

## Resilience

* Validate external input immediately. Never silently ignore malformed input.
* Fail fast on impossible states. Recover gracefully from expected operational failures.
* Preserve service availability whenever recovery is possible.

## Code Review Checklist

Every contribution should leave the codebase:

* More correct.
* More explicit.
* More maintainable.
* Better tested.
* At least as performant.
* Easier to reason about than before.

## Cross-References

The following concerns are governed by dedicated skills — defer to them for specifics:

| Concern | Skill |
|---------|-------|
| Error handling (`Result<T>`, `anyhow::Context`, `StatusCode`), `unsafe` policy, async runtime, file system, patterns to avoid | `rust-backend/SKILL.md` |
| Test organization, fixtures, handler/static-file test patterns | `testing/SKILL.md` |
| Profiling, allocations, hot-path optimization, benchmarking | `performance/SKILL.md` |
| Path traversal, TLS, security headers, CORS, input validation | `security/SKILL.md` |
