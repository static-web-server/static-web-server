---
name: coding-style
description: Write Rust code for the Static Web Server (SWS) project that reads as if one person wrote it — function size and shape, naming of variables, functions and types, nesting and early exit, comments, conditional compilation, and log message wording. Use when writing or reviewing any code under src/, naming something new, refactoring for readability, or deciding where a comment or #[cfg] belongs.
---

# Coding Style

The goal is code that reads as if one person wrote it. Match the surrounding file before anything below; these rules settle what the file doesn't. Formatting is `rustfmt` defaults and is not discussed here. The rest adapts the Linux kernel coding style to Rust. Build and verification mechanics: `rust-conventions/SKILL.md`. Commit and PR text: `prose/SKILL.md`.

**When to load**: writing or reviewing code under `src/`, naming a function, type, field, or option, splitting or refactoring a function, writing a comment or log message, or placing a `#[cfg]`.

## Functions

- One job per function, named after that job. If a function has many local variables or needs section comments to navigate, split it
- Keep the common path at the lowest indentation. Reject early with `?`, `return`, `let ... else`, and `if let ... &&` chains (edition 2024); the kernel's centralized exit is `?`
- More than three levels of nesting means a helper is missing. Extract the inner block into a named function; don't comment your way through it
- More than a handful of parameters means a struct is missing (`HandleOpts`, `RequestHandlerOpts`)
- Keep the testable logic in a plain function that takes values and returns values (`append_headers`, `auto`, `sanitize_path`); wrap I/O and pipeline plumbing around it
- Existing long functions (`settings::parse_from`, `RequestHandler::handle`) are not a license to grow them; new behavior goes into a helper they call

## Naming

Scope decides length: a name is as long as the distance over which it must be understood.

- **Locals and parameters**: short and conventional. The codebase's forms are `req`, `resp`, `opts`, `err`, `buf`, `addr`, `len`, `dir`, `fmt`, `md`; use them, not `request`/`response`/`options`. A loop index is `i`. Don't invent a new abbreviation when a word already has one here
- **Functions and types**: descriptive, no abbreviations unless the abbreviation is the term (`uri`, `tls`, `cors`, `etag`). `count_active_users()`, never `cntusr()`
- **Predicates** return `bool` and read as a question: `is_allowed`, `is_hidden`, `has_content`, `can_serve`. A function that returns a value or `Result` never starts with `is_`
- **Fallible lookups** are `try_*` and return `Result<T, StatusCode>` or `Option<T>` (`try_file_open`, `try_memory_cache`); `get_*` is infallible
- **Pipeline verbs** are fixed: `init`, `pre_process`, `post_process`, `append_headers`. New modules reuse them
- **Booleans** name the true state (`follow_symlinks`, `include_hidden`, `is_dir`), never `flag`, `enabled2`, or a negative (`no_cache: bool` → `cache: bool`)
- **No type encoding in names**: `allow_headers_str` next to `allow_headers_vec` is a smell; name the role, let the type say the type
- **Types** say what the value is, not how it is stored: `ContentCoding`, `FileSize`, `AccessLogFormat`. Newtypes over raw `u64`/`String` when a value has an invariant (`FileSize(u64)`); enums over `bool` pairs; `type` aliases only when they add a name worth having (`Body`, `Result`, `CacheKey`), never to hide what something is
- **Config options** share one name across CLI flag, `SERVER_*` env var, and kebab-case TOML key (`--compression-static`, `SERVER_COMPRESSION_STATIC`, `compression-static`), and the field is that name in snake_case

## Comments

Rust has two kinds of comment with opposite rules.

**Doc comments (`///`, `//!`) are required and are the contract.** `#![deny(missing_docs)]` enforces them on every public item; rustdoc and IDE hover show them, so write them for the caller:

