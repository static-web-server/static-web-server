---
name: code-quality
description: Review or self-check a change to the Static Web Server (SWS) project before it is called done — correctness, invariants, API design, concurrency, maintainability, and an SWS-specific review checklist. Use when reviewing a PR or diff, finishing an implementation, or refactoring.
---

# Code Quality and Review

The engineering principles in `AGENTS.md` (correctness first, every change has a test, explicit invariants, own regressions, evidence over assumptions) apply to every change. This skill turns them into review criteria. Coding mechanics live in `rust-backend/SKILL.md`.

**When to load**: reviewing a diff or PR, finishing a change before reporting it done, or planning a refactor.

## Definition of Done

A change is done when all of these hold, verified by running them, not by inspection:

1. `cargo fmt --all -- --check tests/*.rs` passes
2. `cargo clippy` with `-D warnings` passes for `--features all` and `--no-default-features`, with and without `--tests`
3. `cargo test --features all` and `cargo test --no-default-features` pass
4. A new or changed test fails without the change and passes with it
5. Public items have doc comments (`#![deny(missing_docs)]`)
6. User-visible behavior changes have a `CHANGELOG.md` entry and a note about the docs repo update
7. The commit message follows `docs/COMMITS.md`

If a step was skipped or fails, say so in the report with the output.

## Principles

### Correctness and Invariants
- Encode invariants in types where practical (enums over boolean pairs, newtypes for validated values)
- Validate external input at the boundary: config at startup, request data in the step that parses it
- Fail fast on impossible states during startup; on the request path, map them to a status code and log
- Don't mask a defect with a defensive fallback. If a branch "can't happen", say why in a comment or make it unrepresentable

### Explicitness
- Control flow, ownership, and side effects are visible at the call site
- No hidden defaults: a default lives in `settings/cli.rs` and is documented in `--help`
- Same input, same output. Time, filesystem state, and randomness are explicit inputs

### Concurrency
- No locks held across `.await`. Keep critical sections short
- Shared config is immutable (`Arc<RequestHandlerOpts>`); per-thread state uses `thread_local!`
- Cancellation (client disconnect, shutdown) leaves no partial state: caches insert only complete entries

### Maintainability
- Small functions with one job; modules organized by feature, matching the existing layout
- Remove duplication with a real abstraction, not indirection. Three similar call sites are fine; a shared helper with flags is not
- Refactor when a fix needs another special case; don't stack conditions
- Keep refactors out of bug-fix PRs

## SWS Review Checklist

**Pipeline and HTTP**
- [ ] New steps sit at a justified position in the pipeline (`design/SKILL.md`), after basic auth if they expose data
- [ ] Post-processing remains additive; custom headers still apply last
- [ ] `HEAD` mirrors `GET` headers; `OPTIONS` and disallowed methods behave as before
- [ ] Error statuses are deliberate: 404 for traversal/hidden, 403 only for symlink policy

**Configuration**
- [ ] New options work via CLI, env, and TOML, and TOML-over-CLI precedence holds
- [ ] Validation runs at startup with a clear error message
- [ ] Defaults are secure and match the docs

**Features and Build**
- [ ] Optional code is feature-gated, including fields, constructors, imports, and tests
- [ ] `--no-default-features` builds without dead-code or unused-import errors
- [ ] New dependencies are justified and within the binary-size budget

**Security and Performance**
- [ ] Every request-derived path goes through `sanitize_path()` and `security::enforce()`
- [ ] No panics reachable from request input (`unwrap`, indexing, overflow)
- [ ] No new per-request syscalls, allocations, or regex/glob compilation without a measurement

**Tests and Docs**
- [ ] Tests cover the change, its error path, and the feature-off path
- [ ] Doc comments explain *why* for non-obvious code; no commented-out code
- [ ] CHANGELOG and commit message follow project conventions (`prose/SKILL.md`)

## Writing Review Feedback

- Lead with correctness and security findings; style last
- Cite `file:line` and give a concrete failing input or scenario for each bug
- Separate "must fix" from "suggestion"
- Verify a suspected bug with a test or a reproduction before reporting it as confirmed
