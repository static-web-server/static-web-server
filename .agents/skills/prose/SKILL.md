---
name: prose
description: Write or edit human-readable text for the Static Web Server (SWS) project — commit messages, CHANGELOG entries, PR descriptions, issue bodies, rustdoc comments, CLI help text, READMEs, and user docs — following project style and conventions. Use whenever producing text a person will read, not only Markdown files.
---

# Writing SWS Prose

**When to load**: writing a commit message, CHANGELOG entry, PR description, issue body, advisory text, `///` doc comment, `--help` text, README section, or documentation for the `static-web-server/docs` repo.

## Style

- **Fact-focused and direct**: what a thing is and does. No hedging, buzzwords ("leverage", "seamless", "robust"), or weasel words ("very", "really")
- **No dramatic terms**: "critical", "crucial", "essential" only when something breaks without it. In security text the CVSS score carries severity
- **Literal over figurative**: "out of the box" → "by default"; "under the hood" → describe the mechanism
- **Concrete**: the flag, the env var, the TOML key, the header, the status code
- **Active voice, present tense**: "SWS appends `Vary: Accept-Encoding`"
- **Current behavior only** in docs and doc comments; history belongs in commit messages
- **Plain characters in code**: no homoglyphs or invisible Unicode in identifiers, code, or config examples

**Bad**: "SWS leverages advanced algorithms to enhance delivery."
**Good**: "SWS picks `zstd`, `br`, `gzip`, or `deflate` from the client's `Accept-Encoding` q-values."

## Commit Messages (`docs/COMMITS.md`)

```
<type>(<scope>): <subject>

<body>

<footer>
```

- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`
- **Scope**: the module touched (`server`, `tls`, `compression`, `fs`, `handler`, `static_files`, `control_headers`, `custom_headers`, `cors`, `rewrites`, `directory_listing`, `access_logs`, `health`, …). Omit when cross-cutting
- **Subject**: imperative, lowercase first letter, no trailing period
- **Body**: imperative; what was wrong or missing, and how behavior differs now
- **Footer**: `Fixes #123`; `BREAKING CHANGE: <description>` for breaking changes. No AI `Co-authored-by:` trailers or "Generated with …" lines
- Every line ≤ 100 characters

The body follows the kernel's patch-description rules, adapted:

- **Problem first, then the change.** Open with what was wrong and how a user noticed it (the request, the status, the header). The second paragraph says what the code does now and why that approach. A reader should be able to judge the fix without opening the diff
- **One problem per commit.** A commit that fixes two things, or fixes one and reformats another, is two commits. Unrelated whitespace, import ordering, or rename churn goes in its own `style:`/`refactor:` commit or not at all
- **Orders, not narration.** Write as if telling the codebase what to do: "skip auth only for CORS preflights", not "this commit skips", "I changed", or "now it skips". Never "this patch" or "this PR"
- **Quantify.** A `perf:` commit names the benchmark and the before/after numbers; a size change names the bytes. "Faster" and "smaller" alone are not claims
- **State the user-visible effect** when there is one: new flag, changed default, changed status code, removed endpoint. A `BREAKING CHANGE:` footer repeats it
- **Reference, don't repeat.** Cite the RFC section, the issue, or the earlier commit hash instead of restating them
- **Security fixes**: describe the defect plainly, without the GHSA ID, severity words, or exploit steps; the message becomes public on merge

## PR Descriptions

Follow `.github/PULL_REQUEST_TEMPLATE.md`: what changes, why (link the issue or GHSA), how it was tested (commands and results), user-visible effects including new flags and defaults, and whether the docs repo needs an update.

A few lines per section, each fact checkable. No file-by-file recap of the diff, no filler, no AI attribution or credits. Only related changes; `CHANGELOG.md` is written by the maintainer, not in the PR.

## CHANGELOG Entries (`CHANGELOG.md`, maintainer-only)

Written by the maintainer at release time; PRs don't touch it. When asked to draft one, use Keep a Changelog sections as already used in the file (`### Breaking Changes`, `### New Features`, `### Bug Fixes`, `### Security & Hardening`, `### Performance`, `### Refactoring`, `### Testing`, `### Documentation`, `### Maintenance`):

```markdown
- **Short bold title.** One or two sentences on what changed for users, naming flags, env vars, or TOML keys. ([#758](https://github.com/static-web-server/static-web-server/pull/758) by [@user](https://github.com/user))
```

Describe the effect on users, not the implementation. For a breaking change, state old and new behavior. For a security fix, describe the impact without a ready-made exploit.

## Rustdoc and CLI Help

- `#![deny(missing_docs)]`: every public item, field, and variant needs `///`
- The `///` on a `General` field in `settings/cli.rs` is the `--help` text: what the option does, accepted values, and the default when not obvious from `default_value`
- Module docs (`//!`) state the module's responsibility and non-obvious invariants (`etag.rs`, `access_logs.rs`)
- A function's `///` is its contract: what it does, what it returns on failure, what it assumes; never a restatement of the signature. Inline `//` comments default to none; the ones that stay cite a constraint, a trap, or a measured cost (`coding-style/SKILL.md`, Comments)
- Log message wording lives in `coding-style/SKILL.md`, Log Messages: lowercase, subject first, no trailing punctuation

## User Documentation

User docs live in `static-web-server/docs` (`src/v3/`, `src/v2/`). A feature page contains:

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
4. Behavior when enabled and disabled, with a `curl -I` example
5. Interactions with related features

## Terminology

- **SWS**: Static Web Server; spell out on first mention
- **Root directory**: the `--root` / `-d` directory
- **Pre-compressed / static compression**: serving `.br`, `.gz`, `.zst` files from disk
- **On-the-fly / dynamic compression**: encoding at request time
- **Config file**: the TOML file (`sws.toml` by default, `--config-file` / `-w`)
- **Advanced options**: TOML-only `[advanced]` rules (headers, rewrites, redirects, virtual hosts, memory cache)
- **Early response**: a response from pipeline steps before the static file handler (405, health, CORS rejection, 401, metrics, maintenance, redirects)
- **Health probes**: `/livez` and `/readyz` (`--health`)
