---
name: code-quality
description: Review or self-check a change to the Static Web Server (SWS) project before it is called done — correctness, invariants, API design, concurrency, maintainability, and an SWS-specific review checklist. Use when reviewing a PR or diff, finishing an implementation, or refactoring.
---

# Code Quality and Review

The principles in `AGENTS.md` (correctness first, every change has a test, explicit invariants, own regressions, evidence over assumptions) become review criteria here. Build and verification mechanics: `rust-conventions/SKILL.md`. Readability and naming: `coding-style/SKILL.md`. Reviewing a GitHub PR end to end: `pr-review/SKILL.md`.

**When to load**: reviewing a diff, finishing a change before reporting it done, or planning a refactor.

## Definition of Done

Verified by running, not by inspection:

1. `cargo fmt --all -- --check tests/*.rs` passes
2. `cargo clippy -- -D warnings` passes for `--features all` and `--no-default-features`, with and without `--tests`
3. `cargo test --features all` and `cargo test --no-default-features` pass
4. The new or changed test fails without the change and passes with it
5. Public items have doc comments (`#![deny(missing_docs)]`)
6. User-visible changes note the `static-web-server/docs` follow-up in the PR. `CHANGELOG.md` is written by the maintainer, not in the PR
7. Commit messages follow `docs/COMMITS.md`; no AI attribution (`prose/SKILL.md`)

If a step was skipped or fails, say so with the output.

## Principles

**Correctness and invariants**
- Encode invariants in types (enums over boolean pairs, newtypes for validated values)
- Validate at the boundary: config at startup, request data in the step that parses it
- Fail fast on impossible states at startup; on the request path, map them to a status code and log
- Don't mask a defect with a defensive fallback. If a branch can't happen, say why or make it unrepresentable

**Explicitness**
- Control flow, ownership, and side effects visible at the call site
- A default lives in `settings/cli.rs` and is documented in `--help`
- Time, filesystem state, and randomness are explicit inputs

**Concurrency**
- No locks held across `.await`
- Shared config is immutable `Arc<RequestHandlerOpts>`; per-thread state uses `thread_local!`
- Cancellation (client disconnect, shutdown) leaves no partial state: caches insert only complete entries

**Maintainability**
- Small functions with one job; modules by feature, matching the existing layout
- Remove duplication with a real abstraction, not a helper with flags
- Refactor when a fix needs another special case; don't stack conditions
- Keep refactors out of bug-fix PRs

## SWS Review Checklist

**Pipeline and HTTP**
- [ ] New steps sit at a justified position (`design/SKILL.md`), after basic auth if they expose data
- [ ] Post-processing stays additive; custom headers apply last, including `scope = "all"` on early responses
- [ ] `HEAD` mirrors `GET` headers; `OPTIONS` is authenticated except CORS preflights; bodiless statuses carry no `Content-Encoding`
- [ ] Statuses are deliberate: 404 for traversal/hidden, 403 only for symlink policy, 401 for auth

**Configuration**
- [ ] New options work via CLI, env, and TOML; TOML-over-CLI precedence holds
- [ ] Validation at startup with a clear error
- [ ] Defaults are secure and match `--help`

**Features and Build**
- [ ] Optional code is feature-gated: fields, constructors, imports, tests
- [ ] `--no-default-features` builds without dead-code or unused-import errors
- [ ] Platform-specific options stay behind `#[cfg(unix)]`/`#[cfg(windows)]` wherever they are read
- [ ] New dependencies justified and within the binary-size budget

**Security and Performance**
- [ ] Every request-derived path goes through `sanitize_path()` and `security::enforce()`
- [ ] No panics reachable from request input
- [ ] No new per-request syscalls, allocations, or regex/glob compilation without a measurement
- [ ] Per-request labels, cache keys, and log fields are bounded

**Readability** (`coding-style/SKILL.md`)
- [ ] Reads like the surrounding file: same abbreviations (`req`, `resp`, `opts`), same pipeline verbs (`init`, `pre_process`, `post_process`), same log shape
- [ ] Names say what, not how or which type: predicates `is_`/`has_`/`can_` return `bool`, fallible lookups are `try_`, no `_str`/`_vec` suffixes
- [ ] Common path at the lowest indentation; no block nested more than three levels; no function that needs section comments to navigate
- [ ] No inline comment that a name or helper could replace; remaining ones cite a constraint, trap, cost, or deliberate fall-through. `#[cfg]` sits on items, not on statements

**Tests and Docs**
- [ ] Tests cover the change, its error path, and the feature-off path
- [ ] Each public function's `///` states its contract; no commented-out code
- [ ] One problem per commit; body states the problem before the change (`prose/SKILL.md`); the PR contains only related changes

## Writing Review Feedback

- Lead with security and correctness; style last
- Cite `file:line` and a concrete failing input for each bug
- Separate "must fix" from "suggestion"
- Verify a suspected bug with a test or reproduction before reporting it as confirmed
