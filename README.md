# SSHive

> One SSH key per service — made as simple as reusing the same one.

SSHive is a desktop SSH access manager. Most users maintain a single SSH key
for all their services — not from ignorance, but from lack of tooling. A single
compromise becomes total; rotation is nearly impossible to do cleanly.

SSHive makes the right behaviour as simple as the wrong one.

## Features

- **Service management** — create, edit, delete services; stored in `~/.config/sshive/config.yaml`; Cancel button on all form steps with dirty-state confirmation
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2); passphrase collected via pinentry; re-encryption done entirely in-process via the `ssh-key` crate (no subprocess, no passphrase in process arguments)
- **Key deployment** — automatic (`ssh-copy-id` or GitHub/GitLab API) or guided (copy-paste command); pre-flight token format + authorization check before any key is generated
- **ExternalCm mode** — for services whose `authorized_keys` are managed by NixOS, Ansible, Puppet, or similar: displays the public key to copy with step-by-step instructions, skips `ssh-copy-id` and post-deploy verification; configurable per service
- **GitHub & GitLab connectors** — deploy, revoke, and verify keys via REST API; supports GitLab self-hosted; in-app token creation guide with clickable URL (`xdg-open`) and scope recommendation (`admin:public_key` / `api`)
- **Post-deploy verification** — confirms the deployed key is accepted by SSH/API after deployment; mandatory and non-bypassable for sk-ed25519/YubiKey keys
- **SK (YubiKey) rotation safety** — pre-rotation warning screen; mandatory verify-before-revoke; backup prompt for key handle files (`HealthReason::HardwareKeyHandleNotBackedUp`)
- **Key revocation** — remove old keys from servers or APIs directly from the detail panel
- **Key assignment** — attach any existing `~/.ssh/*.pub` to a service; scrollable picker (240px cap) with text filter for large key collections
- **Copy public key** — "Copier la clef publique" button in key detail and service detail panels; clipboard feedback "✓ Copié" for 2 seconds
- **Unprotected key detection** — warns when a private key has no passphrase and offers to add one via pinentry
- **GPG-encrypted secrets** — API tokens encrypted with your own GPG key; never logged or printed; `ApiToken` type Debug-obfuscated
- **Health/Diagnostic view** — rotation age, protection status, pending deployment, missing API token, YubiKey backup status; per-service health levels (Critical / Warning / Info / OK)
- **Deploy mode indicator** — service detail panel shows the deploy mode (Automatique / Guidé / Géré externalement) in the CONNEXION section
- **Local audit log** — append-only `~/.config/sshive/audit.log` (0600) for key generation, revocation, and service deletion
- **3-column layout** — sidebar, list, fully scrollable detail/wizard panel with rotation age and fingerprint display

## Requirements

- Linux (x86_64 or aarch64)
- `gpg` and a GPG key (required at first launch)
- `pinentry-gtk-2`, `pinentry-gnome3`, or `pinentry-qt` (for passphrase dialogs)
- `ssh-keygen` and `ssh-copy-id` (for key generation and deployment)
- Nix with flakes enabled (recommended) — or a Rust toolchain with the system
  libraries listed below

### System libraries (Wayland/X11)

`wayland`, `libxkbcommon`, `libGL`, `vulkan-loader`, `gtk3`

## Build

```bash
nix develop       # enter the dev shell
just build        # cargo build --release
```

The binary is at `target/release/sshive`.

## Configuration

SSHive reads `~/.config/sshive/config.yaml`. The file is created empty on
first launch. Services and keys can be managed entirely from the UI.

The file must have permissions `0600`. SSHive warns on startup if it is
world-readable.

## Privacy

SSHive makes **outgoing connections only**, initiated explicitly by you.
See [PRIVACY.md](PRIVACY.md) for details.

- No telemetry, no update checks, no analytics
- SSH connections to your own servers (deployment, verification, revocation)
- HTTPS to GitHub/GitLab APIs only when you trigger an action on those service types (rustls, no native TLS)
- Config: `~/.config/sshive/config.yaml` (permissions 0600)
- Secrets: `~/.config/sshive/secrets.yaml.gpg` (GPG-encrypted)

## Security

See [SECURITY.md](SECURITY.md) for the vulnerability reporting policy and
known advisories.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
