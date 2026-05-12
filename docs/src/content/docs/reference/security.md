---
title: Security
description: Security philosophy, threats, and protection measures.
---

SSHive is designed with **security as first priority**. This page explains the philosophy, identified threats, and implemented protection measures.

## Security Principles

### Least Privilege

Each SSH key is dedicated to a single service. No key is shared between multiple services or users.

**Benefit:** if a key is compromised, only that service is exposed.

### Encrypted Secrets

API tokens and passphrases are **never stored in plain text**.

- **API tokens** → encrypted with GPG in `secrets.yaml.gpg`
- **Passphrases** → decrypted only when needed, never stored

### Hardened Process

At startup, SSHive applies Linux process hardening measures:

- `PR_SET_DUMPABLE = 0` — disables core dumps (cannot be exploited to extract secrets)
- `PR_SET_PTRACER = -1` — blocks ptrace attachments (debuggers, strace)

### Modern Cryptography

- **Algorithms**: Ed25519 (signatures), Curve25519 (encryption), AES-256-CTR (key re-encryption)
- **Encryption**: bcrypt-pbkdf for passphrase key derivation
- **RNG**: `/dev/urandom` (system), no custom RNG

## Threats and Mitigations

### T1: Passphrase Theft During Entry

**Threat**: an attacker captures the passphrase by reading `/proc/<pid>/cmdline`.

**Mitigation v0.2.0**: passphrases entered via `pinentry` (isolated GUI), not directly passed to `ssh-keygen`.

**Mitigation v0.3.0**: passphrases decrypted only inside SSHive, never passed as process argument. Pure Rust re-encryption via `ssh-key` crate + bcrypt-pbkdf.

**Residual**: if an attacker has root or process debugging access, passphrase can be read from memory. **Mitigation**: `mlock` on passphrase buffers to pin in physical RAM.

### T2: Unprotected Private Keys

**Threat**: an SSH key without passphrase can be used immediately if someone accesses `~/.ssh/`.

**Mitigation**:
- Generation: **passphrase mandatory** (≥ 12 characters, NIST compliant policy)
- Detection: SSHive scans `~/.ssh/` at startup and alerts if a key has no passphrase
- Retroactive protection: **"Add a passphrase"** for existing unprotected keys

**Residual**: a user can always force generation without passphrase via `ssh-keygen` directly. **Mitigation**: local audit of all SSHive actions.

### T3: Key Compromise via Disk Access

**Threat**: an attacker copies the private key file from `~/.ssh/` (even without passphrase, they can force the passphrase offline).

**Mitigation**:
- File permissions: `0600` (user only, readable by owner)
- Optional encryption: SSH passphrase (`ssh-keygen -N`)
- Hardware keys: SK-Ed25519 (private key never on disk, only on YubiKey)

**Residual**: if attacker has disk/VM access, they can extract passphrased keys via GPU cracking offline. **Mitigation**: strong passphrase (16+ characters) + regular rotation (90 days) = limited time window.

### T4: Exposed API Tokens

**Threat**: a GitHub/GitLab token allows adding/removing SSH keys without limit. If exposed, an attacker can access all services.

**Mitigation**:
- **GPG encryption** — tokens stored in `secrets.yaml.gpg`, encrypted with your GPG key
- **Never in logs** — tokens never appear in `audit.log`, error messages, or config
- **On-demand decryption** — tokens exist in memory decrypted only during API call, then forgotten
- **Minimal scopes** — GitHub: `admin:public_key` only (no `repo`, `delete_repo`, etc.); GitLab: `api` (minimal for `/user/keys`)

**Residual**: if GPG is compromised (attacker has your GPG passphrase), all tokens are exposed. **Mitigation**: strong GPG passphrase + hardened process blocks debugger attachments.

### T5: Compromise of Intermediate Service (GitHub/GitLab)

**Threat**: an attacker compromises GitHub or GitLab and adds public keys to your account.

**Mitigation**:
- **Post-deployment verification** — after deploying a key, SSHive calls the API to confirm only its key is listed
- **Local audit** — `audit.log` records all deployments with fingerprint and timestamp
- **Distinct keys per service** — if GitHub is compromised, only your GitHub access is exposed (not your other services)

**Residual**: if GitHub is compromised long before detection, attacker can create SSH keys. **Mitigation**: regular audit (weekly verification of public key list on GitHub).

### T6: Loss of YubiKey with SK-Ed25519

**Threat**: the private key of an SK-Ed25519 is stored physically on the YubiKey. If the YubiKey is lost/stolen, the key is inaccessible.

**Mitigation**:
- **`.pub` file backup** — after generation, SSHive alerts to back up the public key file (necessary for revocation)
- **Pre-rotation warning** — before renewing an SK, SSHive warns: "YubiKey must stay connected"
- **Mandatory verification** — any SK rotation must pass post-deployment verification (extra security)

