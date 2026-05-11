# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.3.x (incl. r1) | ✅ |
| 0.2.x   | ❌        |
| 0.1.x   | ❌        |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Report vulnerabilities by email to: **guillaume@friloux.me**

Include:
- A description of the vulnerability and its potential impact
- Steps to reproduce or proof-of-concept
- Affected versions

This project is maintained by one person in their spare time. There is no guaranteed response time. Reports will be addressed as promptly as circumstances allow.

## Scope

SSHive processes SSH public keys and configuration files stored locally on
your machine. In v0.3.x it makes **outgoing connections only**, initiated
explicitly by the user:

- SSH connections to servers you configure (deployment, verification, revocation)
- HTTPS connections to GitHub API (`api.github.com`) and GitLab API (`gitlab.com` or your self-hosted instance) for services of those types — only when you trigger a deploy/revoke/verify action, including a pre-flight `GET /user` probe before rotation starts
- No connections are made for services using `DeployMode::ExternalCm`

It transmits no data to third parties for telemetry or analytics.

## Local Audit Log

Destructive operations (key generation, revocation, service deletion) are
appended to `~/.config/sshive/audit.log` (permissions 0600). This file is
append-only and never transmitted. It is not rotated in v0.3.x.

## Known Advisories

| Advisory | Crate | Status |
|----------|-------|--------|
| RUSTSEC-2023-0071 | `rsa` (via `ssh-key`) | No fix available upstream. SSHive does not expose RSA key operations — only ed25519 and sk-ed25519 keys are accepted. |
| RUSTSEC-2024-0436 | `paste` (via `iced`/`wgpu`) | Unmaintained transitive dep. No action possible until iced updates wgpu. |
