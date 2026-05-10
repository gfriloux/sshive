# SSHive

> One SSH key per service — made as simple as reusing the same one.

SSHive is a desktop SSH access manager. Most users maintain a single SSH key
for all their services — not from ignorance, but from lack of tooling. A single
compromise becomes total; rotation is nearly impossible to do cleanly.

SSHive makes the right behaviour as simple as the wrong one.

## Status

**v0.1.0** — read-only. Loads your existing SSH config and keys, displays
them with rotation age indicators and shared-key warnings. No writes, no
network connections.

## Requirements

- Linux (x86_64 or aarch64)
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
first launch. The expected format:

```yaml
services:
  - name: "GitHub perso"
    service_type: github   # github | gitlab | gitlab-self-hosted | ssh-generic | manual
    active_key: "SHA256:…"
    created_at: "2025-01-15"
    last_rotation: null

keys:
  - fingerprint: "SHA256:…"
    key_type: ed25519      # ed25519 | sk-ed25519
    yubikey: false
    created_at: "2025-01-15"
    comment: "sshive/github_perso/2025-01-15"
```

The file must have permissions `0600`. SSHive warns on startup if it is
world-readable.

## Privacy

SSHive processes data **exclusively on your local machine**.

- No telemetry
- No network connections (v0.1.x)
- No data leaves the filesystem
- Config and keys stay in `~/.config/sshive/`

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
