---
title: Sécurité
description: Philosophie de sécurité, menaces, et mesures de protection.
---

SSHive est conçu avec **sécurité en première priorité**. Cette page explique la philosophie, les menaces identifiées et les mesures de protection implémentées.

## Principes de sécurité

### Moindre privilège

Chaque clef SSH est dédiée à un seul service. Aucune clef n'est partagée entre plusieurs services ou utilisateurs.

**Bénéfice :** si une clef est compromise, seul ce service est exposé.

### Secrets chiffrés

Les tokens API et passphrases ne sont **jamais stockés en clair**.

- **Tokens API** → chiffrés avec GPG dans `secrets.yaml.gpg`
- **Passphrases** → déchiffrées uniquement quand nécessaires, jamais stockées

### Processus durci

Au démarrage, SSHive applique des mesures de durcissement du processus Linux :

- `PR_SET_DUMPABLE = 0` — désactive les core dumps (ne peuvent pas être exploités pour extraire les secrets)
- `PR_SET_PTRACER = -1` — bloque les attachements ptrace (debuggers, strace)

### Cryptographie moderne

- **Algoritmes** : Ed25519 (signatures), Curve25519 (chiffrement), AES-256-CTR (re-encryption de clefs)
- **Chiffrement** : bcrypt-pbkdf pour dérivation de clef de passphrase
- **RNG** : `/dev/urandom` (système), pas de RNG custom

## Menaces et mitigations

### T1 : Vol de passphrase pendant la saisie

**Menace** : un attaquant capture la passphrase en lisant `/proc/<pid>/cmdline`.

**Atténuation v0.2.0** : passphrases saisies via `pinentry` (interface graphique isolée), pas directement passées à `ssh-keygen`.

**Atténuation v0.3.0** : passphrases déchiffrées uniquement à l'intérieur de SSHive, jamais passées en argument à un processus. Re-encryption pure Rust via la crate `ssh-key` + bcrypt-pbkdf.

**Résidu** : si un attaquant a accès root ou débogage du processus, la passphrase peut être lue en mémoire. **Mitigation** : `mlock` sur les buffers de passphrase pour épingler en RAM physique.

### T2 : Clefs privées non protégées

**Menace** : une clef SSH sans passphrase peut être utilisée immédiatement si quelqu'un accède à `~/.ssh/`.

**Atténuation** :
- Génération : **passphrase obligatoire** (≥ 12 caractères, policy NIST conforme)
- Détection : SSHive scanne `~/.ssh/` au démarrage et alerte si une clef n'a pas de passphrase
- Protection rétrospective : **"Ajouter une passphrase"** pour les clefs existantes sans protection

**Résidu** : un utilisateur peut toujours forcer la génération sans passphrase via `ssh-keygen` directement. **Mitigation** : audit local de toutes les actions SSHive.

### T3 : Compromise d'une clef via accès disque

**Menace** : un attaquant copie le fichier clef privée de `~/.ssh/` (même sans passphrase, il peut forcer la passphrase hors ligne).

**Atténuation** :
- Permissions fichiers : `0600` (utilisateur seulement, lisible par le propriétaire)
- Chiffrement optionnel : passphrase SSH (`ssh-keygen -N`)
- Hardware keys : SK-Ed25519 (clef privée jamais sur disque, seulement sur YubiKey)

**Résidu** : si l'attaquant a accès au disque/VM, il peut extraire les clefs passphrasées via GPU cracking hors ligne. **Mitigation** : passphrase forte (16+ caractères) + rotation régulière (90 jours) = fenêtre de temps limitée.

### T4 : Tokens API exposés

**Menace** : un token GitHub/GitLab permet d'ajouter/supprimer des clefs SSH sans limite. S'il est exposé, un attaquant peut accéder à tous les services.

