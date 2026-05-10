# SSHive

> One SSH key per service — made as simple as reusing the same one.

SSHive is a desktop SSH access manager. Most users maintain a single SSH key
for all their services — not from ignorance, but from lack of tooling. A single
compromise becomes total; rotation is nearly impossible to do cleanly.

SSHive makes the right behaviour as simple as the wrong one.

## Status

**v0.3.0** — security & connectors. Pure-Rust SSH key re-encryption (no
passphrase in process arguments), GitHub and GitLab API connectors, post-deploy
verification, key revocation, and a Health/Diagnostic view.

## Features

- **Service management** — create, edit, delete services; stored in `~/.config/sshive/config.yaml`
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2); passphrase collected via pinentry; re-encryption done entirely in-process (no subprocess, no passphrase in argv)
- **Key deployment** — automatic (`ssh-copy-id` or GitHub/GitLab API) or guided (copy-paste command)
- **GitHub & GitLab connectors** — deploy, revoke, and verify keys via API; supports GitLab self-hosted
- **Post-deploy verification** — confirms the deployed key is accepted by SSH/API after deployment
- **Key revocation** — remove old keys from servers or APIs directly from the detail panel
- **Key assignment** — attach any existing `~/.ssh/*.pub` to a service
- **Unprotected key detection** — warns when a private key has no passphrase and offers to add one
- **GPG-encrypted secrets** — API tokens encrypted with your own GPG key; never logged or printed
- **Health/Diagnostic view** — rotation age, protection status, pending deployment, per-service health level
- **3-column layout** — sidebar, list, detail/wizard panel with rotation age and fingerprint display

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
