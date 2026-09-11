# Security Policy

## Supported Versions

The following Static Web Server versions are currently being supported with security updates.

| Version | Supported |
| -- | -- |
| 3.x.x | :white_check_mark: (development) |
| 2.x.x | :white_check_mark: (LTS since 2026-07-31) |
| 1.x.x | :x: (deprecated due to [End of Life](https://github.com/static-web-server/static-web-server/releases/tag/v1.19.4)) |

## Report a security issue

The Static Web Server project team welcomes security reports and is committed to providing prompt attention to security issues. Security issues should be reported **privately** by sending a direct message (DM) to the principal maintainer ([@joseluisq](https://github.com/joseluisq)) on either [Keybase](https://keybase.io/joseluisq) (preferred) or [Discord Server](https://discord.gg/VWvtZeWAA7). Security issues should **not** be reported via any *public* GitHub Issue tracker, chat or similar.

## Vulnerability coordination

Remediation of security vulnerabilities is prioritized by the project team. The project team coordinates remediation with third-party project stakeholders via [GitHub Security Advisories](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/about-repository-security-advisories). Third-party stakeholders may include the reporter of the issue, affected direct or indirect users of Static Web Server, and maintainers of upstream dependencies if applicable.

Downstream project maintainers and Static Web Server users can request participation in the coordination of applicable security issues by sending your contact email address, GitHub username(s) and any other salient information to the project team. Participation in security issue coordination processes is at the discretion of the Static Web Server team.

## Security advisories

The project team is committed to transparency in the security issue disclosure process. The Static Web Server team announces security issues via [GitHub Release Notes](https://github.com/static-web-server/static-web-server/releases) and [GitHub Advisory Database](https://github.com/advisories).

## Accepted risks

The following advisories are known, assessed and accepted for the `2.x` (LTS) branch. They are listed in [`.cargo/audit.toml`](.cargo/audit.toml) so that dependency scanners have an explanation to point at.

| Advisory | Crate | Reason |
| -- | -- | -- |
| [RUSTSEC-2026-0258](https://rustsec.org/advisories/RUSTSEC-2026-0258.html) | `h2 0.3.x` (via `hyper 0.14`) | Low-severity DoS with no fix available for `2.x`: the patch only exists in `h2 0.4.16+`, which requires `http 1.x` that `hyper 0.14` cannot use, and there is no upstream `0.3.x` backport. Mitigated in `2.x` by enforcing HTTP/1 on the plaintext listeners, so HTTP/2 is only reachable over TLS via `--http2`. Resolved by the `hyper 1.x` migration in `3.x`. |
