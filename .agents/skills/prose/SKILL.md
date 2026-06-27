---
name: prose
description: Author or edit any prose for the Static Web Server (SWS) project — documentation, design docs, READMEs, PR descriptions, issue bodies, commit message bodies, or other human-readable text — following project writing conventions
---

# Writing SWS Prose

Load this skill whenever writing or editing prose for this project: documentation in `docs/`, READMEs, design docs, PR descriptions, issue bodies, commit message bodies, or any other human-readable markdown.

**When to load**: editing any file under `docs/`, writing a commit message body, drafting a PR description or issue body, or producing any user-facing markdown for the SWS project.

## Writing Style

- **Avoid ambiguous characters**: Do not use ambiguous Unicode characters, homoglyphs, and confusables in identifiers, code, code comments/docs or user input as they can lead to security issues.
- **Be fact-focused**: State what things are and what they do
- **Avoid buzzwords**: No "leverage", "synergy", "paradigm", etc.
- **Avoid fluff**: Every sentence should convey information
- **Avoid weasel words**: No "very", "really", "quite", "somewhat"
- **Avoid dramatic terms**: No "critical", "crucial", "vital", "essential" unless something will actually break
- **Avoid figurative metaphors**: Pick the literal word for the thing, not the analogy. "Blazing fast" → "sub-millisecond latency" or "serves files at line rate". "Battle-tested" → "used in production since 2019". Other recurring offenders: "under the hood" (just describe what's there), "out of the box" (just say "by default"), "first-class" (say what's actually supported). If you can't replace the metaphor with a literal noun or verb without losing meaning, you probably don't know what you mean yet.
- **Be direct**: Say what you mean without hedging
- **Use concrete examples**: Show, don't tell. Include CLI invocations and HTTP response snippets
- **Use active voice**: "SWS appends security headers to the response" not "Security headers are appended by SWS"
- **Use present tense**: Describe how the system works now, not how it was designed or how it will work
- **Document current behavior only**: Omit historical decisions, deprecated approaches, and planned future work

### Examples

**Bad**: "This feature is critical for ensuring optimal web server performance."

**Good**: "Static compression serves pre-compressed `.br` files from disk with zero CPU overhead, avoiding on-the-fly compression."

**Bad**: "SWS leverages advanced algorithms to enhance delivery."

**Good**: "SWS uses `accept-encoding` header negotiation to select the best compression algorithm (zstd, brotli, gzip, deflate) supported by the client."

**Bad**: "## Features that work out of the box"

**Good**: "## Enabled by default"

## Document Structure

- Start with what the thing is
- Explain why it exists (what problem it solves)
- Explain what it does
- Show how to use it (if applicable)
- Provide examples (CLI invocations, config snippets, HTTP headers)

## Feature Documentation

When documenting an SWS feature in `docs/content/features/`:

1. **One sentence summary** at the top of what the feature does
2. **Default state**: Whether enabled by default, and the flag to toggle it
3. **CLI example**: A `static-web-server` invocation with the relevant flags
4. **Behavior**: What happens when enabled vs disabled
5. **Related features**: Cross-link to features that interact (e.g., compression-static → compression)

## Terminology

- **SWS**: Static Web Server (the project). Use "SWS" after the first mention
- **Pre-compressed / static compression**: Serving `.br`/`.gz`/`.zst` files from disk
- **On-the-fly / dynamic compression**: Compressing responses in real-time
- **Root directory**: The `--root` directory from which files are served
- **Base path**: The canonicalized root directory used for path containment checks
