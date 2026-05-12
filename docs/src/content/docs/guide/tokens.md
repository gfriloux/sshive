---
title: API Tokens
description: Configure and manage GitHub and GitLab API tokens.
---

SSHive uses **API tokens** to access GitHub and GitLab in order to deploy and revoke your SSH keys automatically, without having to copy them manually.

## Why Tokens Are Necessary

Tokens allow SSHive to:

1. **Deploy keys** via API (instead of `ssh-copy-id`)
2. **Revoke keys** remotely (in case of compromise)
3. **Verify success** of deployment immediately

Without a token, you're limited to **Guided mode** (copy-paste commands).

## Token Security

SSHive treats tokens with care:

- **GPG encryption** — all tokens are immediately encrypted and stored in `~/.config/sshive/secrets.yaml.gpg`
- **Never in memory unnecessarily** — tokens are decrypted only when used
- **Never in logs** — tokens never appear in `audit.log` or error messages
- **Easy revocation** — delete the token from the service in SSHive to "forget" it locally

See the [Security Policy](/reference/security/) for more details.

## GitHub Tokens

### Get a GitHub Token

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens)
2. Click **Generate new token (classic)**
3. Give it a name: `SSHive`
4. **Scopes to check (IMPORTANT)**:
   - ✓ `admin:public_key` — **ONLY scope to check**
   
   This scope implicitly includes `read:public_key`, which is sufficient for deploying and verifying keys.

5. Click **Generate token**
6. **Copy the token immediately** — GitHub will only show it once

### Configure Token in SSHive

**When creating a GitHub service:**

1. At **Step 2**, paste the token in the **API Token** field
2. Click **Next**

**For an existing GitHub service:**

1. Select the service from the list
2. Click **Edit**
3. At step 2, click **Configure token**
4. Paste the new token (it replaces the old one)
5. Click **Validate**

### Scope Details

The `admin:public_key` scope includes:

- `read:public_key` — read the user's public keys
- `write:public_key` — add, delete the user's public keys

This is exactly what SSHive needs. **Don't grant additional scopes.**

### Test Token

SSHive automatically tests the token before using it:

1. Validates token format (starts with `ghp_` or `github_pat_`)
2. API call `GET /user` to verify authentication
3. Displays **DeployBlocker** alert if test fails

## GitLab.com Tokens

### Get a GitLab Token

**For GitLab.com:**

1. Go to [gitlab.com/profile/personal_access_tokens](https://gitlab.com/profile/personal_access_tokens)
2. Click **Add new token**
3. Name: `SSHive`
4. **Scopes to check (IMPORTANT)**:
   - ✓ `api` — access to the API (includes `/user/keys`)

5. Click **Create personal access token**
6. **Copy the token immediately** — GitLab will only show it once

### Configure Token in SSHive

Same procedure as GitHub:

1. Create or edit a GitLab.com service
2. At step 2, paste the token in the **API Token** field
3. Validate

## Self-hosted GitLab Tokens

### Get a Token (Private Instance)

For a private GitLab instance (ex: `https://gitlab.example.com`):

1. Go to `https://gitlab.example.com/profile/personal_access_tokens`
2. Click **Add new token**
3. Name: `SSHive`
4. **Scope**: ✓ `api`
5. Click **Create personal access token**
6. Copy the token

### Configure in SSHive

1. Create a self-hosted GitLab service
2. Enter the **Base URL**: `https://gitlab.example.com`
3. At step 2, paste the token
4. Validate

SSHive will use the URL to make API calls automatically.

## Token Management

### List Configured Tokens

Go to **Settings** → **Data** to see the list of services with tokens configured.

Tokens themselves are not displayed (for security), only their presence is indicated.

### Delete a Token

1. Select the service from the list
2. Click **Edit**
3. At step 2, click **Delete token**
4. Validate

The local token is forgotten. The token on GitHub/GitLab remains valid until you manually revoke it on the site.

### Revoke Token on Site

If you've compromised a token or want to stop it:

**GitHub:**

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens)
2. Find the `SSHive` token
3. Click **Delete**

**GitLab.com:**

1. Go to [gitlab.com/profile/personal_access_tokens](https://gitlab.com/profile/personal_access_tokens)
2. Find the token
3. Click the trash icon

Revocation is immediate. Future SSHive attempts will fail.

## Token Rotation

It's good practice to **change tokens regularly** (every 6-12 months).

**Procedure:**

1. Generate a new token on GitHub/GitLab
2. Go to SSHive and update the service token (Edit → Configure token)
3. Verify that deployment works
4. Go to GitHub/GitLab and delete the old token

## Troubleshooting

### "Invalid token" or "Authentication failed"

- Verify you copied the **entire token** (no missing characters)
- Verify the token **hasn't expired**
- Verify the **scopes**: GitHub must have `admin:public_key`, GitLab must have `api`

### GitHub/GitLab service shows "warning: NoApiToken"

This means no token is configured. Go to **Edit** the service and add the token.

### Token verification fails with "DeployBlocker"

- Check **Internet connectivity** (SSHive calls the API)
- Verify the token **hasn't expired** or been revoked
- Verify your GitHub/GitLab account has no issues (2FA blocking, etc.)

## Best Practices

**One token per service, or shared?** — For simplicity, you can use the same token for all your GitHub services. For isolation, generate one token per service (not necessary for security, just traceability).

**Strong passphrases on tokens** — tokens are stored encrypted with GPG, so their security depends on your GPG passphrase.

**Revoke immediately if leaked** — if you accidentally expose a token, go to GitHub/GitLab and delete it.

**Store nowhere else** — never add a token to `config.yaml` or an unencrypted file. SSHive does this for you (secrets.yaml.gpg).

**Regular rotation** — every 6-12 months, generate a new token and replace the old one in SSHive.

---

See also:
- [Services](/guide/services/) — create GitHub/GitLab services
- [Security](/reference/security/) — security policy and philosophy
- [Configuration](/reference/configuration/) — config file structure
