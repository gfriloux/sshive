# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x   | ✅        |
| 0.1.x   | ✅        |

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
your machine. In v0.2.x it makes **outgoing SSH connections only**, initiated
explicitly by the user (key deployment, verification). It transmits no data
to third parties.

## Known Advisories

| Advisory | Crate | Status |
|----------|-------|--------|
| RUSTSEC-2023-0071 | `rsa` (via `ssh-key`) | No fix available upstream. SSHive does not expose RSA key operations — only ed25519 and sk-ed25519 keys are accepted. |
| RUSTSEC-2024-0436 | `paste` (via `iced`/`wgpu`) | Unmaintained transitive dep. No action possible until iced updates wgpu. |