- First line: one sentence saying what the item is or does, no "This function…". On a `General` field it is the `--help` text
- Then only what a caller can't see in the signature: what failure means (`Err(StatusCode::NOT_FOUND)` when the file is missing or hidden), the invariant relied on (the base path is already canonical), the precondition. Headed sections in use: `# Security`, `# Warning`, `# Example`, `# ABNF`
- Don't restate the signature. `/// Returns the file path.` on `fn file_path()` says nothing; if there is nothing beyond the signature, the name is wrong or the item shouldn't be public
- Module docs (`//!`) state the module's responsibility and its non-obvious invariant (`etag.rs`, `access_logs.rs`, `static_files/mod.rs`)

**Inline comments (`//`) default to none.** Readable code is the first tool and a comment the last. Before writing one, try a name (`is_cors_preflight(opts, req)` replaces a comment about browsers at the call site), a `let` that names the intermediate value, or a helper function. Write the comment only for what code cannot say:

- An external constraint: the RFC section, the OS or hyper quirk, the browser behavior (`// RFC 9110 §12.4.2: a weight of 0 means "not acceptable"`)
- A trap the code deliberately avoids (`// Containment is check-then-use; don't add I/O here`)
- A measured cost that justifies an unusual shape
- A fall-through or no-op that would otherwise look like a bug (`// Hit, but the cached entry returned an error status. Fall through so the error path is consistent`)

Never narrate the next line (`// Set the path`), restate a type, walk through an algorithm step by step (split it into named functions), or label sections of a long function (split the function). Step labels inside `handle()` (`// CORS`, `// Redirects`) are the one accepted exception: they mirror the pipeline table in `design/SKILL.md`. The codebase sits at about one inline comment per 40 lines, nearly all of the kinds above; keep it there or lower.

**Suppressions carry a reason.** `#[expect(lint, reason = "...")]` over `#[allow(lint)]`: it says why and errors once the suppression is no longer needed (MSRV 1.88 supports it). `src/` has three `#[allow]`s; don't add a fourth.

**Never**: commented-out code, `// TODO` without an issue number, author tags or dates, `todo!()` / `unimplemented!()` in production code.

## Conditional Compilation

- Put `#[cfg(feature = ...)]` on items (modules, fields, functions), not on statements inside a function body. When a caller needs a value either way, provide a `#[cfg(not(...))]` stub (`will_serve_fallback`), the Rust form of the kernel's no-op header stubs
- Platform-only options (`--unix-socket*`, `--fd`, `--windows-service`) are `#[cfg(unix)]`/`#[cfg(windows)]`; code shared by all platforms must not read them unguarded
- Which places to gate and the compression `any(...)` block: `rust-conventions/SKILL.md`, Feature-Gated Code

## Log Messages

`tracing` macros, in this shape: lowercase, no trailing period or exclamation mark, the subject first, values as structured fields or `{:?}` for client-controlled strings.

```rust
tracing::info!(enabled, hsts = handler_opts.hsts, "security headers");
tracing::debug!("file not found: {:?} {:?}", file_path, err);
```

Startup settings log at `info` from each `init()`; per-request decisions at `debug`/`trace`; `warn` for a configuration that works but is probably not what the user meant; `error` only at the boundary that converts to a status code.

## Reuse

Don't reinvent helpers: `MethodExt`, `body::*`, `exts::headers`, `exts::mime`, `fs::meta` already exist. Search before adding. Three similar call sites are fine; a shared helper with mode flags is not.

## Review Checklist

- [ ] Reads like the surrounding file: same abbreviations, same pipeline verbs, same log shape
- [ ] Names say what, not how or which type; predicates return `bool`; fallible lookups are `try_`
- [ ] Common path at the lowest indentation; no block nested more than three levels; no function that needs section comments to navigate
- [ ] No inline comment the code could say with a name or a helper; those that remain cite a constraint, a trap, a cost, or a deliberate fall-through. `///` states the contract, not the signature; suppressions use `#[expect(..., reason)]`
- [ ] `#[cfg]` sits on items, not statements; platform-only options stay guarded
- [ ] Log messages are lowercase, subject first, no trailing punctuation
