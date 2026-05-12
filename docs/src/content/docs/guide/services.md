---
title: Services
description: Create and manage SSH services in SSHive.
---

A **service** in SSHive represents an SSH destination (GitHub, GitLab, a server) and one of your SSH keys deployed to it.

## Service Types

SSHive supports 4 service types, each with its own characteristics:

### GitHub

Allows key management via GitHub API.

**Required configuration:**
- **Hostname**: `github.com` (auto-filled)
- **API Token**: `admin:public_key` scope
- **Deployment mode**: Automatic (via API) or Guided

**Benefits:**
- Automatic deployment via API
- Post-deployment verification (SSH)
- Simplified revocation

### GitLab.com

Allows key management via public GitLab API.

**Required configuration:**
- **Hostname**: `gitlab.com` (auto-filled)
- **API Token**: `api` scope
- **Deployment mode**: Automatic (via API) or Guided

**Benefits:**
- Automatic deployment via API
- Post-deployment verification (SSH)
- Simplified revocation

### Self-hosted GitLab

Allows key management for a private GitLab instance.

**Required configuration:**
- **Hostname**: Base URL of your GitLab instance (ex: `https://gitlab.example.com`)
- **API Token**: `api` scope
- **Deployment mode**: Automatic (via API) or Guided

**Benefits:**
- Same API as GitLab.com, but on your server
- Automatic deployment if API is accessible
- Otherwise, switch to Guided mode

### Generic SSH

For any standard SSH server (Linux, BSD, your machines…).

**Required configuration:**
- **Hostname**: IP address or domain name
- **Username**: SSH user (ex: `deploy`, `ec2-user`)
- **Port**: SSH port (default 22)
- **Deployment mode**: Automatic (`ssh-copy-id`), Guided (copy key), or Externally Managed

**Benefits:**
- Flexible for any SSH server
- Manual or automatic deployment
- Support for external CM (NixOS, Ansible)

![SSHive main view — service list and detail panel](../../../assets/screenshots/detail-panel.png)

*Detail panel of a service: health banner, amber fingerprint, deployment history and available actions.*

## Create a Service

1. Click **"Add a service"** in the empty list, or the **+** button in the sidebar
2. **Step 1: Service type**
   - Select the type: GitHub, GitLab, Self-hosted GitLab or Generic SSH
3. **Step 2: Parameters**
   - Enter a **unique name** for the service
   - Generic SSH: fill in **hostname**, **SSH user**, **port** (default 22)
   - Self-hosted GitLab: fill in the **instance URL**
   - GitHub/GitLab: enter the **API token** (optional — can be configured later)
4. **Step 3: Deployment mode**
   - Select Automatic, Guided, or External CM
   - Review the summary then click **Create service**

The service is created **without an attached SSH key**. The next step is to associate a key with it.

## Attach an SSH Key to a Service

Two paths are available from the **service detail panel** (click on the service in the list):

### Generate a new key

Click **"Generate a key"** on the **Keys** page while selecting the target service. This creates an ed25519 pair (or sk-ed25519 for YubiKey), encrypted with a passphrase, and immediately attaches it to the service.

### Attach an existing key

If you already have a key managed by SSHive (visible on the **Keys** page) not yet attached to a service:

1. Select the service from the list — the detail panel opens
2. In the **SSH Key** section, click **"Attach an existing key"**
3. A list shows available keys with their fingerprint and creation date
4. Click on the desired key — it's immediately linked to the service

:::note
Only **managed** keys (with private key in `~/.ssh/`) can be attached from this selector. Discovered keys without a private key are not offered.
:::

## Edit a Service

1. Select the service from the list
2. Click **Edit** at the top of the detail
3. Modify the desired fields (hostname, username, API token, etc.)
4. Changes are automatically saved when you click **Validate**

**Important**: editing preserves the attached key, deployment history, and rotations. Only the form fields are modified.

## Delete a Service

1. Select the service
2. Click **Delete** at the top of the detail
3. Confirm the deletion

**Note**: the SSH key is not deleted, only the service. The key remains available for other services.

## Status and Health

Each service displays a **health badge** (colored dot) indicating its state:

- **✓ ok (green)** — key deployed, protected, API token valid (if applicable)
- **ⓘ info (blue)** — key present but not protected by passphrase
- **⚠ warning (orange)** — key missing, token absent, rotation recommended
- **✗ critical (red)** — key missing + rotation overdue, or loss of API token

See the [Health and Diagnostics](/guide/health/) page for the complete list of reasons.

## Deploy a Key

Once the service is created, you must **deploy** the key:

1. Select the service
2. Click **Deploy** in the detail panel
3. SSHive executes the deployment according to the chosen mode:
   - **Automatic** — GitHub/GitLab API or `ssh-copy-id`
   - **Guided** — shows command, you paste it in terminal
   - **Externally Managed** — displays key to put in your CM

4. Post-deployment verification confirms the key is accepted
5. You'll see **"✓ Connection verified"** if all is OK

## View Deployment History

In the detail panel, the **DEPLOYMENTS** section lists all past deployments and revocations with timestamps.

## Revoke a Key

1. Select the service
2. Find the key in the **SSH Keys** section of the detail
3. Click **Revoke**

For GitHub/GitLab, this calls the API to delete the key. For Generic SSH, this removes it from `~/.ssh/authorized_keys` on the server.

## Deploy an Existing Key

If you have an SSHive key not deployed on a service:

1. Select the service
2. Find the key in the **Available SSH Keys** section
3. Click **Select** to attach it
4. Click **Deploy** to start deployment

## Tips and Best Practices

**One key per service** — always prefer generating a new key for each service rather than reusing the same key everywhere.

**Explicit names** — use clear service names like `GitHub Production` rather than `srv1`.

**Local documentation** — note the purpose of each service (ex: "CI/CD deployments" vs "Personal access").

**Regular rotation** — set a rotation alert delay (default 90 days) and check the Health page regularly.

---

See also:
- [SSH Keys](/guide/keys/) — key management
- [Deployment](/guide/deployment/) — detailed deployment modes
- [Health](/guide/health/) — diagnostics and alerts
