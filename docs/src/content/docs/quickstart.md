---
title: Quick Start
description: Add your first GitHub service in 5 minutes.
---

This guide shows you how to add your first service (GitHub) and deploy an SSH key in 5 minutes.

## Step 1: Launch SSHive

```bash
sshive
```

You'll see the interface with three columns: **Navigation** (left), **Services** (center), **Detail** (right).

## Step 2: Create a new service

1. Click **"Add my first service"** in the services list
2. Fill in the service information:
   - **Name**: `GitHub` (or any unique name)
   - **Type**: `GitHub`
   - **Hostname**: `github.com` (auto-filled)

3. Click **Next**

## Step 3: Configure deployment

1. Choose **Automatic mode** — SSHive will use GitHub API to deploy
2. Paste your **GitHub token** with the `admin:public_key` scope

**How to get a GitHub token:**

- Go to [github.com/settings/tokens](https://github.com/settings/tokens)
- Click **Generate new token (classic)**
- Scopes: check only **`admin:public_key`**
- Generate and copy the token into SSHive

3. Click **Next**

## Step 4: Generate an SSH key

1. Choose **Generate a new key**
2. Type: **ed25519** (default, recommended)
3. Passphrase: enter a passphrase ≥ 12 characters
4. Click **Generate**

SSHive encrypts the passphrase with GPG and generates the key. You'll see the fingerprint displayed.

## Step 5: Deploy the key

1. Click **Deploy**
2. SSHive sends the key to GitHub via API
3. Post-deployment verification is performed
4. A "✓ Connection verified" message confirms success

Congratulations! Your first key is now deployed on GitHub.

## Step 6: Check health

1. Click **Health** in the sidebar
2. You'll see your GitHub service listed with **ok** status (green dot)

## Next Steps

- **Add an SSH server** — see [Services](/guide/services/)
- **Understand deployment modes** — see [Deployment](/guide/deployment/)
- **Renew a key** — see [SSH Keys](/guide/keys/)
- **Manage API tokens** — see [API Tokens](/guide/tokens/)

## Tips

### Test with a generic SSH service (no public account required)

If you don't have a GitHub account yet, you can test with a generic SSH service:

1. Create a new service with type **Generic SSH**
2. Fill in the connection information:
   - **Hostname**: `127.0.0.1` or your local machine
   - **Username**: `root`
   - **Port**: `22`
3. Generate and deploy a key as above

### Copy an existing key

If you already have keys in `~/.ssh/`, SSHive discovers them automatically. You can attach them to a service without regenerating:

1. Open the service detail
2. Find the key in the **Available SSH Keys** list
3. Click **Select** to attach it

### YubiKey (hardware security key)

To generate an sk-ed25519 key (requires a YubiKey or other FIDO2 USB device):

1. In the key generation step, choose **SK-Ed25519** instead of ed25519
2. Follow the instructions to press the YubiKey button when prompted
3. The private key is saved in `~/.ssh/` — save the file somewhere safe

---

You're ready. Check the [Detailed Guides](/guide/services/) for more information on each feature.
