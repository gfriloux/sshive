---
title: Deployment
description: Deploy your SSH keys to services (GitHub, GitLab, SSH).
---

**Deployment** is the process of pushing your public key to a service. SSHive supports three modes depending on the service type and your infrastructure.

## Deployment Modes

![SSHive deployment modal — guided mode with ssh-copy-id command](../../../assets/screenshots/deploy-modal.png)

*Deployment modal: adapts the interface based on the configured mode (automatic, guided, or external CM).*

### Automatic Mode

SSHive deploys the key completely without intervention.

**Supported services:**
- GitHub (via `/user/keys` API)
- GitLab.com (via `/user/keys` API)
- Self-hosted GitLab (via `/user/keys` API)
- Generic SSH (via `ssh-copy-id`)

**Flow:**

1. You click **Deploy**
2. SSHive sends the public key (GitHub/GitLab) or runs `ssh-copy-id` (SSH)
3. Post-deployment verification connects to the service to confirm acceptance
4. The screen displays **"✓ Connection verified"** or **"⚠ Verification not performed"**

**Prerequisites:**
- Valid API token (GitHub/GitLab)
- Working SSH access (Generic SSH)
- No existing key with the same fingerprint (GitHub/GitLab detect duplicates)

### Guided Mode

You copy-paste a command provided by SSHive.

**Useful when:**
- You're working on a bastion or behind NAT
- The service API is inaccessible
- You prefer manual control
- SSH-key authentication is already active (Automatic mode can't authenticate)

**Flow:**

1. You click **Deploy**
2. SSHive displays a formatted `ssh-copy-id` command with the public key
3. You copy it (**"Copy command"**)
4. You paste it in a terminal with server access
5. The key is added to `~/.ssh/authorized_keys`
6. No post-deployment verification is performed (semi-automatic mode)

**Example generated command:**

```bash
echo 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAb... user@host' \
  | ssh deploy@server.example.com "cat >> ~/.ssh/authorized_keys"
```

### Externally Managed Mode (External CM)

Your infrastructure (NixOS, Ansible, Terraform, etc.) manages `authorized_keys`.

**Useful when:**
- You deploy with NixOS, Ansible, Puppet, etc.
- Keys are managed declaratively in code
- You don't want SSHive to directly touch files
- You have centralized access management

**Flow:**

1. You create a service with type **Generic SSH** and mode **Externally Managed**
2. You click **Deploy**
3. SSHive displays the complete public key in a monospace block with instructions
4. You copy the public key
5. You add it to your Ansible/NixOS/Terraform/etc. configuration
6. You deploy your infrastructure normally

**Example for NixOS:**

```nix
users.users.deploy = {
  openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAb... sshive"
  ];
};
```

**Example for Ansible:**

```yaml
- name: Add deployment key
  authorized_key:
    user: deploy
    key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAb... sshive"
    state: present
```

## Select Mode at Creation

When creating a service, at **Step 2** (Deployment), you choose:

- **Automatic** — SSHive manages everything
- **Guided** — you provide the command
- **Externally Managed** — you integrate with your CM (Generic SSH only)

## Post-Deployment Verification

After automatic or guided deployment, SSHive can verify the key works:

1. For GitHub/GitLab: API call `GET /user` to confirm the token works
2. For Generic SSH: attempt SSH connection without running any command
3. For SK-Ed25519: mandatory verification (YubiKey must be present)

**Result:**
- **✓ Connection verified** — the key is accepted and works
- **⚠ Verification not performed** — guided deployment, no verification
- **✗ Verification error** — the key couldn't connect (see logs)

## Revoke a Deployed Key

To remove a key from a service:

1. Select the service
2. In the **SSH Keys** section, find the key
3. Click **Revoke**

SSHive:
- Calls GitHub/GitLab API to delete the key (if applicable)
- Removes the entry from `~/.ssh/authorized_keys` on the SSH server (via inverted `ssh-copy-id` or `ssh` + `sed`)
- For external CM, you must go back to your configuration and redeploy

## Deployment History

Each service lists all its past deployments:

```
DEPLOYMENTS
2026-05-11 14:35:22 — Automatic deployment via GitHub API (✓ verified)
2026-05-11 10:22:10 — Guided deployment (⚠ not verified)
2026-05-10 09:15:33 — Revocation (key deleted)
```

Consult this list to trace access and rotations.

## Troubleshooting

### Automatic deployment fails with API error

- **GitHub**: check token and `admin:public_key` scope
- **GitLab**: check token, `api` scope, and base URL
- **Generic SSH**: check SSH connectivity (hostname, username, port, access keys)

### Verification fails after deployment

- **Possible cause**: key deployed correctly but firewall/SSH config blocks verification
- **Solution**: manually verify the key works: `ssh -i ~/.ssh/id_sshive_github user@host`

### "Key already exists" (GitHub/GitLab)

SSHive detects duplicate keys and displays **ApiKeyAlreadyPresent**. This means the key was already deployed.

- **Solution**: revoke the old key first, then redeploy

### Guided Mode: command doesn't work

- Verify you're running it with valid SSH access (existing SSHive key passphrase, etc.)
- Check permissions: `~/.ssh/authorized_keys` must be `0600`

## Best Practices

**Automatic deployment by default** — it's the safest and fastest if the API is accessible.

**Guided mode for debugging** — if automatic fails, switch to guided to see exactly what command is executed.

**Post-deployment verification** — always verify the key works (SSHive does it, or verify manually).

**External CM for production** — if you have infrastructure as code (NixOS, Ansible), use Externally Managed mode for full traceability.

**Revoke before renewal** — when you generate a new key, revoke the old one first to avoid duplicates.

---

See also:
- [Services](/guide/services/) — create and edit services
- [SSH Keys](/guide/keys/) — generate and manage keys
- [Health](/guide/health/) — check deployment status
