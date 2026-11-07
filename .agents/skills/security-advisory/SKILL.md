---
name: security-advisory
description: Draft a GitHub Security Advisory (GHSA) for the Static Web Server (SWS) project — reproduce and scope the vulnerability, verify affected versions, score it with CVSS 3.1 and CWEs, write the advisory text, and create or update the private draft via gh. Use when a vulnerability is reported privately or confirmed by an audit, or when editing an existing draft advisory.
---

# Security Advisory Drafting

**When to load**: a vulnerability is reported (`SECURITY.md`) or found in an audit, an existing draft needs editing, or a fix has merged and the advisory must match it. Fix code follows `security/SKILL.md`; the fix PR is reviewed with `pr-review/SKILL.md` (Advisory PR). The worked example at the end is the model for every section below.

## Non-negotiables

- **Reproduced, not inferred.** Every claim (trigger, preconditions, affected versions, impact, mitigation) comes from a run against a built binary. A finding backed only by code reading is not an advisory yet
- **Private until published.** No details in public issues, PRs, commits, discussions, or tracked files. An audit tracking table holds one line and the GHSA link
- **Draft only.** Show the maintainer the final text before creating the draft. Never publish, request a CVE, add collaborators, or create the private fork without an explicit go-ahead for that step
- **One defect per advisory.** Related defects in different components are filed separately even when the class is the same (GHSA-cg27-w6jh-934r, pre-compressed variant symlink bypass, and GHSA-3p6p-6r38-h33v, directory archive symlink traversal). Two impacts of one defect are split too when they would score differently, and cross-referenced

## 1. Reproduce and Scope

Work in the scratchpad with a release build and the smallest root and `sws.toml` that trigger it:

```bash
cargo build --release --features all
./target/release/static-web-server -w ./repro.toml -g trace &
curl -si http://127.0.0.1:8787/          # record status, headers, body
```

Record the exact requests, the observed response, and the expected one. Then establish:

- **Minimal preconditions**: turn each flag and feature off in turn. Name the ones required and confirm the others are not
- **Variants**: `GET`/`HEAD`/`OPTIONS`, HTTP/1 vs HTTP/2+TLS, Unix socket, cold vs warm memory cache, percent-encoded and trailing-slash paths, index and pre-compressed resolution, virtual hosts. Variants with the same impact go in one advisory
- **Affected versions**: run the PoC against each released version (release binary, `joseluisq/static-web-server:<version>` image, or a `git worktree` of the tag). `git log -S '<code>'` finds the commit that introduced it. Check the v2 LTS line (`2.x`) separately
- **Root cause**: the exact `file:line`, pinned to a commit hash
- **Overlap**: list existing advisories and check for duplicates or a shared root cause:
  ```bash
  gh api 'repos/static-web-server/static-web-server/security-advisories?per_page=50' \
    --jq '.[] | [.ghsa_id, .state, .severity, .summary] | @tsv'
  ```

## 2. Score

**CVSS 3.1**: justify each metric from the reproduction, not from the worst case imaginable. Score the vulnerable configuration; a non-default setting is stated in Impact and does not raise Attack Complexity. Examples from published SWS advisories:

| Metric | Guidance |
|--------|----------|
| `AV` | `N`: SWS is a network server |
| `AC` | `L` unless the attack depends on luck or state the attacker can't control: `H` for the bcrypt truncation collision (GHSA-cx3m-fg6q-xf3v, 3.7 Low) |
| `PR` | `N` when any client can do it: idle-connection exhaustion (GHSA-2fch-9x9r-mxmv, 7.5 High). `H` when the attacker must first write into the root, such as planting a symlink (GHSA-966j-8jgr-656v, 4.9 Medium) |
| `UI` | `R` when another user's request completes the attack: a victim fetching a poisoned cache entry (GHSA-67fg-g42f-773c, 4.3 Medium) |
| `C`/`I`/`A` | The demonstrated effect only: `A:H` for file-descriptor exhaustion, `I:L` for mislabelled cache content, `C:H` for reading any file the server can |

