---
title: Changelog
description: Version history and changes in SSHive.
---

All notable changes to SSHive are documented here.
Following the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.
Versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.0] - 2026-05-11

### Added

- **Design system** — unified color palette with 4 depth levels (`BACKGROUND_BASE/PANEL/CARD/ELEVATED`), spacing grid (4/8/12/16/20/24/32px), border radius scale (4/6/8px), locked typography (10–20px) ; `TEXT_MONO` for fingerprints and paths ; `FONT_BOLD` for critical counters
- **Top bar** — full-width header spanning three columns ; displays app name and version from `CARGO_PKG_VERSION` ; eliminates vertical misalignment between sidebar and page titles
- **Sidebar redesign** — Unicode icons (▣◈◉◎), "NAVIGATION" section label, 2px left accent bar for active element (consistent with list selection), borderless buttons (radius 0), health alert badge (pulsing red if Critical services), counters for Services and SSH Keys, Settings pinned at bottom
- **Service list redesign** — two-line rows (service name + hostname/URL subtitle) ; status badge column (8px, color-coded from `HealthSnapshot`) ; fixed column widths replacing unstable `FillPortion` — eliminates overflow/overlap glitch at medium window ; table header with `BACKGROUND_PANEL`
- **Detail panel max-width** — detail column limited to 720px, left-aligned ; stops unlimited growth on wide windows
- **Settings page** — replaces "coming soon" placeholder ; three sections: Security (rotation threshold, minimum passphrase length), GPG (active key + Change button, pinentry backend selector), Data (audit log button, config file paths)
- **Animations** — spinner `◐◓◑◒` at 200ms on Preflight / AutoDeploying / Verifying / Generating ; critical health pulse at 30Hz via `sin(t)` opacity ; copy feedback fade-in ~100ms ; subscriptions gated on need (inactive app = `Subscription::none()`)
- **GPG setup rework** — title changed to "Protect your sensitive data" ; two-sentence concept explanation before key selection ; "Create a GPG key" button on no-key path triggers in-app Ed25519/Cv25519 key generation ; terminal fallback kept as secondary option
- **"Renew SSH key"** — action renamed from "Rotate key" to clearer English
- **Copy key button visible at rest** — "Copy public key" now has 1px border and card background at rest ; no longer invisible until hover
- **Empty states** — service list: "Add my first service" CTA button with explanation ; SSH keys: redirect to Services with button

### Fixed

- Service list column overflow / text overlap at medium window widths — replaced `FillPortion` fingerprint column with fixed-width two-line name cell
- Header vertical misalignment across three columns — structurally resolved by top bar ; `HEADER_HEIGHT = 64.0` constant for remaining column sub-headers
- Sidebar border rendered on all 4 sides — replaced with unique `rule::vertical` separator

### Changed

- `Config` gains `security: SecurityConfig` (`min_passphrase_len: u8`, default 12) — `#[serde(default)]`, backward-compatible
- `GpgConfig` gains `pinentry_backend: Option<String>` — `#[serde(default)]`, backward-compatible
- Deploy mode cards: "Uses: ssh-copy-id" and "Useful if: bastion…" removed ; rewritten in clear benefit language
- "Reference in secrets: {token_ref}" removed from service form step 2

### Compatibility

Forward and backward compatible with all previous config files. All new fields use `#[serde(default)]`.

## [0.3.0] - 2026-05-11

### Added