**Atténuation** :
- **Chiffrement GPG** — tokens stockés dans `secrets.yaml.gpg`, qui est chiffré avec votre clef GPG
- **Jamais en logs** — tokens n'apparaissent jamais dans `audit.log`, les messages d'erreur, ou la config
- **Déchiffrement à la demande** — tokens n'existent en mémoire déchiffrée que pendant l'appel API, puis sont oubliés
- **Scopes minimaux** — GitHub : `admin:public_key` seulement (pas de `repo`, `delete_repo`, etc.) ; GitLab : `api` (minimal pour `/user/keys`)

**Résidu** : si GPG est compromis (attaquant a votre passphrase GPG), tous les tokens sont exposés. **Mitigation** : passphrase GPG forte + processus durci bloque les attachements debugger.

### T5 : Compromise du service intermédiaire (GitHub/GitLab)

**Menace** : un attaquant compromet GitHub ou GitLab et ajoute des clefs publiques à votre compte.

**Atténuation** :
- **Vérification post-déploiement** — après déployer une clef, SSHive appelle l'API pour confirmer que sa clef (et seulement sa clef) est listée
- **Audit local** — `audit.log` enregistre tous les déploiements avec fingerprint et timestamp
- **Clefs distinctes par service** — si GitHub est compromis, seul votre accès GitHub est exposé (pas vos autres services)

**Résidu** : si GitHub est compromis longtemps avant la détection, l'attaquant peut créer des clefs SSH. **Mitigation** : audit régulier (vérification hebdomadaire de la liste de clefs publiques sur GitHub).

### T6 : Perte de YubiKey avec SK-Ed25519

**Menace** : la clef privée d'une SK-Ed25519 est stockée physiquement sur le YubiKey. Si le YubiKey est perdu/volé, la clef est inaccessible.

**Atténuation** :
- **Backup du fichier `.pub`** — après génération, SSHive alerte pour sauvegarder le fichier clef publique (nécessaire pour la révocation)
- **Avertissement avant rotation** — avant de renouveler une SK, SSHive alerte : "YubiKey doit rester branchée"
- **Vérification obligatoire** — toute rotation de SK doit passer la vérification post-déploiement (sécurité supplémentaire)

**Résidu** : si le YubiKey est perdu et que vous n'avez pas la sauvegarde, vous ne pouvez pas révoquer la clef à distance. **Mitigation** : sauvegardez le `.pub` immédiatement après génération.

## Protections impliquées

### Chiffrement des secrets

**Fichier** : `~/.config/sshive/secrets.yaml.gpg`

- Chiffré avec **GPG** + votre clef GPG privée
- Nécessite votre **passphrase GPG** pour déchiffrer
- Contenus : tokens API GitHub/GitLab

**Format déchiffré (ne jamais toucher) :**

```yaml
tokens:
  github_token:
    api_token: "ghp_xxxxx..."
  gitlab_token:
    api_token: "glpat-xxxxx..."
```

### Permissions fichiers

| Fichier | Permission | Propriétaire | Justification |
|---------|-----------|--------------|---------------|
| `config.yaml` | `0600` | utilisateur | Lisible par propriétaire seulement |
| `secrets.yaml.gpg` | `0600` | utilisateur | Secrets chiffrés, mais lisible seul par proprio |
| `audit.log` | `0600` | utilisateur | Audit local, sensible |
| `~/.ssh/` | `0700` | utilisateur | Standard SSH |
| `~/.ssh/id_*` | `0600` | utilisateur | Clef privée standard |
| `~/.ssh/id_*.pub` | `0644` | utilisateur | Clef publique, peut être lue par tous |

SSHive vérifie les permissions au démarrage et alerte si elles sont trop permissives.

### Passphrase GPG

La sécurité de tous les secrets dépend de votre **passphrase GPG**.

- Choisissez une passphrase forte (16+ caractères)
- Ne la réutilisez pas sur d'autres services
- Stockez-la en lieu sûr (gestionnaire de mots de passe, papier chiffré, etc.)

### Audit local

