---
title: SSH Keys
description: Generate, manage, and renew your SSH keys.
---

In SSHive, **SSH keys** are the cryptographic artifacts that authorize access to your services. Each key is stored locally, protected, and can be attached to multiple services.

## Key Types

### Ed25519

Curve25519 signature key, recommended for most cases.

**Benefits:**
- Modern and robust cryptography
- Short (64-character fingerprint)
- Fast to verify
- Widely supported by GitHub, GitLab, OpenSSH

**Default location**: `~/.ssh/id_sshive_github` (or similar)

### SK-Ed25519 (YubiKey / FIDO2)

Hardware key using a YubiKey or other FIDO2 token.

**Benefits:**
- Private key never stored on disk (hardware only)
- Requires physical button press for each signature
- Better protection against key theft/copy
- Supported by GitHub and OpenSSH 8.2+

**Requirements**: YubiKey 5+ compatible with FIDO2, or other token (Titan, etc.)

**Default location**: `~/.ssh/id_sshive_github_sk`

**Important**: the `.pub` file is stored on disk (like ed25519), but the private key lives only in the hardware. After generation, **back up the `.pub` file and note the path to the private key file** in case the hardware is lost.

## Generate a Key

From the **Keys** page:

1. Click **"Generate a key"**
2. Select the **target service** from the dropdown
3. Choose the type: **ed25519** (recommended) or **sk-ed25519** (YubiKey)
4. Enter a **strong passphrase** (≥ 12 characters)
5. Click **Generate**

SSHive runs `ssh-keygen`, stores the key in `~/.ssh/`, records the fingerprint in the config, and attaches the key to the selected service.

## Attach an Existing Key to a Service

If you already have a key in SSHive and want to associate it with a service without regenerating:

1. Select the service from the list (on the **Services** page)
2. In the detail panel, **SSH Key** section, click **"Attach an existing key"**
3. Select the key from the selector — only managed keys (with private key) are offered
4. The binding is saved immediately

:::tip
A service created without a key displays a **Critical** warning in the Health view. You can resolve this by generating a new key or attaching an existing one.
:::

## Protect an Existing Key

If you have existing SSH keys in `~/.ssh/` without a passphrase, SSHive detects them and alerts.

1. Open the key detail
2. Click **Add a passphrase**
3. Enter a passphrase (≥ 12 characters)
4. Confirm

The key is re-encrypted locally without touching its public key.

## Renew a Key (Rotation)

To replace a worn or compromised key:

1. Select the service using the key
2. Click **Renew SSH key** in the detail
3. SSHive displays a warning if it's an SK (YubiKey must stay connected)
4. Confirm
5. A new key is generated
6. The old key is listed in **Previous Keys** with a **Revoke** button

**Important for SK-Ed25519**: throughout the rotation operation, the YubiKey must stay connected. Post-rotation verification is mandatory.

## View All Keys

1. Click **Health** in the sidebar
2. You'll see a list of all your SSH keys with:
   - Fingerprint (monospace)
   - Type (ed25519 / sk-ed25519)
   - YubiKey badge if applicable
   - Age (days since generation)
   - Services using it
   - Protection status (✓ protected / ⚠ unprotected)

## Copy a Public Key

To share your public key:

1. Select the key from the SSH keys list
2. Click **Copy public key**
3. The key is copied to the clipboard
4. The button shows **"✓ Copied"** for 2 seconds

You can then paste it in a browser, form, etc.

## Revoke a Key

**Locally** (deletion from `~/.ssh/`):

1. Go to the SSH Keys view
2. Select the key
3. Click **Delete locally**
4. The `.pub` file and private key are deleted from `~/.ssh/`

**Remotely** (deletion from GitHub/GitLab):

1. Select the service using the key
2. Click **Revoke** on the key
3. SSHive calls the API to delete the key from the service
4. The entry in `~/.ssh/authorized_keys` on SSH servers is deleted

## Passphrase Management

### Passphrase Entry

Whenever SSHive needs a passphrase (generation, rotation, signing), it launches a **graphical pinentry**:

- **GNOME** → `pinentry-gnome3`
- **KDE** → `pinentry-qt`
- **Other** → `pinentry-gtk-2`

You can force a backend with `export SSHIVE_PINENTRY=pinentry-gtk-2`.

### Passphrase Encryption

Passphrases are **never** stored in plain text. SSHive encrypts them immediately with GPG and forgets them.

Whenever a passphrase is needed, SSHive requests it via pinentry.

### Minimum Length

By default, passphrases must be ≥ 12 characters. You can adjust this threshold in **Settings** → **Security** → **Minimum password length**.

## Key Audit

Every generation, rotation, and revocation of a key is recorded in the audit log:

```
~/.config/sshive/audit.log
```

Example:

```
2026-05-11T10:34:22Z [sshive] Generated SSH key ed25519 for service GitHub (fingerprint: abc123...)
2026-05-11T10:35:10Z [sshive] Deployed key to GitHub via API
2026-05-11T14:22:45Z [sshive] Rotated key for service Production (new fingerprint: def456...)
```

## Best Practices

**Strong passphrases** — use at least 12 characters (compliant with NIST SP 800-63B).

**No reuse** — each service should have its own key, not a shared key.

**Regular rotation** — SSHive alerts you when a key approaches 90 days. Renew it.

**YubiKey backup** — after generating an SK-Ed25519, back up the `.pub` file and note the path to the private key file in case the hardware is lost.

**Audit** — regularly check `audit.log` to verify actions on keys.

---

See also:
- [Services](/guide/services/) — attach keys to services
- [Deployment](/guide/deployment/) — deploy keys
- [Health](/guide/health/) — security diagnostics
