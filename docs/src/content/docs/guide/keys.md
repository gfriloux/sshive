---
title: Clefs SSH
description: Générer, gérer et renouveler vos clefs SSH.
---

Dans SSHive, les **clefs SSH** sont les artefacts cryptographiques qui autorisent l'accès à vos services. Chaque clef est stockée localement, protégée et peut être attachée à plusieurs services.

## Types de clefs

### Ed25519

Clef de signature Curve25519, recommandée pour la plupart des cas.

**Avantages :**
- Cryptographie moderne et robuste
- Courtes (64 caractères de fingerprint)
- Rapides à vérifier
- Largement supportées par GitHub, GitLab, OpenSSH

**Emplacement par défaut** : `~/.ssh/id_sshive_github` (ou similar)

### SK-Ed25519 (YubiKey / FIDO2)

Clef matérielle utilisant une YubiKey ou autre token FIDO2.

**Avantages :**
- Clef privée jamais stockée sur disque (hardware seulement)
- Nécessite l'appui physique du bouton pour chaque signature
- Meilleure protection contre le vol/copie de clef
- Supportée par GitHub et OpenSSH 8.2+

**Prérequis** : YubiKey 5+ compatible FIDO2, ou autre token (Titan, etc.)

**Emplacement par défaut** : `~/.ssh/id_sshive_github_sk`

**Important** : le fichier `.pub` est stocké sur disque (comme pour ed25519), mais la clef privée vit uniquement dans le hardware. Après génération, **sauvegardez le fichier `.pub` et notez le chemin du fichier clef privée** en cas de perte du hardware.

## Générer une clef

Depuis la page **Clefs** :

1. Cliquez **"Générer une clef"**
2. Sélectionnez le **service cible** dans la liste déroulante
3. Choisissez le type : **ed25519** (recommandé) ou **sk-ed25519** (YubiKey)
4. Entrez une **passphrase** robuste (≥ 12 caractères)
5. Cliquez **Générer**

SSHive exécute `ssh-keygen`, stocke la clef dans `~/.ssh/`, enregistre le fingerprint dans la configuration et attache la clef au service sélectionné.

## Attacher une clef existante à un service

Si vous avez déjà une clef dans SSHive et souhaitez l'associer à un service sans régénérer :

1. Sélectionnez le service dans la liste (page **Services**)
2. Dans le panneau de détail, section **Clef SSH**, cliquez **"Attacher une clef existante"**
3. Sélectionnez la clef dans le sélecteur — seules les clefs gérées (avec clef privée) sont proposées
4. La liaison est sauvegardée immédiatement

:::tip
Un service créé sans clef affiche un avertissement **Critical** dans la vue Santé. Vous pouvez résoudre cela en générant une nouvelle clef ou en en attachant une existante.
:::

## Protéger une clef existante

Si vous avez des clefs SSH existantes dans `~/.ssh/` sans passphrase, SSHive les détecte et alerte.

1. Ouvrez le détail de la clef
2. Cliquez **Ajouter une passphrase**
3. Saisissez une passphrase (≥ 12 caractères)
4. Confirmez

La clef est re-chiffrée localement sans toucher à sa clef publique.

## Renouveler une clef (rotation)

Pour remplacer une clef usée ou compromise :

1. Sélectionnez le service qui utilise la clef
2. Cliquez **Renouveler la clef SSH** dans le détail
3. SSHive affiche un avertissement si c'est une SK (YubiKey doit rester branchée)
4. Confirmez
5. Une nouvelle clef est générée
6. L'ancienne clef est listée dans **Clefs précédentes** avec un bouton **Révoquer**

**Important pour SK-Ed25519** : pendant toute l'opération de rotation, le YubiKey doit rester branché. Une vérification post-rotation est obligatoire.

## Voir toutes les clefs

1. Cliquez **Santé** dans la barre latérale
2. Vous verrez une liste de toutes vos clefs SSH avec :
   - Fingerprint (en monospace)
   - Type (ed25519 / sk-ed25519)
   - Badge YubiKey si applicable
   - Âge (jours depuis génération)
   - Services qui l'utilisent
   - État de protection (✓ protégée / ⚠ non protégée)

## Copier une clef publique

Pour partager votre clef publique :

1. Sélectionnez la clef dans la liste des clefs SSH
2. Cliquez **Copier la clef publique**
3. La clef est copiée dans le presse-papiers
4. Le bouton affiche **"✓ Copié"** pendant 2 secondes

Vous pouvez alors la coller dans un navigateur, un formulaire, etc.

## Révoquer une clef

**Local** (suppression de `~/.ssh/`) :

1. Allez à la vue Clefs SSH
2. Sélectionnez la clef
3. Cliquez **Supprimer localement**
4. Le fichier `.pub` et la clef privée sont supprimés de `~/.ssh/`

**Distant** (suppression de GitHub/GitLab) :

1. Sélectionnez le service qui utilise la clef
2. Cliquez **Révoquer** sur la clef
3. SSHive appelle l'API pour supprimer la clef du service
4. L'entrée `~/.ssh/authorized_keys` sur les serveurs SSH est supprimée

## Gestion des passphrases

### Saisie des passphrases

Chaque fois que SSHive a besoin d'une passphrase (génération, rotation, signature), elle lance un **pinentry graphique** :

- **GNOME** → `pinentry-gnome3`
- **KDE** → `pinentry-qt`
- **Autres** → `pinentry-gtk-2`

Vous pouvez forcer un backend avec `export SSHIVE_PINENTRY=pinentry-gtk-2`.

### Chiffrement de passphrases

Les passphrases ne sont **jamais** stockées en clair. SSHive les chiffre immédiatement avec GPG et les oublie.

Chaque fois qu'une passphrase est nécessaire, SSHive la demande via pinentry.

### Longueur minimale

Par défaut, les passphrases doivent faire ≥ 12 caractères. Vous pouvez ajuster ce seuil dans **Paramètres** → **Sécurité** → **Longueur minimale du mot de passe**.

## Audit des clefs

Chaque génération, rotation et révocation de clef est enregistrée dans le journal d'audit :

```
~/.config/sshive/audit.log
```

Exemple :

```
2026-05-11T10:34:22Z [sshive] Generated SSH key ed25519 for service GitHub (fingerprint: abc123...)
2026-05-11T10:35:10Z [sshive] Deployed key to GitHub via API
2026-05-11T14:22:45Z [sshive] Rotated key for service Production (new fingerprint: def456...)
```

## Bonnes pratiques

**Passphrases fortes** — utilisez au moins 12 caractères (conforme au NIST SP 800-63B).

**Pas de réutilisation** — chaque service devrait avoir sa propre clef, pas une clef partagée.

**Rotation régulière** — SSHive vous alerte quand une clef approche de 90 jours. Renouvelez-la.

**Sauvegarde YubiKey** — après génération d'une SK-Ed25519, sauvegardez le fichier `.pub` et notez le chemin du fichier clef privée en cas de perte du hardware.

**Audit** — consultez régulièrement `audit.log` pour vérifier les actions sur les clefs.

---

Voir aussi :
- [Services](/guide/services/) — attacher des clefs aux services
- [Déploiement](/guide/deployment/) — déployer les clefs
- [Santé](/guide/health/) — diagnostics de sécurité
