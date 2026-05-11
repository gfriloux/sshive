# Changelog

All notable changes to SSHive are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-05-11

### Added

- **Pure-Rust SSH key re-encryption** — `ssh-key` crate (bcrypt-pbkdf + AES-256-CTR) replaces `ssh-keygen -N`; passphrase never appears in process arguments or `/proc/<pid>/cmdline`
- **GitHub API connector** — deploy, revoke, and verify ed25519 keys via `POST/DELETE/GET /user/keys`; duplicate detection returns `ApiKeyAlreadyPresent` instead of erroring
- **GitLab.com and self-hosted API connector** — same operations via `/api/v4/user/keys` with `Private-Token` header; self-hosted base URL configurable
- **Post-deploy verification** — after automatic deployment the deployer re-connects via SSH/API to confirm the key is accepted; result shown in deploy success screen
- **Pre-flight rotation check** — before starting key rotation on API services: local token format validation (prefix check for `ghp_`/`github_pat_`/`glpat-`), then a `GET /user` probe to confirm authorization; a `DeployBlocker` modal is shown before any key is generated; "Configurer le token" redirects directly to step 2 of the service edit form
- **Key revocation UI** — detail panel lists all previous keys for a service with a Revoke button; calls the appropriate API or SSH revocation path
- **Health/Diagnostic view** — full-page table showing key age, protection status, missing API token, pending deployment, and rotation overdue warnings; accessible from sidebar
- **`NoApiToken` health reason** — GitHub/GitLab services without a configured API token appear as Warning; escalated to Critical when combined with `RotationOverdue`; suppressed when the GPG vault is locked to avoid false positives
- **`HealthReason::HardwareKeyHandleNotBackedUp`** — Info-level health reason for sk-ed25519 keys where `backup_prompted` is false; label: "Fichier clef YubiKey à sauvegarder"
- **SK (YubiKey) rotation safety** — pre-rotation warning screen ("YubiKey doit rester branchée pendant toute l'opération"); verification is mandatory for sk-ed25519 keys (failed verify → error state, revocation blocked); verification failure on standard ed25519 produces `Success { verified: false }`
- **SK key handle backup prompt** — after generating an sk-ed25519 key, the key detail panel shows a persistent warning with the file path and a "Compris — j'ai sauvegardé" button that persists `backup_prompted: true` to config
- **ExternalCm deploy mode** — new `DeployMode::ExternalCm` for services whose `authorized_keys` are managed by NixOS, Ansible, Puppet, or similar tools; shows the public key to copy in a monospace block with step-by-step instructions; skips `ssh-copy-id` and post-deploy SSH verification; badge `EXT` in the service list; configurable via service edit step 2 (SSH generic services only)
- **Deploy mode indicator in service detail** — the CONNEXION section shows a "Déploiement" row: `Automatique (ssh-copy-id)`, `Guidé (commande à copier)`, or `Géré externalement (NixOS, Ansible…)`; visible even when no host/user is configured
- **Token guide in-app** — step 2 of the service form shows the URL and required scope for creating an API token (GitHub: `admin:public_key`; GitLab: `api`); "Ouvrir dans le navigateur" launches `xdg-open`; "Copier l'URL" copies to clipboard as fallback
- **Cancel button in service form** — "Annuler" button on all three wizard steps; closes immediately on a clean form; inline confirmation row when the form has been modified
- **Scrollable service detail panel** — the entire service detail panel is wrapped in a scrollable container; the "attach existing key" list is capped at 240px; a text filter appears above the list when more than 8 keys are present
- **Copy public key to clipboard** — "Copier la clef publique" button in the key detail header and on the fingerprint row of the service detail panel; label changes to "✓ Copié" for 2 seconds then reverts
- **Local audit log** — append-only `~/.config/sshive/audit.log` (chmod 0600) recording key generation, revocation, and service deletion with ISO timestamp
- **"Faire pivoter la clef" disabled for externally-managed private keys** — when `private_path` is `None` (key managed by sops, age, or similar), the rotation button is grayed out with an explanatory message
- **`ApiToken(SecretString)`** — wraps sensitive tokens; `Debug` prints `ApiToken(***)` only; backed by `secrecy` crate
- **`mlock` on `Passphrase`** — private 256-byte pinned buffer; locked with `libc::mlock`, zeroed on `Drop`
- **`HttpClient` trait** — injectable for tests; `ReqwestHttpClient` (rustls + webpki roots, no native TLS); `FakeHttpClient` for unit tests
- **`docs/CRYPTO.md`** — cryptographic policy (algorithms, key lengths, cipher modes, RNG, mlock)
- **`docs/THREAT_MODEL.md`** — 6 threat scenarios T1–T6 with mitigations and accepted residual risks
- **Regression suites** — `regression_v020.rs` (8 tests) and `regression_v030.rs` (10 tests) covering config round-trip, key UUID stability, health computation, and backward compatibility of all new fields

### Fixed

