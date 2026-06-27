---
name: issue-tracking
description: Triage, debug, fix, and document issues for the Static Web Server (SWS) project — bug reports, root cause analysis, fix implementation, and regression prevention
---

# Issue Tracking & Debugging

Load this skill when triaging bug reports, investigating issues, implementing fixes, or writing post-mortem documentation for SWS.

**When to load**: triaging a new issue report, investigating a regression, writing a fix for a bug, drafting a post-mortem, or reviewing a bug-fix PR.

## Issue Triage

### Reproducibility First

- **Can you reproduce it?** Follow the exact steps in the report. If unreproducible, ask the reporter for environment details (OS, architecture, SWS version, config file, TLS setup)
- **Minimal reproduction**: Reduce the scenario to the smallest config + file structure that triggers the bug. Strip unrelated features
- **Write a failing test first**: Before fixing, write a test that reproduces the bug. Use the fixture infrastructure in `tests/` and `src/testing.rs`

### Severity Classification

| Severity | Definition | Response |
|----------|-----------|----------|
| **P0 - Critical** | Security vulnerability, data exposure, path traversal | Drop everything. Fix immediately. Security release |
| **P1 - High** | Broken core feature (file serving, TLS), crash on start | Fix in current sprint. Patch release |
| **P2 - Medium** | Broken non-core feature, workaround exists | Schedule in next sprint |
| **P3 - Low** | Cosmetic, log message, doc typo | Backlog. Fix when touching related code |

## Root Cause Analysis

### Debugging Process

1. **Gather evidence**: Logs (`-g trace`), stack traces, HTTP response headers, request URIs, config file
2. **Form a hypothesis**: Based on the evidence, propose what might cause the bug
3. **Test the hypothesis**: Add tracing, run reproduction, or step through with a debugger
4. **Identify the root cause**: Find the exact line or condition that triggers the bug. Don't stop at symptoms
5. **Verify the fix**: The reproduction test now passes. The original scenario works

### Rust Debugging

- **Use `tracing` crate for structured logs**: SWS uses `tracing-subscriber`. Run with `-g trace` for maximum detail. Log levels: ERROR (actionable), WARN (unexpected but handled), INFO (key events), DEBUG (detailed), TRACE (noisy)
- **Use `dbg!()` for quick inspection**: Temporary, remove before committing
- **Enable backtraces**: `RUST_BACKTRACE=1` for panic backtraces, `RUST_LIB_BACKTRACE=1` for error backtraces

### HTTP Debugging

- **Inspect response headers**: Use `curl -v http://localhost:8787/path` to see full request/response exchange
- **Test with specific Accept-Encoding headers**: `curl -H "Accept-Encoding: br" ...` to test compression variant selection
- **Check security headers**: `curl -I http://localhost:8787/ | grep -i 'x-\|strict\|csp\|referrer'`
- **Test byte-range requests**: `curl -H "Range: bytes=0-99" http://localhost:8787/file`
- **CORS preflight debugging**: `curl -X OPTIONS -H "Origin: https://example.com" -H "Access-Control-Request-Method: GET" http://localhost:8787/`
- **TLS verification**: `openssl s_client -connect localhost:8787 -servername localhost`

### File-Serving Debugging

- **Path resolution issues**: Check if the file exists at the resolved path. SWS logs the resolved path at `trace` level
- **Hidden file / symlink blocking**: Verify `--include-hidden` and `--follow-symlinks` settings (both default to `false`). Hidden files return 404 (stealth), symlinks return 403
- **Index file resolution**: If a directory returns 404 instead of an index, check `--index-files` list and file existence
- **MIME type issues**: SWS uses `mime_guess` from file extension. If the wrong `Content-Type` is served, check the file extension

## Fix Implementation

### Before Writing the Fix

- [ ] Is there a failing test that reproduces the bug?
- [ ] Is the root cause identified (not just the symptom)?
- [ ] Does the fix address the root cause?
- [ ] Are there other places in the codebase with the same bug pattern?

### Writing the Fix

- **Minimal change**: Fix the bug with the smallest possible code change. Do not refactor unrelated code in the same PR
- **Add a regression test**: The reproduction test becomes a permanent regression test
- **Update documentation**: If the fix changes behavior, update the relevant feature doc in `docs/content/features/`

### Commit Message Format

```
fix(scope): brief description of the fix

Detailed explanation of the root cause and the fix.
Include steps to reproduce, expected behavior, and actual behavior.

Fixes #123
```

`scope` is the affected module or feature (e.g., `compression`, `tls`, `static-files`). See `COMMITS.md` for the full convention.

## Regression Prevention

- **The reproduction test stays**: Every bug fix adds a test that prevents the same bug from returning
- **Check similar code paths**: Search the codebase for patterns that could cause the same class of bug
- **Add a lint rule if applicable**: If a pattern caused the bug and can be detected statically, add a clippy or ESLint rule

## Post-Mortem (P0/P1 only)

For critical and high-severity issues, write a brief post-mortem:

1. **What happened**: Timeline of the incident
2. **Root cause**: The specific code or configuration that caused it
3. **Impact**: What users were affected and how
4. **Fix**: What change resolved the issue
5. **Prevention**: What process, tooling, or test prevents recurrence

Store post-mortems in `docs/post-mortems/YYYY-MM-DD-title.md`.

## Checklist

- [ ] Bug is reproduced and understood
- [ ] Root cause identified (not just symptom)
- [ ] Failing test written before the fix
- [ ] Fix is minimal and addresses root cause
- [ ] Regression test added
- [ ] Similar code paths checked for the same bug pattern
- [ ] Commit message follows format
