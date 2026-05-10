# Changelog

All notable changes to SSHive are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-05-10

### Added

- **Pure-Rust SSH key re-encryption** — `ssh-key` crate (bcrypt-pbkdf + AES-256-CTR) replaces `ssh-keygen -N`; passphrase never appears in process arguments or `/proc/<pid>/cmdline`
- **GitHub API connector** — deploy, revoke, and verify ed25519 keys via `POST/DELETE/GET /user/keys`; duplicate detection returns `ApiKeyAlreadyPresent` instead of erroring
- **GitLab.com and self-hosted API connector** — same operations via `/api/v4/user/keys` with `Private-Token` header; self-hosted base URL configurable
- **Post-deploy verification** — after automatic deployment the deployer re-connects via SSH to confirm the key is accepted; result shown in deploy success screen
- **Key revocation UI** — detail panel lists all previous keys for a service with a Revoke button; calls the appropriate API or SSH revocation path
- **Health/Diagnostic view** — full-page table showing key age, protection status, pending deployment, and rotation overdue warnings; accessible from sidebar
- **`HealthSnapshot`** — pure computation from config + keys + protection, recomputed on every relevant event (no per-frame computation)
- **`ApiToken(SecretString)`** — wraps sensitive tokens; `Debug` prints `ApiToken(***)` only; backed by `secrecy` crate
- **`mlock` on `Passphrase`** — private 256-byte pinned buffer; locked with `libc::mlock`, zeroed on `Drop`
- **`HttpClient` trait** — injectable for tests; `ReqwestHttpClient` (rustls + webpki roots, no native TLS); `FakeHttpClient` for unit tests
- **Regression suite** — `regression_v020.rs` with 8 migration-safety tests covering config round-trip, key UUID stability, and health computation
- **`docs/CRYPTO.md`** — cryptographic policy (algorithms, key lengths, cipher modes, RNG, mlock)
- **`docs/THREAT_MODEL.md`** — 6 threat scenarios T1–T6 with mitigations and accepted residual risks

### Changed

- `DeployStep::Success` now carries `verified: bool`; UI distinguishes "✓ Connexion vérifiée" from "⚠ Vérification non effectuée"
- Sidebar gains a Health navigation item
- Service config model gains `deployments: Vec<Deployment>` (tracks deployed keys with date and remote ref) and `Config.health` (`rotation_warning_days`, default 90)

### Security

- Passphrase never passed via `-N` to `ssh-keygen` — pure Rust re-encryption via `ssh-key` crate closes the v0.2.0 known limitation
- All HTTP connections use rustls + WebPKI roots; no native TLS, no OpenSSL, no system certificate store
- API tokens stored in `secrets.yaml.gpg` and exposed only through `ApiToken::expose()` at call sites; never logged

### Fixed

- GitLab 400 response body is a nested object (`{"message": {"fingerprint": [...]}}`), not a plain string; now correctly detected as `ApiKeyAlreadyPresent` without unwrap panic

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
