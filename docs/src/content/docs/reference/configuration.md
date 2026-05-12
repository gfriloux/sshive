---
title: Configuration
description: Complete YAML schema and SSHive configuration structure.
---

SSHive stores its configuration in a YAML file located at:

```
~/.config/sshive/config.yaml
```

This file contains your list of services, SSH keys, and global preferences.

## Location and Permissions

The `config.yaml` file is automatically created on first launch with permissions `0600` (read/write user only).

SSHive displays an alert if permissions are too permissive (readable by other users).

## Overall Structure

```yaml
security:
  min_passphrase_len: 12
  rotation_warning_days: 90

gpg:
  key_id: "ABC123DEF456..."
  pinentry_backend: "gnome3"

services:
  - name: "GitHub"
    service_type: "github"
    # ... (see services section below)

ssh_keys:
  - uuid: "550e8400-e29b-41d4-a716-446655440000"
    # ... (see keys section below)
```

## Security Section

```yaml
security:
  min_passphrase_len: 12              # Minimum passphrase length (default 12)
  rotation_warning_days: 90           # Days before rotation alert (default 90)
```

**Default values:**
- `min_passphrase_len: 12` — compliant with NIST SP 800-63B
- `rotation_warning_days: 90` — alert after 90 days without rotation

## GPG Section

```yaml
gpg:
  key_id: "ABC123DEF456789..."       # Short or long ID of GPG key
  pinentry_backend: "gnome3"          # gnome3, qt, gtk2 (optional)
```

**Explanations:**

- `key_id` — unique identifier of your GPG key used to encrypt secrets
  - Can be short ID (16 hex characters) or long ID (40 characters)
  - Displayed in **Settings** → **GPG**

- `pinentry_backend` — passphrase entry agent (optional, auto-detected)
  - `gnome3` — GNOME (used on Ubuntu GNOME, Fedora GNOME…)
  - `qt` — KDE Plasma
  - `gtk2` — universal fallback
  - Omit this field to let SSHive auto-detect from `XDG_CURRENT_DESKTOP`

## Services Section

Each service represents an SSH destination (GitHub, GitLab, server).

### General Service Structure

```yaml
services:
  - name: "GitHub"
    service_type: "github"
    hostname: "github.com"
    username: "git"
    port: 22
    active_key: "550e8400-e29b-41d4-a716-446655440000"
    token_ref: "secrets.github_token"
    created_at: "2026-05-11T10:34:22Z"
    last_rotation: "2026-05-11T10:34:22Z"
    pending_key: null
    deployments: []
    comment: "SSH key for GitHub public repositories"
```

### Required Fields

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `name` | string | `"GitHub"` | Unique name, displayed in list |
| `service_type` | enum | `"github"` | `github` \| `gitlab` \| `gitlab-self-hosted` \| `ssh-generic` |
| `hostname` | string | `"github.com"` | Domain or IP of the service |

### Type-Specific Fields

**GitHub:**

```yaml
service_type: "github"
hostname: "github.com"           # Auto-filled, normally "github.com"
username: "git"                  # Auto-filled
port: 22                         # Auto-filled
token_ref: "secrets.github_token"  # Reference to encrypted token in secrets.yaml.gpg
deploy_mode: "automatic"         # automatic | guided
```

**GitLab.com:**

```yaml
service_type: "gitlab"
hostname: "gitlab.com"           # Auto-filled
username: "git"                  # Auto-filled
port: 22
token_ref: "secrets.gitlab_token"
deploy_mode: "automatic"
```

**Self-hosted GitLab:**

```yaml
service_type: "gitlab-self-hosted"
hostname: "https://gitlab.example.com"  # Full URL
username: "git"
port: 22
token_ref: "secrets.gitlab_custom_token"
deploy_mode: "automatic" | "guided"
```

**Generic SSH:**

```yaml
service_type: "ssh-generic"
hostname: "deploy.example.com"
username: "deploy"
port: 2222
deploy_mode: "automatic" | "guided" | "external-cm"
```

### Optional Fields (All Services)

| Field | Type | Default | Meaning |
|-------|------|---------|---------|
| `username` | string | `"git"` (GitHub/GitLab) | SSH user |
| `port` | integer | `22` | SSH port |
| `active_key` | string (UUID) | `null` | UUID of attached SSH key |
| `pending_key` | string (UUID) | `null` | UUID of key being deployed |
| `token_ref` | string | `null` | Reference to API token in secrets.yaml.gpg |
| `deploy_mode` | enum | `"automatic"` | `automatic` \| `guided` \| `external-cm` |
| `created_at` | ISO 8601 | auto | Service creation timestamp |
| `last_rotation` | ISO 8601 | auto | Last key rotation timestamp |
| `deployments` | array | `[]` | Deployment history (see below) |
| `comment` | string | `""` | Internal note on service |

### Deployments Field (History)

```yaml
deployments:
  - timestamp: "2026-05-11T14:35:22Z"
    status: "success"
    verified: true
    action: "deploy"  # deploy | revoke
    error_message: null
  - timestamp: "2026-05-10T09:15:33Z"
    status: "failure"
    verified: false
    action: "revoke"
    error_message: "SSH connection timed out"
```

## SSH Keys Section

Each SSH key is stored locally with metadata.

```yaml
ssh_keys:
  - uuid: "550e8400-e29b-41d4-a716-446655440000"
    fingerprint: "SHA256:abc123def456..."
    public_path: "/home/user/.ssh/id_sshive_github.pub"
    private_path: "/home/user/.ssh/id_sshive_github"
    key_type: "ed25519"
    key_size: 256
    created_at: "2026-05-11T10:34:22Z"
    protected: true
    backup_prompted: false
    comment: "Generated by SSHive"
```