The API takes either `cvss_vector_string` or `severity`, not both. When the defect is an insecure default rather than an attack (no attacker action, harm comes from a misconfiguration), a vector fits poorly; set `severity` directly, as the worked example does.

**CWEs**: the most specific one to three. Used in SWS advisories so far:

| Class | CWE |
|-------|-----|
| Auth bypass through another method or path | CWE-288 |
| Protected data exposed; existence oracle | CWE-200; CWE-203 |
| Path traversal, symlink escape | CWE-22, CWE-59, CWE-61 |
| Unbounded resource use | CWE-400, CWE-770, CWE-772 |
| Cache or intermediary misinterpretation | CWE-436, CWE-706, CWE-838 |
| Spoofable forwarded client address | CWE-348 |
| Insecure default | CWE-1188 |
| Weak credential check | CWE-20, CWE-287 |

## 3. Write

Prose rules from `prose/SKILL.md` apply: factual, concrete flags and status codes, no "critical" or "severe" (the score carries severity), nothing beyond what was reproduced.

**Title** (`summary`): `<effect> <when|via|by> <condition>`, one line. Good: "Strict-Transport-Security with preload is sent over plain HTTP by default". Bad: "Critical HSTS misconfiguration vulnerability".

**Description**, these headings in this order:

| Section | Content |
|---------|---------|
| `### Summary` | One or two sentences: which configuration, what happens, to whom |
| `### Details` | The mechanism, component by component: which piece lacks what knowledge or check, and which standard says it must not (the RFC section) |
| `### PoC` | The steps you ran: configuration, requests, observed vs expected. Numbered steps or a short shell block. No scanners or mass-exploitation tooling |
| `### Impact` | Who is affected, what they lose, for how long, which deployments, and whether defaults widen it ("all release binaries") |
| `### Mitigation until fixed` | Configuration-only workarounds, each verified against the PoC |
| `### Fix` | The behavior the fix must enforce, as bullets, not a patch. Rewritten to match the merged fix |

Optional `### Affected code (<commit>)`: `file:line` bullets pinned to a commit hash, when the location isn't obvious from Details. Never link to the private fork.

**Fields**: `vulnerabilities` (ecosystem `rust`, package `static-web-server`, `vulnerable_version_range` such as `>= 3.0.0-beta.1, <= 3.0.0-beta.2`, `patched_versions` empty until a release ships the fix; one advisory per release line when v2 and v3 are fixed separately, each naming the other), `cwe_ids`, `cvss_vector_string` or `severity`, `credits` (`reporter` with the public name only by consent; `analyst`, `remediation_developer`, `remediation_reviewer`, `remediation_verifier` as the roles were).

## 4. Create or Update the Draft

Keep the description in a Markdown file and build the request with `jq`, so backticks and code blocks survive:

```bash
jq -n --rawfile d description.md \
  --arg s 'Strict-Transport-Security with preload is sent over plain HTTP by default' \
  '{summary: $s, description: $d, severity: "low", cwe_ids: ["CWE-1188"],
    vulnerabilities: [{package: {ecosystem: "rust", name: "static-web-server"},
      vulnerable_version_range: ">= 3.0.0-beta.1, <= 3.0.0-beta.2",
      patched_versions: null, vulnerable_functions: []}]}' > advisory.json

R=repos/static-web-server/static-web-server/security-advisories
gh api $R -X POST --input advisory.json                      # create (state: draft)
gh api $R/<GHSA> -X PATCH --input changes.json               # update only the fields that change
gh api $R/<GHSA> --jq .description | diff - description.md   # confirm what was stored
```

When fix work starts, and only on the maintainer's go-ahead:

```bash
gh api $R/<GHSA>/forks -X POST    # creates static-web-server/static-web-server-ghsa-xxxx-xxxx-xxxx
git remote add ghsa-xxxx git@github.com:static-web-server/static-web-server-ghsa-xxxx-xxxx-xxxx.git
```

