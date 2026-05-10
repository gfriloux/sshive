# Changelog

All notable changes to SSHive are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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