- **Pure-Rust SSH key re-encryption** — `ssh-key` crate (bcrypt-pbkdf + AES-256-CTR) replaces `ssh-keygen -N` ; passphrase never in process arguments or `/proc/<pid>/cmdline`
- **GitHub API connector** — deploys, revokes, and verifies ed25519 keys via `POST/DELETE/GET /user/keys` ; duplicate detection returns `ApiKeyAlreadyPresent` instead of error
- **GitLab.com and self-hosted API connector** — same operations via `/api/v4/user/keys` with `Private-Token` header ; self-hosted base URL configurable
- **Post-deploy verification** — after automatic deployment the deployer reconnects via SSH/API to confirm key accepted ; result displayed in deployment success screen
- **Pre-flight rotation check** — before rotation on API services: validate local token format (check `ghp_`/`github_pat_`/`glpat-` prefix), then probe `GET /user` to confirm authorization ; `DeployBlocker` modal shown before key generation ; "Configure token" redirects directly to service edit step 2
- **Key revocation UI** — detail panel lists all previous keys for a service with Revoke button ; calls appropriate API or SSH revocation path
- **Health/Diagnostic view** — full-page table showing key age, protection status, missing API token, pending deployment, rotation alerts ; accessible from sidebar
- **`NoApiToken` health reason** — GitHub/GitLab services without configured API token appear Warning ; escalated Critical when combined with `RotationOverdue` ; suppressed when GPG vault locked to avoid false positives
- **`HealthReason::HardwareKeyHandleNotBackedUp`** — Info-level health reason for sk-ed25519 keys where `backup_prompted` is false ; label: "YubiKey file to back up"
- **SK (YubiKey) rotation safety** — pre-rotation warning screen ("YubiKey must stay connected throughout the operation") ; mandatory verification for sk-ed25519 keys (verify failed → error state, revocation blocked) ; verify fail on standard ed25519 produces `Success { verified: false }`
- **SK key handle backup prompt** — after sk-ed25519 generation, detail panel displays persistent warning with file path and "Understood — I've backed up" button that persists `backup_prompted: true` to config
- **ExternalCm deploy mode** — new `DeployMode::ExternalCm` for services whose `authorized_keys` managed by NixOS, Ansible, Puppet, similar ; displays public key to copy in monospace block with step-by-step instructions ; skips `ssh-copy-id` and post-deploy SSH verification ; `EXT` badge in service list ; configurable via service edit step 2 (generic SSH services only)
- **Deploy mode indicator in service detail** — CONNECTION section shows "Deployment" line: `Automatic (ssh-copy-id)`, `Guided (command to copy)`, or `Externally Managed (NixOS, Ansible…)` ; visible even if host/user unconfigured
- **Token guide in-app** — service form step 2 displays URL and required scope for API token creation (GitHub: `admin:public_key` ; GitLab: `api`) ; "Open in browser" launches `xdg-open` ; "Copy URL" copies clipboard fallback
- **Cancel button in service form** — "Cancel" button on three wizard steps ; cleanly closes form ; inline row confirmation when form modified
- **Scrollable service detail panel** — entire service detail panel wrapped in scrollable container ; "attach existing key" list limited to 240px ; text filter above list when > 8 keys present
- **Copy public key to clipboard** — "Copy public key" button in key detail header and fingerprint row in service detail ; label changes to "✓ Copied" for 2 seconds then reverts
- **Local audit log** — append-only `~/.config/sshive/audit.log` (chmod 0600) recording key generation, revocation, service deletion with ISO timestamp
- **"Renew SSH key" disabled for externally-managed private keys** — when `private_path` is `None` (key managed by sops, age, similar), rotation button grayed with explanatory message
- **`ApiToken(SecretString)`** — wraps sensitive tokens ; `Debug` displays `ApiToken(***)` only ; backed by `secrecy` crate
- **`mlock` on `Passphrase`** — private 256-byte pinned buffer ; locked with `libc::mlock`, zeroized on `Drop`
- **`HttpClient` trait** — injectable for tests ; `ReqwestHttpClient` (rustls + webpki roots, no native TLS) ; `FakeHttpClient` for unit tests
- **`docs/CRYPTO.md`** — cryptographic policy (algorithms, key lengths, cipher modes, RNG, mlock)
- **`docs/THREAT_MODEL.md`** — 6 threat scenarios T1–T6 with mitigations and accepted residual risks
- **Regression suites** — `regression_v020.rs` (8 tests) and `regression_v030.rs` (10 tests) covering config round-trip, key UUID stability, health computation, backward compatibility of all new fields

### Fixed

- **`active_key` silently wiped on service edit** — `SubmitServiceForm` merge now updates only form-controlled fields on existing service, preserving `active_key`, `pending_key`, `created_at`, `last_rotation`, `deployments`
- **`token_ref` orphaned on service edit** — editing GitHub/GitLab service without retyping token no longer hides `token_ref` ; existing reference preserved
- **GitLab 400 response** — body is nested object (`{"message": {"fingerprint": [...]}}`); now correctly detects `ApiKeyAlreadyPresent` without unwrap panic

### Changed