### Key Fields

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `uuid` | string (UUID v4) | `"550e8400-e29b-41d4-a716-446655440000"` | Stable unique identifier (based on fingerprint) |
| `fingerprint` | string | `"SHA256:abc123def456789..."` | SHA256 fingerprint of public key |
| `public_path` | string | `"/home/user/.ssh/id_sshive_github.pub"` | Path to `.pub` file |
| `private_path` | string | `"/home/user/.ssh/id_sshive_github"` | Path to private key file |
| `key_type` | enum | `"ed25519"` | `ed25519` \| `sk-ed25519` |
| `key_size` | integer | `256` | Key bits (always 256 for ed25519) |
| `created_at` | ISO 8601 | `"2026-05-11T10:34:22Z"` | Generation timestamp |
| `protected` | boolean | `true` | Key encrypted with passphrase |
| `backup_prompted` | boolean | `false` | For SK-Ed25519: backup confirmed |
| `comment` | string | `"GitHub deployment key"` | Optional comment |

## Secrets (secrets.yaml.gpg)

API tokens and other secrets are encrypted in a separate file:

```
~/.config/sshive/secrets.yaml.gpg
```

This file should **never** be edited manually. SSHive manages it completely.

Decrypted structure (don't touch):

```yaml
tokens:
  github_token:
    api_token: "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  gitlab_token:
    api_token: "glpat-xxxxxxxxxxxxxxxxxxxxxxxx"
  custom_gitlab_token:
    api_token: "glpat-xxxxxxxxxxxxxxxxxxxxxxxx"
```

## Audit Log

Every action is recorded in:

```
~/.config/sshive/audit.log
```

Format:

```
2026-05-11T14:35:22Z [sshive] Generated SSH key ed25519 for service GitHub (fingerprint: abc123...)
2026-05-11T14:35:45Z [sshive] Deployed key to GitHub via API (verified: true)
2026-05-11T16:20:10Z [sshive] Rotated key for service Production
2026-05-12T09:10:33Z [sshive] Revoked key from GitLab
```

Logs **never** contain API tokens or passphrases.

## Complete Example

```yaml
security:
  min_passphrase_len: 12
  rotation_warning_days: 90

gpg:
  key_id: "ABC123DEF456789ABCDEF456789ABC123"
  pinentry_backend: "gnome3"

services:
  - name: "GitHub"
    service_type: "github"
    hostname: "github.com"
    username: "git"
    port: 22
    active_key: "550e8400-e29b-41d4-a716-446655440000"
    token_ref: "secrets.github_token"
    created_at: "2026-05-11T10:34:22Z"
    last_rotation: "2026-05-11T10:34:22Z"
    pending_key: null
    deploy_mode: "automatic"
    deployments:
      - timestamp: "2026-05-11T14:35:22Z"
        status: "success"
        verified: true
        action: "deploy"
        error_message: null
    comment: "SSH key for GitHub public repositories"

  - name: "Production"
    service_type: "ssh-generic"
    hostname: "deploy.prod.example.com"
    username: "deploy"
    port: 2222
    active_key: "660f9511-f30c-52e5-b827-557766551111"
    token_ref: null
    created_at: "2026-05-10T08:20:15Z"
    last_rotation: "2026-05-10T08:20:15Z"
    pending_key: null
    deploy_mode: "automatic"
    deployments: []
    comment: "Production server deploy key"

ssh_keys:
  - uuid: "550e8400-e29b-41d4-a716-446655440000"
    fingerprint: "SHA256:abc123def456789abcdef456789abc123def456789abc12"
    public_path: "/home/user/.ssh/id_sshive_github.pub"
    private_path: "/home/user/.ssh/id_sshive_github"
    key_type: "ed25519"
    key_size: 256
    created_at: "2026-05-11T10:34:22Z"
    protected: true
    backup_prompted: false
    comment: "GitHub"

  - uuid: "660f9511-f30c-52e5-b827-557766551111"
    fingerprint: "SHA256:def456789abc123def456789abc123def456789abc123de"
    public_path: "/home/user/.ssh/id_sshive_production.pub"
    private_path: "/home/user/.ssh/id_sshive_production"
    key_type: "ed25519"
    key_size: 256
    created_at: "2026-05-10T08:20:15Z"
    protected: true
    backup_prompted: false
    comment: "Production deploy key"
```

## Manual Management (Advanced)

SSHive displays an alert if `config.yaml` is modified outside the application. Manual edits can break consistency.

**If you manually edit:**

1. Make a backup: `cp ~/.config/sshive/config.yaml ~/.config/sshive/config.yaml.bak`
2. Edit with your editor
3. Restart SSHive and check alerts
4. Check the **Health** page to verify consistency

**Valid use cases:**

- Add a comment
- Fix a typo in `hostname` or `username`
- Restore from backup

**Cases to avoid:**

- Modify `uuid` or `fingerprint` (inconsistency)
- Add services without valid keys
- Modify `secrets.yaml.gpg` (encrypted, incomprehensible)

## Migration from Other Managers

If you have a config file from another SSHive version, SSHive migrates it automatically with full backward-compatibility.

See the [Changelog](/reference/changelog/) for migration details from v0.1 to v0.4.

---

See also:
- [Security](/reference/security/) — security philosophy
- [Changelog](/reference/changelog/) — version history
