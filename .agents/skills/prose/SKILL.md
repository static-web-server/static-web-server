---
name: prose
description: Write or edit human-readable text for the Static Web Server (SWS) project — commit messages, CHANGELOG entries, PR descriptions, issue bodies, rustdoc comments, CLI help text, READMEs, and user docs — following project style and conventions. Use whenever producing text a person will read, not only Markdown files.
---

# Writing SWS Prose

**When to load**: writing a commit message, CHANGELOG entry, PR description, issue body, `///` doc comment, `--help` text, README section, or documentation for the `static-web-server/docs` repo.

## Style

- **Fact-focused**: state what a thing is and does
- **Direct**: no hedging, no buzzwords ("leverage", "seamless", "robust"), no weasel words ("very", "quite", "really")
- **No dramatic terms**: "critical", "crucial", "vital", "essential" only when something breaks without it
- **Literal over figurative**: "blazing fast" → "serves X req/s at p99 Y ms"; "out of the box" → "by default"; "under the hood" → describe the mechanism
- **Concrete**: include the flag, the env var, the TOML key, the header, and the status code
- **Active voice, present tense**: "SWS appends `Vary: Accept-Encoding`", not "the header is appended"
- **Current behavior only** in docs and doc comments. History belongs in the CHANGELOG and commit messages
- **Plain characters in code**: no homoglyphs or invisible Unicode in identifiers, code, or config examples. Em dashes are fine in prose; in Rust source write them as the character, not a `—` escape inside a `//` comment

**Bad**: "SWS leverages advanced algorithms to enhance delivery."
**Good**: "SWS picks `zstd`, `br`, `gzip`, or `deflate` from the client's `Accept-Encoding` q-values."

## Commit Messages (`docs/COMMITS.md`)

```
<type>(<scope>): <subject>

<body>

<footer>
```

- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`
- **Scope**: the module touched, e.g. `server`, `http2`, `tls`, `compression`, `fs`, `handler`, `static_files`, `control_headers`, `custom_headers`, `cors`, `rewrites`, `directory_listing`. Omit when the change is cross-cutting
- **Subject**: imperative, lowercase first letter, no trailing period
- **Body**: imperative; the motivation and how behavior differs from before
- **Footer**: `Fixes #123`, and `BREAKING CHANGE: <description>` for breaking changes
- Every line ≤ 100 characters

## CHANGELOG Entries (`CHANGELOG.md`)

Keep a Changelog format. Each release groups bullets under the sections already used in the file: `### Breaking Changes`, `### New Features`, `### Bug Fixes`, `### Security & Hardening`, `### Performance`, `### Refactoring`, `### Testing`, `### Documentation`, `### Maintenance`. One bullet per change:

```markdown
- **Short bold title.** One or two sentences on what changed for users, naming flags, env vars, or TOML keys. ([#758](https://github.com/static-web-server/static-web-server/pull/758) by [@user](https://github.com/user))
```

Describe the effect on users, not the implementation. For a breaking change, state the old and new behavior.

## PR Descriptions

Follow `.github/PULL_REQUEST_TEMPLATE.md`. Cover: what changes, why (link the issue), how it was tested (commands run), and user-visible effects including new flags and defaults. Note when the docs repo needs an update.

## Rustdoc and CLI Help

- `#![deny(missing_docs)]`: every public item, field, and variant needs `///`
- The `///` on a `General` field in `settings/cli.rs` is the `--help` text. Say what the option does, the accepted values, and the default when it isn't obvious from `default_value`
- Module docs (`//!`) state the module's responsibility and any non-obvious invariant (see `etag.rs`)
- Comments explain *why*: the constraint, the RFC section, the measured cost. Don't restate the code

## User Documentation

User docs live in the separate `static-web-server/docs` repository (`src/v3/` for v3, `src/v2/` for v2), not in this repo. A feature page contains:

1. One-sentence summary
2. Default state and how to toggle it
3. CLI, env var, and TOML forms:
   ```bash
   static-web-server --compression-static true
   SERVER_COMPRESSION_STATIC=true static-web-server
   ```
   ```toml
   [general]
   compression-static = true
   ```
4. Behavior when enabled and disabled, with an HTTP example (`curl -I` output)
5. Interactions with related features

## Terminology

- **SWS**: Static Web Server; spell out on first mention
- **Root directory**: the `--root` / `-d` directory being served
- **Pre-compressed / static compression**: serving `.br`, `.gz`, `.zst` files from disk
- **On-the-fly / dynamic compression**: encoding the response at request time
- **Config file**: the TOML file (`sws.toml` by default, `--config-file` / `-w`)
- **Advanced options**: TOML-only `[advanced]` rules (headers, rewrites, redirects, virtual hosts, memory cache)