- `DeployStep::Success` now carries `verified: bool` ; UI distinguishes "✓ Connection verified" from "⚠ Verification not performed"
- `DeployMode` gains third variant `ExternalCm` (`"external-cm"` YAML) ; backward-compatible (`#[serde(default)]` = `Automatic`)
- `SshKey` gains `backup_prompted: bool` (`#[serde(default)]`) ; backward-compatible
- `HealthSnapshot::compute` now accepts `Option<&Secrets>` ; `None` suppresses `NoApiToken` checks when GPG vault locked
- `compute_service_health` accepts `has_api_token: Option<bool>`
- Sidebar gains Health navigation item
- Service config model gains `deployments: Vec<Deployment>` and `Config.health` (`rotation_warning_days`, default 90)

### Removed

- **`ServiceType::Manual`** — removed ; configs with `service_type: manual` auto-migrated to `ssh-generic` via serde alias ; no data loss, no manual migration

### Security

- Passphrase never passed via `-N` to `ssh-keygen` — pure Rust re-encryption via `ssh-key` crate closes v0.2.0 known limitation
- All HTTP connections use rustls + WebPKI roots ; no OpenSSL, no native TLS, no system certificate store
- API tokens stored in `secrets.yaml.gpg` and exposed only via `ApiToken::expose()` at call sites ; never logged
- Corrected GitHub token scope to `admin:public_key` (includes `read:public_key` needed for verification)

### Compatibility

Forward and backward compatible with v0.2.0 config files. All new fields use `#[serde(default)]`. The `service_type: manual` alias ensures zero-friction migration for existing configs.

## [0.2.0] - 2026-05-10

### Added

- **Service CRUD** — create, edit, delete services from UI ; saved atomically to `config.yaml`
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2) ; passphrase mandatory ≥ 12 chars, collected via pinentry
- **Deploy flow** — automatic (`ssh-copy-id`) and guided (display command to copy/paste) modes
- **Key assignment** — attach existing `~/.ssh/*.pub` key to service directly from detail panel
- **GPG setup** — first-launch wizard to select or create GPG key for encrypting secrets
- **Unprotected key detection** — detects SSH private keys without passphrase and offers to add one via pinentry
- **Pinentry integration** — collects passphrase via `pinentry-gtk-2`, `pinentry-gnome3`, or `pinentry-qt` (Assuan protocol) ; backend auto-selected from `XDG_CURRENT_DESKTOP` or `SSHIVE_PINENTRY` override
- **Process hardening** — `PR_SET_DUMPABLE` + `PR_SET_PTRACER` at startup to block core dumps and ptrace (Linux)
- **3-column layout** — sidebar, list (services or keys), detail/wizard panel
- **Key detail panel** — fingerprint, type, YubiKey badge, service usage, security warning if unprotected

### Changed

- `active_key` on `Service` now references `SshKey` by stable UUID (was string fingerprint in v0.1.0) ; UUIDs stabilized across restarts via fingerprint matching at scan time
- SSH key scanner now stores `public_path` on discovered key
- Passphrase required (≥ 12 chars) for ed25519 and sk-ed25519 key generation

### Security

- All `config.yaml` and secrets writes atomic (tmp + chmod 0600 + rename)
- `prctl(PR_SET_DUMPABLE, 0)` prevents core dumps that could expose secrets
- All subprocess inputs validated before execution (hostname, username, port)
- No subprocess spawned via `sh -c` — arguments always passed as discrete tokens
- SK-Ed25519 protection detected via binary header parsing (no subprocess, no hardware interaction)
- Known limitation: passphrase passed via `-N` to `ssh-keygen` (briefly visible in `/proc/<pid>/cmdline` to same-UID processes) ; planned PTY or library-based replacement v0.3.0

### Fixed

- Key attachment to service persists across restarts (UUID stabilized by fingerprint)
- SK-Ed25519 keys correctly detected unprotected when hardware absent
- Previously silent subprocess failures now surface as visible UI errors

## [0.1.0] - 2026-05-10

### Added

- Load `~/.config/sshive/config.yaml` on startup (created empty if absent)
- Scan `~/.ssh/*.pub` local public SSH keys (ed25519 and sk-ed25519)
- Service list view: name, type badge, fingerprint, rotation age, YubiKey/shared key badges
- SSH Keys view: fingerprint, key type, YubiKey indicator, comment, usage count
- Dark mode UI with Inter Variable and JetBrains Mono fonts
- Sidebar navigation (Services, SSH Keys, Settings placeholder)
- File permission check on `config.yaml` (alert if readable by other users)
- Symlink detection in `~/.ssh/` — silently ignored
- 1 MB size limit on `config.yaml` before parsing
- `#![forbid(unsafe_code)]` throughout