GitHub Actions don't run in the private fork, so the fix gets no CI. Verify it locally, in a Linux container, or in a Windows VM, as the platform matrix in `pr-review/SKILL.md` requires.

## 5. After the Fix Merges

- Rewrite `### Fix` to the merged behavior. If the fix also repaired something outside the advisory (the HSTS fix made an explicit `--security-headers=false` apply with `--tls`), that belongs in the PR and commit, not in the advisory
- Re-run the PoC against the fixed build: it must fail, and every mitigation in the text must still be accurate
- Set `patched_versions` when the release exists. Publishing and requesting a CVE are the maintainer's calls
- A v2 backport gets its own fork PR, titled `(v2 backport)`
- `CHANGELOG.md` entries are written by the maintainer

## Worked Example

A complete draft as it should read after the fix merged. Severity Low, CWE-1188, `>= 3.0.0-beta.1, <= 3.0.0-beta.2`, credits `remediation_verifier`, no CVSS vector (insecure default, see step 2).

```markdown
Strict-Transport-Security with preload is sent over plain HTTP by default

### Summary

In builds with the TLS feature (all release binaries), the `--security-headers` option
is enabled by default and its header set includes
`Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`. The header is
emitted on every response, including responses served over plain HTTP.

### Details

The security-headers step does not know whether the connection is TLS, so HSTS is added
unconditionally. RFC 6797 requires that an HSTS host must not include the header in
responses sent over non-secure transport. The default value also includes
`includeSubDomains` and `preload`, which extend the policy to every subdomain and signal
eligibility for browser preload lists.

### PoC

- Run SWS without `--tls` (plain HTTP) with default settings.
- Request any file with `curl -i http://host:port/`.
- The response contains the `Strict-Transport-Security` header.

### Impact

Browsers that receive the header from a plain-HTTP deployment on a real hostname will
insist on HTTPS for that host and all its subdomains for two years. If HTTPS is not
available (or is served by another system with a different certificate), users are locked
out of the site and its subdomains, and the `preload` token can lead to the domain being
submitted to preload lists. This is an insecure default that can cause an outage from a
single misconfiguration.

### Mitigation until fixed

- On plain-HTTP deployments, start SWS with `--security-headers=false`, or override the
  header with a custom-header rule.
- Terminate TLS in front of SWS and strip the header at the proxy when the backend
  connection is plain.

### Fix

- Send `Strict-Transport-Security` only when the server runs with `--tls` (HTTP/1 and
  HTTP/2); plain HTTP and Unix socket servers keep the other security headers.
- Drop `preload` from the default value: `max-age=63072000; includeSubDomains`.
- Honour an explicit `--security-headers=false` together with `--tls`.
```

What makes it a model:

- **Summary** names the configuration (TLS builds, default on), the header, and the exact condition (plain HTTP) in two sentences
- **Details** separates the two defects (no transport knowledge; `preload` in the default) and cites the standard that forbids the behavior
- **PoC** is three steps anyone can run; no tooling, no payload
- **Impact** says who (browsers on a real hostname), what (locked out), how long (two years), and why it is worse than it looks (`preload`, subdomains)
- **Mitigation** gives two config-only options, both verified
- **Fix** lists behaviors, not code, and was rewritten after merge to match what shipped

## Checklist

- [ ] PoC reproduced on a built binary; observed and expected responses recorded
- [ ] Preconditions minimal; each one tested off
- [ ] Affected range verified per release; v2 LTS checked
- [ ] No existing advisory covers it; related ones cross-referenced
- [ ] Each CVSS metric justified by the reproduction, or `severity` set directly for an insecure default; CWEs specific
- [ ] Summary, Details, PoC, Impact, Mitigation until fixed, Fix present and factual
- [ ] Mitigations verified against the PoC
- [ ] Stored draft read back and matches the local text
- [ ] Nothing published, no CVE requested, no fork created without the maintainer's go-ahead
