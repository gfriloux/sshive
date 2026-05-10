# SSHive

> One SSH key per service — made as simple as reusing the same one.

SSHive is a desktop SSH access manager. Most users maintain a single SSH key
for all their services — not from ignorance, but from lack of tooling. A single
compromise becomes total; rotation is nearly impossible to do cleanly.

SSHive makes the right behaviour as simple as the wrong one.

## Status

**v0.2.0** — write mode. Create and manage services, generate SSH keys
(ed25519 and sk-ed25519/YubiKey), deploy them via `ssh-copy-id`, and encrypt
secrets with your GPG key.

## Features

- **Service management** — create, edit, delete services; stored in `~/.config/sshive/config.yaml`
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2); passphrase collected via pinentry
- **Key deployment** — automatic (`ssh-copy-id`) or guided (copy-paste command)
- **Key assignment** — attach any existing `~/.ssh/*.pub` to a service
- **Unprotected key detection** — warns when a private key has no passphrase and offers to add one
- **GPG-encrypted secrets** — API tokens and secrets encrypted with your own GPG key
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

SSHive makes **outgoing connections only**, initiated explicitly by you
(key deployment to your own servers). See [PRIVACY.md](PRIVACY.md) for details.

- No telemetry, no update checks, no analytics
- Outgoing SSH connections only (to servers you configure)
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