**Residual**: if YubiKey is lost and you don't have the backup, you can't revoke the key remotely. **Mitigation**: back up the `.pub` immediately after generation.

## Implemented Protections

### Secrets Encryption

**File**: `~/.config/sshive/secrets.yaml.gpg`

- Encrypted with **GPG** + your private GPG key
- Requires your **GPG passphrase** to decrypt
- Contents: GitHub/GitLab API tokens

**Decrypted format (never touch):**

```yaml
tokens:
  github_token:
    api_token: "ghp_xxxxx..."
  gitlab_token:
    api_token: "glpat-xxxxx..."
```

### File Permissions

| File | Permission | Owner | Justification |
|------|-----------|-------|---------------|
| `config.yaml` | `0600` | user | Readable by owner only |
| `secrets.yaml.gpg` | `0600` | user | Encrypted, but readable by owner only |
| `audit.log` | `0600` | user | Local audit, sensitive |
| `~/.ssh/` | `0700` | user | Standard SSH |
| `~/.ssh/id_*` | `0600` | user | Standard private key |
| `~/.ssh/id_*.pub` | `0644` | user | Public key, readable by all |

SSHive checks permissions at startup and alerts if too permissive.

### GPG Passphrase

The security of all secrets depends on your **GPG passphrase**.

- Choose a strong passphrase (16+ characters)
- Don't reuse it on other services
- Store it safely (password manager, encrypted paper, etc.)

### Local Audit

All actions are recorded in `~/.config/sshive/audit.log` (append-only, `0600`):

```
2026-05-11T14:35:22Z [sshive] Generated SSH key ed25519 for service GitHub
2026-05-11T14:35:45Z [sshive] Deployed key to GitHub via API (verified: true)
2026-05-11T16:20:10Z [sshive] Rotated key for service Production
```

Check this log to:
- Detect unauthorized access
- Trace rotations and revocations
- Audit compliance

### Cryptographic Algorithms

| Usage | Algorithm | Details |
|-------|-----------|---------|
| Key generation | Ed25519 | Curve25519, 256 bits, quantum-resistant |
| Passphrase derivation | bcrypt-pbkdf | 16 rounds, SHA512, salted |
| Key re-encryption | AES-256-CTR | 256 bits, CTR mode |
| GPG encryption | GnuPG (RSA/DSA/ECDSA) | Depends on your key (configurable) |
| HTTP signatures | TLS 1.2+ / RUSTLS | No OpenSSL, WebPKI roots |

## Operational Best Practices

### Strong SSH Passphrases

- **Minimum**: 12 characters (configurable, default complies with NIST)
- **Recommended**: 16+ characters, mix of alphanumeric + symbols
- **Example**: `"MyGitHub_Prod@2026!Key"`

### Regular Rotation

- **Default**: alert after 90 days without rotation
- **Recommended**: renew every 60-90 days for production
- **Procedure**: SSHive handles rotation atomically (generate, deploy, archive old)

### GPG Passphrase

- **Length**: 16+ characters (don't reuse other passwords)
- **Storage**: password manager or encrypted paper
- **Test**: validate your passphrase works regularly

### Audit

- **Weekly**: check Health page for alerts
- **Monthly**: review `audit.log` for unexpected actions
- **Yearly**: verify public key list on GitHub/GitLab vs SSHive config

### Backup

- **config.yaml**: back up regularly (version in private git if possible)
- **secrets.yaml.gpg**: back up the encrypted file (unreadable without GPG)
- **SK-Ed25519 `.pub`**: back up after generation (needed for revocation)

## Report Vulnerabilities

If you discover a security flaw in SSHive:

1. **Don't publish** the exploit in a public issue
2. **Contact** the author via email (see [GitHub](https://github.com/gfriloux/sshive))
3. **Include** details: version, impact, PoC if relevant
4. **Wait** for a fix before public disclosure

All security vulnerabilities are treated as priority.

## Known and Accepted Limitations

1. **Attacker with root** — if someone has root access, they can extract secrets. **Mitigation**: audit root access on your system.

2. **Weak passphrases** — a weak SSH passphrase can be cracked in hours. **Mitigation**: enforce minimum length (default 12).

3. **GPG passphrase in memory** — after decrypting `secrets.yaml.gpg`, the passphrase isn't immediately forgotten (GPG caches for a bit). **Mitigation**: strong GPG passphrase.

4. **Compromise of GPG** — if your GPG key is compromised or stolen, all secrets are exposed. **Mitigation**: protect your GPG key with a strong passphrase and YubiKey protection.

---

See also:
- [Configuration](/reference/configuration/) — file format and location
- [SSH Keys](/guide/keys/) — generate and protect keys
- [API Tokens](/guide/tokens/) — configure tokens safely