Tous les actions sont enregistrées dans `~/.config/sshive/audit.log` (append-only, `0600`) :

```
2026-05-11T14:35:22Z [sshive] Generated SSH key ed25519 for service GitHub
2026-05-11T14:35:45Z [sshive] Deployed key to GitHub via API (verified: true)
2026-05-11T16:20:10Z [sshive] Rotated key for service Production
```

Consultez ce journal pour :
- Détecter les accès non autorisés
- Tracer les rotations et révocations
- Auditer la conformité

### Algorithmes cryptographiques

| Usage | Algorithme | Détails |
|-------|-----------|---------|
| Génération de clefs | Ed25519 | Crypto25519, 256 bits, résistant au quantum |
| Dérivation de passphrase | bcrypt-pbkdf | 16 rounds, SHA512, salé |
| Re-encryption de clefs | AES-256-CTR | 256 bits, mode CTR |
| Chiffrement GPG | GnuPG (RSA/DSA/ECDSA) | Dépend de votre clef (configurable) |
| Signatures HTTP | TLS 1.2+ / RUSTLS | Pas de OpenSSL, WebPKI roots |

## Bonnes pratiques opérationnelles

### Passphrases SSH fortes

- **Minimum** : 12 caractères (configurable, défaut conforme NIST)
- **Recommandé** : 16+ caractères, mélange d'alphanumériques + symboles
- **Exemple** : `"MyGitHub_Prod@2026!Key"`

### Rotation régulière

- **Défaut** : alerte après 90 jours sans rotation
- **Recommandé** : renouveler tous les 60-90 jours pour la production
- **Procédure** : SSHive gère la rotation atomiquement (génère, déploie, archive ancien)

### Passphrase GPG

- **Longueur** : 16+ caractères (ne réutilisez pas d'autres mots de passe)
- **Stockage** : gestionnaire de mots de passe ou papier chiffré
- **Test** : validez que votre passphrase fonctionne régulièrement

### Audit

- **Hebdomadaire** : consultez la page Santé pour alertes
- **Mensuel** : vérifiez `audit.log` pour actions inattendues
- **Annuel** : vérifiez la liste de clefs sur GitHub/GitLab vs config SSHive

### Sauvegarde

- **config.yaml** : sauvegardez régulièrement (versionne-la dans git privé si possible)
- **secrets.yaml.gpg** : sauvegardez le fichier chiffré (illisible sans GPG)
- **SK-Ed25519 `.pub`** : sauvegardez le fichier après génération (nécessaire pour révocation)

## Signalement de vulnérabilités

Si vous découvrez une faille de sécurité dans SSHive :

1. **Ne pas publier** l'exploit dans un issue public
2. **Contactez** l'auteur via email (voir [GitHub](https://github.com/gfriloux/sshive))
3. **Incluez** les détails : version, impact, PoC si pertinent
4. **Attendez** une correction avant divulgation publique

Toutes les vulnérabilités de sécurité sont traitées en priorité.

## Limitations connues et acceptées

1. **Attacker avec root** — si quelqu'un a accès root, il peut extraire les secrets. **Mitigation** : auditez l'accès root sur votre système.

2. **Passphrases faibles** — une passphrase SSH faible peut être cassée en quelques heures. **Mitigation** : imposez une longueur minimale (défaut 12).

3. **Passphrase GPG en mémoire** — après déchiffrer `secrets.yaml.gpg`, la passphrase n'est pas immédiatement oubliée (GPG cache pendant un peu). **Mitigation** : passphrase GPG forte.

4. **Compromise de GPG** — si votre clef GPG est compromise ou volée, tous les secrets le sont. **Mitigation** : protégez votre clef GPG avec une passphrase forte et une protection YubiKey.

---

Voir aussi :
- [Configuration](/reference/configuration/) — format et emplacement des fichiers
- [Clefs SSH](/guide/keys/) — générer et protéger les clefs
- [Tokens API](/guide/tokens/) — configurer les tokens en toute sécurité
