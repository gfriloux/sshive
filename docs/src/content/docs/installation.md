---
title: Installation
description: Install SSHive on your Linux system with Nix.
---

SSHive distributes via **Nix flake** and is designed to work on NixOS and any Linux system with Nix.

## Prerequisites

- **Nix 2.13+** ([install Nix](https://nixos.org/download.html))
- **GPG 2.2+** — already included in most Linux distributions
- **SSH configured** — existing public/private keys in `~/.ssh/`

## Quick Installation

To run SSHive without a permanent installation:

```bash
nix run github:gfriloux/sshive
```

This is the best way to try it for the first time.

## Persistent Installation (Recommended)

To install SSHive in your user profile:

```bash
nix profile install github:gfriloux/sshive
```

Then launch:

```bash
sshive
```

The application automatically creates the `~/.config/sshive/` directory on first launch.

## Local Development

If you want to develop or compile from source:

```bash
git clone https://github.com/gfriloux/sshive.git
cd sshive
nix develop
```

You're now in a development shell with Rust, Cargo, and all dependencies. To run the application:

```bash
cargo run --release
```

## Initial Configuration

On first launch, SSHive creates the following structure:

```
~/.config/sshive/
├── config.yaml         # Services and keys configuration
├── audit.log           # Action log (chmod 0600)
└── secrets.yaml.gpg    # API tokens encrypted with GPG
```

The application also automatically scans `~/.ssh/` to discover your existing public keys (ed25519 and sk-ed25519).

### Select a GPG Key

On first launch, SSHive will ask you to select a GPG key to encrypt secrets. You can:

1. **Select an existing key** — if you already have a GPG key
2. **Create a new key** — SSHive can generate an Ed25519 key directly in the GUI
3. **Use the terminal** — run `gpg --generate-key` if you prefer

### Pinentry Backend

SSHive automatically detects your desktop environment and selects the correct passphrase entry agent:

- **GNOME** → `pinentry-gnome3`
- **KDE** → `pinentry-qt`
- **Other** → `pinentry-gtk-2` (fallback)

You can force a specific backend with:

```bash
export SSHIVE_PINENTRY=pinentry-gnome3
sshive
```

## Updates

To update to the latest version:

```bash
nix profile upgrade
```

Or if you use `nix run`:

```bash
nix run --update-input sshive github:gfriloux/sshive
```

## Troubleshooting

### SSHive won't start

- Check that GPG is working: `gpg --list-keys`
- Check the logs: `~/.config/sshive/audit.log`
- Make sure you have at least one GPG key available

### Passphrases not appearing

- Check that `pinentry` is installed: `which pinentry-gnome3` (or your backend)
- Try forcing a backend: `SSHIVE_PINENTRY=pinentry-gtk-2 sshive`

### Permission issues

SSHive creates all files with mode `0600` (read/write user only). If you see permission errors:

```bash
# Check the permissions of the config directory
ls -la ~/.config/sshive/
```

They should be `drw-------` (0700). If not:

```bash
chmod 700 ~/.config/sshive/
chmod 600 ~/.config/sshive/*
```

## Next Steps

Once installed, follow the [Quick Start](/quickstart/) to add your first service.
