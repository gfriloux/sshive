---
title: Configuration
description: Schéma YAML complet et structure de configuration SSHive.
---

SSHive stocke sa configuration dans un fichier YAML situé à :

```
~/.config/sshive/config.yaml
```

Ce fichier contient la liste de vos services, clefs SSH, et préférences globales.

## Emplacement et permissions

Le fichier `config.yaml` est créé automatiquement au premier lancement avec les permissions `0600` (lecture/écriture utilisateur seulement).

SSHive restitue une alerte si les permissions sont trop permissives (lisibles par d'autres utilisateurs).

## Structure globale

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
    # ... (voir section services ci-dessous)

ssh_keys:
  - uuid: "550e8400-e29b-41d4-a716-446655440000"
    # ... (voir section clefs ci-dessous)
```

## Section Security

```yaml
security:
  min_passphrase_len: 12              # Longueur minimale de passphrase (défaut 12)
  rotation_warning_days: 90           # Jours avant alerte de rotation (défaut 90)
```

**Valeurs par défaut :**
- `min_passphrase_len: 12` — conforme NIST SP 800-63B
- `rotation_warning_days: 90` — alerte après 90 jours sans rotation

## Section GPG

```yaml
gpg:
  key_id: "ABC123DEF456789..."       # ID court ou long de la clef GPG
  pinentry_backend: "gnome3"          # gnome3, qt, gtk2 (optionnel)
```

**Explications :**

- `key_id` — identificateur unique de votre clef GPG utilisée pour chiffrer les secrets
  - Peut être un ID court (16 caractères hex) ou long (40 caractères)
  - Affiché dans **Paramètres** → **GPG**

- `pinentry_backend` — agent de saisie de passphrase (optionnel, auto-détecté)
  - `gnome3` — GNOME (utilisé sur Ubuntu GNOME, Fedora GNOME…)
  - `qt` — KDE Plasma
  - `gtk2` — fallback universel
  - Omettez ce champ pour laisser SSHive auto-détecter à partir de `XDG_CURRENT_DESKTOP`

## Section Services

Chaque service représente une destination SSH (GitHub, GitLab, serveur).

### Structure générale d'un service

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

### Champs obligatoires

| Champ | Type | Exemple | Notes |
|-------|------|---------|-------|
| `name` | string | `"GitHub"` | Nom unique, affiché dans la liste |
| `service_type` | enum | `"github"` | `github` \| `gitlab` \| `gitlab-self-hosted` \| `ssh-generic` |
| `hostname` | string | `"github.com"` | Domaine ou IP du service |

### Champs spécifiques par type

**GitHub :**

```yaml
service_type: "github"
hostname: "github.com"           # Auto-rempli, normalement "github.com"
username: "git"                  # Auto-rempli
port: 22                         # Auto-rempli
token_ref: "secrets.github_token"  # Référence au token chiffré dans secrets.yaml.gpg
deploy_mode: "automatic"         # automatic | guided
```

**GitLab.com :**

```yaml
service_type: "gitlab"
hostname: "gitlab.com"           # Auto-rempli
username: "git"                  # Auto-rempli
port: 22
token_ref: "secrets.gitlab_token"
deploy_mode: "automatic"
```

**GitLab auto-hébergé :**

```yaml
service_type: "gitlab-self-hosted"
hostname: "https://gitlab.example.com"  # URL complète
username: "git"
port: 22
token_ref: "secrets.gitlab_custom_token"
deploy_mode: "automatic" | "guided"
```

**SSH générique :**

```yaml
service_type: "ssh-generic"
hostname: "deploy.example.com"
username: "deploy"
port: 2222
deploy_mode: "automatic" | "guided" | "external-cm"
```

### Champs optionnels (tous les services)

| Champ | Type | Défaut | Signification |
|-------|------|--------|---------------|
| `username` | string | `"git"` (GitHub/GitLab) | Utilisateur SSH |
| `port` | integer | `22` | Port SSH |
| `active_key` | string (UUID) | `null` | UUID de la clef SSH attachée |
| `pending_key` | string (UUID) | `null` | UUID de la clef en cours de déploiement |
| `token_ref` | string | `null` | Référence au token API dans secrets.yaml.gpg |
| `deploy_mode` | enum | `"automatic"` | `automatic` \| `guided` \| `external-cm` |
| `created_at` | ISO 8601 | auto | Timestamp de création du service |
| `last_rotation` | ISO 8601 | auto | Timestamp de la dernière rotation de clef |
| `deployments` | array | `[]` | Historique des déploiements (voir ci-dessous) |
| `comment` | string | `""` | Note interne sur le service |

### Champ Deployments (historique)

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

## Section SSH Keys

Chaque clef SSH est stockée localement avec métadonnées.

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

### Champs de clefs

| Champ | Type | Exemple | Notes |
|-------|------|---------|-------|
| `uuid` | string (UUID v4) | `"550e8400-e29b-41d4-a716-446655440000"` | Identificateur unique stable (basé sur fingerprint) |
| `fingerprint` | string | `"SHA256:abc123def456789..."` | Fingerprint SHA256 de la clef publique |
| `public_path` | string | `"/home/user/.ssh/id_sshive_github.pub"` | Chemin du fichier `.pub` |
| `private_path` | string | `"/home/user/.ssh/id_sshive_github"` | Chemin du fichier clef privée |
| `key_type` | enum | `"ed25519"` | `ed25519` \| `sk-ed25519` |
| `key_size` | integer | `256` | Bits de clef (toujours 256 pour ed25519) |
| `created_at` | ISO 8601 | `"2026-05-11T10:34:22Z"` | Timestamp de génération |
| `protected` | boolean | `true` | Clef chiffrée par passphrase |
| `backup_prompted` | boolean | `false` | Pour SK-Ed25519 : passphrase `.pub` sauvegardée confirmée |
| `comment` | string | `"GitHub deployment key"` | Comment optionnel |

## Secrets (secrets.yaml.gpg)

Les tokens API et autres secrets sont chiffrés dans un fichier séparé :

```
~/.config/sshive/secrets.yaml.gpg
```

Ce fichier ne doit **jamais** être édité manuellement. SSHive le gère complètement.

Structure déchiffrée (ne pas toucher) :

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

Chaque action est enregistrée dans :

```
~/.config/sshive/audit.log
```

Format :

```
2026-05-11T14:35:22Z [sshive] Generated SSH key ed25519 for service GitHub (fingerprint: abc123...)
2026-05-11T14:35:45Z [sshive] Deployed key to GitHub via API (verified: true)
2026-05-11T16:20:10Z [sshive] Rotated key for service Production
2026-05-12T09:10:33Z [sshive] Revoked key from GitLab
```

Les logs ne contiennent **jamais** les tokens API ou passphrases.

## Exemple complet

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

## Gestion manuelle (avancé)

SSHive restitue une alerte si `config.yaml` est modifié à l'extérieur de l'application. Les modifications manuelles peuvent casser la cohérence.

**Si vous modifiez manuellement :**

1. Faites une sauvegarde : `cp ~/.config/sshive/config.yaml ~/.config/sshive/config.yaml.bak`
2. Éditez avec votre éditeur
3. Redémarrez SSHive et vérifiez les alertes
4. Consultez la page **Santé** pour vérifier la cohérence

**Cas d'usage valides :**

- Ajouter un commentaire
- Corriger un typo dans `hostname` ou `username`
- Restaurer depuis une sauvegarde

**Cas à éviter :**

- Modifier `uuid` ou `fingerprint` (incohérence)
- Ajouter des services sans clef valide
- Modifier `secrets.yaml.gpg` (chiffré, incompréhensible)

## Migration depuis d'autres gestionnaires

Si vous avez un fichier config d'une autre version SSHive, SSHive le migre automatiquement avec backward-compatibility totale.

Voir le [Changelog](/reference/changelog/) pour les détails de migration de v0.1 à v0.4.

---

Voir aussi :
- [Sécurité](/reference/security/) — philosophie de sécurité
- [Changelog](/reference/changelog/) — historique des versions
