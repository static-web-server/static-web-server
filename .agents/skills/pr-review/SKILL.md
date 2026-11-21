---
name: pr-review
description: Review a pull request for the Static Web Server (SWS) project — a standard PR (bug fix, feature, refactor, docs) or an advisory PR (security fix in a GHSA private fork). Reproduce and verify locally, review adversarially, check PR hygiene, then leave concise, evidence-backed inline comments as a pending (draft) GitHub review via gh.
---

# Pull Request Review

**When to load**: a PR number or URL is given, a contributor's branch needs auditing, or the request mentions "review", "draft review", or "pending comments".

## Non-negotiables (both PR types)

- **Draft only.** Comments go into a *pending* review. Never submit, approve, or request changes; the maintainer edits and submits.
- **Never guess: reproduce, verify, fix.** Reproduce the problem the PR addresses on the base. Verify the PR's claims by running them, not by reading them. Every defect you report comes with a fix you applied in the worktree and checked (the test fails before it and passes after). If a claim can't be verified, drop it or ask the maintainer.
- **Adversarial.** Assume the change is wrong until the evidence says otherwise. Try to break it: inputs the author didn't test, feature combinations, the feature-off build, other platforms, every other path to the same code.
- **Verified locally.** Run the gates yourself; a green CI does not replace reproducing the change. Advisory PRs have no CI at all, so every platform they touch is verified locally, in a container, or in a VM (step 3).
- **Same bar for every PR.** A one-line fix gets the same verification as a feature.
- **Signal over volume.** Comment only to fix a defect, propose a clearly better approach, or flag a real risk. No praise, no restating the code, nothing a linter already enforces.
- **In context.** Anchor each comment to the diff line it concerns. Use a file-level comment only when no line fits.

## 1. Classify the PR