- **`active_key` silently wiped on service edit** — `SubmitServiceForm` now merges only form-controlled fields into the existing service, preserving `active_key`, `pending_key`, `created_at`, `last_rotation`, and `deployments`
- **`token_ref` orphaned on service edit** — editing a GitHub/GitLab service without retyping the token no longer clears `token_ref`; the existing reference is preserved
- **GitLab 400 response** — body is a nested object (`{"message": {"fingerprint": [...]}}`); now correctly detected as `ApiKeyAlreadyPresent` without unwrap panic

### Changed

- `DeployStep::Success` now carries `verified: bool`; UI distinguishes "✓ Connexion vérifiée" from "⚠ Vérification non effectuée"
- `DeployMode` gains a third variant `ExternalCm` (`"external-cm"` in YAML); backward-compatible (`#[serde(default)]` = `Automatic`)
- `SshKey` gains `backup_prompted: bool` (`#[serde(default)]`); backward-compatible
- `HealthSnapshot::compute` now accepts `Option<&Secrets>`; `None` suppresses `NoApiToken` checks when the GPG vault is locked
- `compute_service_health` accepts `has_api_token: Option<bool>`
- Sidebar gains a Health navigation item
- Service config model gains `deployments: Vec<Deployment>` and `Config.health` (`rotation_warning_days`, default 90)

### Removed

- **`ServiceType::Manual`** — removed; configs with `service_type: manual` are automatically migrated to `ssh-generic` via a serde alias; no data loss, no manual migration needed

### Security

- Passphrase never passed via `-N` to `ssh-keygen` — pure Rust re-encryption via `ssh-key` crate closes the v0.2.0 known limitation
- All HTTP connections use rustls + WebPKI roots; no native TLS, no OpenSSL, no system certificate store
- API tokens stored in `secrets.yaml.gpg` and exposed only through `ApiToken::expose()` at call sites; never logged
- GitHub token scope recommendation corrected to `admin:public_key` (includes `read:public_key` needed for verification)

### Compatibility

Forward and backward compatible with v0.2.0 config files. All new fields use `#[serde(default)]`. The `service_type: manual` alias ensures zero-friction migration for existing configs.

## [0.2.0] - 2026-05-10

### Added

- **Service CRUD** — create, edit and delete services from the UI; saved atomically to `config.yaml`
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2); passphrase mandatory ≥ 12 chars, collected via pinentry
- **Deploy flow** — automatic (`ssh-copy-id`) and guided (display command to copy/paste) modes
- **Key assignment** — attach any existing `~/.ssh/*.pub` key to a service directly from the detail panel
- **GPG setup** — first-launch wizard to select or create a GPG key for secrets encryption
- **Unprotected key detection** — detects SSH private keys without passphrase and offers to add one via pinentry
- **Pinentry integration** — passphrase collection via `pinentry-gtk-2`, `pinentry-gnome3`, or `pinentry-qt` (Assuan protocol); backend auto-selected from `XDG_CURRENT_DESKTOP` or `SSHIVE_PINENTRY` override
- **Process hardening** — `PR_SET_DUMPABLE` + `PR_SET_PTRACER` at startup to block core dumps and ptrace (Linux)
- **3-column layout** — sidebar, list (services or keys), detail/wizard panel
- **Key detail panel** — fingerprint, type, YubiKey badge, usage by service, security warning if unprotected

### Changed

- `active_key` on `Service` now references `SshKey` by stable UUID (was fingerprint string in v0.1.0); UUIDs are stabilised across restarts via fingerprint matching at scan time
- SSH key scanner now stores `public_path` on each discovered key
- Passphrase required (≥ 12 chars) for both ed25519 and sk-ed25519 key generation

### Security

- All `config.yaml` and secrets writes are atomic (tmp + chmod 0600 + rename)
- `prctl(PR_SET_DUMPABLE, 0)` prevents core dumps that could expose secrets
- All subprocess inputs validated before execution (hostname, username, port)
- No subprocess spawned via `sh -c` — arguments always passed as discrete tokens
- sk-ed25519 file protection detected via binary header parsing (no subprocess, no hardware interaction)
- Known limitation: passphrase passed via `-N` to `ssh-keygen` (briefly visible in `/proc/<pid>/cmdline` to same-UID processes); scheduled for PTY or library-based replacement in v0.3.0

### Fixed

- Key attachment to service now persists across restarts (UUID stabilised by fingerprint)
- sk-ed25519 keys correctly detected as unprotected when hardware is absent
- Previously silent subprocess failures now surface as visible errors in the UI

## [0.1.0] - 2026-05-10

### Added

- Load `~/.config/sshive/config.yaml` on startup (created empty if absent)
- Scan `~/.ssh/*.pub` for local SSH public keys (ed25519 and sk-ed25519)
- Service list view: name, type badge, fingerprint, rotation age, YubiKey/shared key badges
- SSH Keys view: fingerprint, key type, YubiKey indicator, comment, usage count
- Dark mode UI with Inter Variable and JetBrains Mono fonts
- Sidebar navigation (Services, SSH Keys, Settings placeholder)
- File permission check on `config.yaml` (warns if readable by other users)
- Symlink detection in `~/.ssh/` — ignored silently
- 1 MB size limit on `config.yaml` before parsing
- `#![forbid(unsafe_code)]` throughout
