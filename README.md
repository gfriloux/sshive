# SSHive

> One SSH key per service — made as simple as reusing the same one.

SSHive is a desktop SSH access manager built with Tauri v2 + Svelte 5 (Rust
backend). Most users maintain a single SSH key for all their services — not
from ignorance, but from lack of tooling. A single compromise becomes total;
rotation is nearly impossible to do cleanly.

SSHive makes the right behaviour as simple as the wrong one.

## Features

- **Service management** — GitHub, GitLab (cloud + self-hosted), SSH generic;
  create, edit, delete; stored in `~/.config/sshive/config.yaml`
- **SSH key generation** — ed25519 and sk-ed25519 (YubiKey/FIDO2); passphrase
  strength indicator; re-encryption via `ssh-key` crate (no passphrase in argv)
- **Key deployment** — automatic (GitHub/GitLab API or `ssh-copy-id`), guided
  (copy-paste command), or External CM (NixOS/Ansible snippet)
- **GitHub & GitLab connectors** — deploy, revoke, verify via REST API;
  GPG-encrypted tokens; in-app token guide with scope recommendation
- **Health view** — rotation age, protection status, missing API token, YubiKey
  backup reminder; per-service levels Critical / Warning / Info / OK
- **Settings** — rotation threshold, minimum passphrase length, GPG key,
  token management
- **Automatic `~/.ssh/*.pub` scan** — discovered keys shown alongside managed ones
- **Local audit log** — append-only `~/.config/sshive/audit.log` (0600)

## Requirements

- Linux x86_64
- GPG key (required at first launch for secret encryption)
- `ssh-keygen` and `ssh-copy-id` (for key generation and deployment)
- Nix with flakes enabled (recommended)

### System libraries (Tauri / WebKit)

`gtk3`, `webkitgtk_4_1`, `libsoup_3`, `dbus`, `openssl`, `glib`

## Development

```bash
nix develop         # enter the dev shell (includes cargo-tauri, nodejs)
just dev            # cargo-tauri dev — hot-reload frontend + backend
just build-bin      # release build, no packaging
just run            # launch the compiled binary
just screenshots    # generate doc screenshots via Playwright + mock mode
just docs-dev       # Astro dev server for the documentation site
```

The release binary is at `tauri-app/src-tauri/target/release/sshive-app`.

## Configuration

SSHive reads `~/.config/sshive/config.yaml` (created empty on first launch).
Services and keys can be managed entirely from the UI.

The file is created with permissions `0600`. SSHive warns if it is world-readable.

API tokens are stored in `~/.config/sshive/secrets.yaml.gpg`, encrypted with
your own GPG key.

## Documentation

Full documentation: <https://gfriloux.github.io/sshive>

Built with Astro + Starlight. Run locally: `just docs-dev`

## Privacy

SSHive makes **outgoing connections only**, initiated explicitly by you.
See [PRIVACY.md](PRIVACY.md) for details.

- No telemetry, no update checks, no analytics
- SSH connections to your own servers (deployment, verification, revocation)
- HTTPS to GitHub/GitLab APIs only when you trigger an action on those services

## Security

See [SECURITY.md](SECURITY.md) for the vulnerability reporting policy and
known advisories.

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