| | Standard PR | Advisory PR |
|---|---|---|
| What | Bug fix, feature, refactor, docs, maintenance | Fix for a GitHub Security Advisory |
| Where | `static-web-server/static-web-server` | Private fork `static-web-server/static-web-server-ghsa-xxxx-xxxx-xxxx` |
| Spec | The linked issue and maintainer decisions | The draft advisory: PoC, Impact, Fix |
| CI | GitHub Actions run; still verify locally | None (Actions don't run in private forks) |
| Visibility | Public | Private until the advisory is published |
| Skills | Per the diff (step 5) | `security`, `security-advisory`, plus per the diff |

An advisory PR is held to everything a standard PR is, plus step 4b.

## 2. Gather Context

```bash
R=static-web-server/static-web-server        # advisory: static-web-server/static-web-server-ghsa-xxxx-xxxx-xxxx
gh pr view <N> -R $R --json title,body,author,baseRefName,headRefOid,files,commits,mergeable,mergeStateStatus
gh api repos/$R/pulls/<N>/reviews             # existing reviews, incl. the user's PENDING one
gh pr checks <N> -R $R                        # standard PRs only

gh issue view <issue> -R $R --comments        # standard: intent and maintainer decisions
gh api repos/static-web-server/static-web-server/security-advisories/<GHSA> \
  --jq '.summary, .description, .vulnerabilities, .cvss.vector_string'   # advisory: the spec
```

- The PR must implement what was *agreed* (issue) or what the advisory's Fix section requires, not only what its title says.
- Read existing review comments, including the maintainer's pending ones, so you don't duplicate them. Extend them where the same fix is needed elsewhere.
- For an advisory PR, list the other open advisories (`security-advisory/SKILL.md` step 1): a fix often covers part of a related one, and the PR body should say so.

## 3. Check Out and Verify

```bash
git remote -v                                     # GitHub remote may be `github`; forks are `ghsa-<prefix>`
git fetch <remote> pull/<N>/head:pr-<N>
git worktree add ../sws-pr-<N> pr-<N>             # never disturb the user's checkout
BASE=$(git merge-base pr-<N> <remote>/master)     # compare against the merge base, not master's tip
git -c core.pager=cat diff --no-ext-diff $BASE...pr-<N>   # external difftools break line numbers
```

Run the CI gates from `rust-conventions/SKILL.md` (fmt, clippy for both feature sets with and without `--tests`, tests for both feature sets) with an isolated `CARGO_TARGET_DIR` in the scratchpad, in the background.

**Prove the tests**: apply only the PR's test changes onto `$BASE` and run them. They must fail for the reason the PR fixes. A test that passes on the base proves nothing.

```bash
git worktree add ../sws-base-<N> $BASE
git -C ../sws-base-<N> checkout pr-<N> -- tests/<file>.rs   # plus any unit-test hunks
```

**Platform matrix**: for a standard PR, run the host gates and use CI for the other platforms, classifying its failures (below). For an **advisory PR**, which has no CI, run every row the diff touches.

| Diff touches | Verify on |
|--------------|-----------|
| Any code | macOS host (all gates above) |
| Linux behavior, `cfg(unix)` code, Docker files, musl builds | Linux container: `docker run --rm -v "$PWD":/src -w /src -e CARGO_TARGET_DIR=/tmp/target rust:1.88 cargo test --features all` (`1.88` is the MSRV) |
| `cfg(windows)` code, Windows service, path separators, or anything the server options read on every platform | Windows VM: build and run `cargo test` there. A cross `cargo check --target x86_64-pc-windows-gnu` from macOS fails without a mingw-w64 C toolchain (`libz-sys` build script) |
| TLS, HTTP/2, sockets, `--fd` | The release binary on the host, probed with `curl` / `openssl s_client` |

Ask the maintainer before starting a VM. If a platform could not be verified, say so in the report; never assume it builds. (A Unix-only option read from shared server code broke the Windows build in an advisory fix that had no CI.)

**Run the binary** (`cargo build --release --features all`) and exercise the change through HTTP, including the rejection paths (invalid combinations, TOML-only values, env vars). Tell the user first about any visible side effect (opening a browser, binding a port, writing files).

**New dependency**: build `--release` on `$BASE` and the PR and compare binary sizes (>100KB needs a justification or a feature flag). Read the crate source for anything you plan to comment on (blocking calls, process spawning, `unsafe`, platform `compile_error!`s).

**CI failures** (standard PRs): classify each one. Caused by the PR, inherited from the base (reproduces on `$BASE`, or already fixed on `master`), or cancelled by fail-fast? Only PR-caused failures become inline comments; the rest go into the rebase note.

**Conflicts**: `git merge-tree --write-tree --name-only pr-<N> <remote>/master` lists the conflicting files.

## 4a. Adversarial Review: Standard PR

- **Reproduce the problem** on `$BASE` from the issue's steps. If it doesn't reproduce, the PR may fix the wrong thing; ask before reviewing further
- **Break the change**: write your own failing test for each edge case the author skipped (IPv6, `0.0.0.0`/`::`, empty values, missing leading `/`, `--port 0`, non-ASCII paths, `HEAD` vs `GET`, HTTP/2). Run it in the worktree
- **Root cause, not symptom**: the fix sits at the line that causes the bug. `rg` for the same pattern in sibling code (another pipeline step, `directory_listing` vs `static_files`, the `2.x` branch)
- **Feature combinations**: the change with each related feature on and off (memory cache, compression, CORS, basic auth, virtual hosts, rewrites)

## 4b. Adversarial Review: Advisory PR

Everything in 4a, plus:

1. **Exploit on base**: run the advisory's PoC against a `$BASE` release build. It must succeed. If it doesn't, stop and tell the maintainer: the advisory or the PR targets the wrong thing
2. **Exploit on head**: the same PoC against the PR build must fail with the status the fix promises
3. **Every variant**: all other paths to the same data or effect: each method (`GET`, `HEAD`, `OPTIONS`, unsupported), HTTP/1 and HTTP/2+TLS, cold and warm memory cache, percent-encoded, `..`, trailing-slash and index-resolved paths, pre-compressed variants, virtual hosts, rewrites, redirects, fallback page, `--no-default-features`. One open variant means the advisory isn't fixed
4. **The class, not the instance**: the fix removes the root cause named in the advisory's Details. For "HSTS sent over plain HTTP by default": decide HSTS once from `--tls` at startup so every server mode (HTTP/1, HTTP/2, plain, Unix socket) agrees, not a patch in one accept loop; and fix what the investigation turned up next to it (an explicit `--security-headers=false` was ignored with `--tls`)
5. **No regression in legitimate use**: the flows the vulnerable code served still work (a CORS preflight without credentials still gets `204`; an authenticated `GET` still gets the file and still caches it)
6. **Fail closed**: denials follow `security/SKILL.md` (404 for traversal, containment, and hidden files; 403 only for symlink policy; 401 for auth). No new detail leaks through error bodies, headers, or logs
7. **Regression tests**: at least one handler test drives the exploit through the full pipeline (`RequestHandler::handle()`), not only a unit test of the patched function. Proved to fail on `$BASE`
8. **Advisory consistency**: the PR implements the advisory's Fix section; the affected range still holds; anything partially fixed in a related advisory is named in the PR body; the v2 LTS line (`2.x`) is checked and a backport is noted if affected
9. **Disclosure hygiene**: the PR stays in the private fork. Commit messages are ordinary fix commits (`docs/COMMITS.md`): what was wrong and what changes, with no GHSA ID, severity words, or exploit script, because they become public on merge. Review comments go on the fork PR only

## 5. Review Against the Project Skills

Load the skills the diff touches and use their checklists as review criteria. Don't restate them in comments; cite the rule.

| Diff touches | Load |
|--------------|------|
| Any change | `code-quality` (definition of done, review checklist) and `prose` (commits, PR text, rustdoc) |
| `src/` | `rust-conventions`, `coding-style` |
| Tests, fixtures, proptests, fuzz, benches | `testing` |
| Pipeline order, config options, feature flags, defaults | `design` |
| `handler.rs`, `static_files/`, `fs/`, compression, header steps, new dependency | `performance` |
| Paths, auth, CORS, TLS, headers, untrusted input | `security` |
| Compression, caching, ETag, ranges, listing, TOML rules | `static-file-serving` |
| A bug fix | `issue-tracking` |
| An advisory fix | `security-advisory` |

Read the **full file** around each hunk, not just the hunk. Most real defects live in the interaction with unchanged code.

## 6. PR Hygiene

Check the PR itself, not only the code:

```bash
git log --format='%h %s%n%b' $BASE..pr-<N>                 # every commit message
git -c core.pager=cat diff --no-ext-diff --stat $BASE...pr-<N>   # every file touched
```

- **No AI credits.** No `Co-authored-by:` trailers for AI tools or bots, no "Generated with …" lines, no AI attribution in commits, the PR body, code comments, or docs. Human co-authors are fine
- **Only related changes.** No `CHANGELOG.md` hunks (the maintainer writes it), no scratch notes, reports, local configs, or editor files, no version bumps, no lockfile churn without a dependency change, no formatting or refactoring of untouched code. One concern per PR
- **Clear, concise description.** Follows the sections of [.github/PULL_REQUEST_TEMPLATE.md](../../../.github/PULL_REQUEST_TEMPLATE.md), every one filled with facts the reviewer can check. A few lines per section. No restating the diff file by file, no marketing or filler, no explanation the reader doesn't need. For an advisory PR, "Related Issue" is "Reported privately" or the GHSA ID (the fork is private); "How Has This Been Tested?" names the tests, states that they fail on `master`, and lists the gates and platforms run
- **Commit messages** follow `docs/COMMITS.md` (`prose/SKILL.md`)

Raise hygiene problems in one comment listing every item to drop or rewrite, not one comment per item.

## 7. Filter Findings

Keep a finding only if **all** hold:

1. **Verified**: reproduced, or proven by an exact source line
2. **Caused or touched by this PR**: pre-existing issues go to the maintainer in chat, not on the PR
3. **Actionable, with a checked fix**: the author knows exactly what to change, and you ran that change
4. **Not duplicated** in an existing or pending comment
5. **Worth the author's time**

Rank: security defects → correctness defects → risks → better approach → `nit:`. If more than ~8 remain, cut the nits.

## 8. Comment Style

- 1–3 sentences: what, why (the evidence in brief: the failing request, test, or line), and what to do instead
- Friendly and direct; address the author as "you"; prefix pure style points with `nit:`
- Prefer a ```` ```suggestion ```` block when the fix is local and drop-in. It must compile and pass `cargo clippy`; you applied it in the worktree first
- Label optional ideas as ideas ("Idea:", "What about…?") so they don't read as blocking
- Advisory PRs: if the maintainer asks you to push a fix instead of suggesting it, commit it to the fork branch with a `docs/COMMITS.md` message and no AI trailers

## 9. Post as Pending Review

GitHub allows **one pending review per user per PR**: find it first, create one only if none exists, otherwise append threads to it. The `gh` commands, GraphQL mutations, and their traps (new-file line numbers, the empty-body 422, shell escaping) are in [posting.md](posting.md). The same calls work on the private fork with its owner/name.

After posting, re-query the pending review and confirm every comment landed on the intended `path:line`.

## 10. Report Back

Tell the maintainer, briefly:

- PR type, and what was verified: gates, test counts, tests proved to fail on base, platforms run (host, container, VM) and any left unverified, runtime checks, size delta, CI classification
- Advisory PRs: PoC result on base and head, variants tried, whether the advisory text needs updating
- The comments added (one line each), and that the review is still pending
- Findings deliberately **not** posted (pre-existing or out of scope) that deserve their own issue or advisory

Clean up worktrees and scratch builds when done (`git worktree remove`).
