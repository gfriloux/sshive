---
title: Santé et diagnostics
description: Comprendre l'état de santé de vos services et clefs SSH.
---

La **page Santé** (accessible via la barre latérale) affiche une vue diagnostique complète de tous vos services et clefs, ainsi que les alertes de sécurité.

## Niveaux de santé

Chaque service et clef reçoit un **niveau de santé** indiqué par une pastille et un label :

### ✓ ok (vert)

Tout est en ordre.

**Critères :**
- Clef SSH attachée au service
- Clef protégée par passphrase
- Si GitHub/GitLab : token API valide (dernièrement vérifié)
- Clef déployée avec succès
- Pas de rotation dépassée

### ⓘ info (bleu)

Situation à noter, pas d'urgence.

**Raisons courantes :**
- `HardwareKeyHandleNotBackedUp` — clef SK (YubiKey) dont le fichier `.pub` n'a pas été sauvegardé
- Clef de plus de 30 jours sans rotation
- Déploiement en cours

**Action suggérée :** sauvegardez le fichier de clef ou planifiez une rotation.

### ⚠ warning (orange)

Situation à corriger prochainement.

**Raisons courantes :**
- `NoKey` — aucune clef attribuée au service
- `NoApiToken` — service GitHub/GitLab sans token API (déploiement manuel seulement)
- `KeyUnprotected` — clef SSH existante sans passphrase
- `RotationRecommended` — clef atteint le seuil d'alerte (défaut 90 jours)

**Action suggérée :** générez une clef, ajoutez un token API, ou protégez la clef existante.

### ✗ critical (rouge)

Problème de sécurité immédiat.

**Raisons courantes :**
- `NoKey` + `RotationOverdue` — service sans clef ET clef actuelle trop ancienne
- `NoApiToken` + `RotationOverdue` — service GitHub/GitLab sans token ET clef trop ancienne

**Action suggérée :** générez immédiatement une nouvelle clef et déployez-la.

## Raisons de santé détaillées

### Service : aucune clef assignée

```
NoKey
```

**Signification :** le service n'a pas de clef SSH attachée.

**Causes possibles :**
- Service venant d'être créé, clef non encore générée
- Clef précédente supprimée
- Erreur lors de la création du service

**Correction :**
1. Allez au détail du service
2. Cliquez **Générer une nouvelle clef** ou **Sélectionner une clef existante**
3. Déployez la clef

### Service : clef non protégée

```
KeyUnprotected
```

**Signification :** la clef SSH n'a pas de passphrase.

**Risque :** si quelqu'un accède à `~/.ssh/`, il peut utiliser la clef sans saisir de passphrase.

**Correction :**
1. Allez au détail de la clef
2. Cliquez **Ajouter une passphrase**
3. Entrez une passphrase (≥ 12 caractères)

### Rotation dépassée

```
RotationOverdue (85 jours)
```

**Signification :** la clef n'a pas été renouvelée depuis longtemps.

**Causes possibles :**
- Clef créée il y a plus de X jours (défaut 90 jours)
- Pas de rotation planifiée
- Configuration de seuil trop restrictive

**Correction :**
1. Allez au détail du service
2. Cliquez **Renouveler la clef SSH**
3. Confirmez la rotation
4. La nouvelle clef est générée et déployée
5. L'ancienne clef est archivée

Vous pouvez ajuster le seuil d'alerte dans **Paramètres** → **Sécurité** → **Rotation warning threshold**.

### Pas de token API

```
NoApiToken
```

**Signification :** le service est GitHub/GitLab mais aucun token API n'est configuré.

**Impact :**
- Déploiement en mode manuel seulement
- Pas de vérification post-déploiement
- Pas de révocation automatique

**Correction :**
1. Allez au détail du service
2. Cliquez **Éditer**
3. À l'étape 2, cliquez **Configurer le token**
4. Saisissez le token API valide
5. Validez

### Fichier clef YubiKey à sauvegarder

```
HardwareKeyHandleNotBackedUp
```

**Signification :** clef SK (YubiKey) dont vous n'avez pas confirmé la sauvegarde du fichier.

**Risque :** si le YubiKey est perdu ou détruit, vous ne pouvez pas récupérer la clef.

**Correction :**
1. Allez au détail de la clef SK
2. Notez le chemin du fichier clef privée (ex : `~/.ssh/id_sshive_github_sk`)
3. Cliquez **Compris — j'ai sauvegardé**
4. L'alerte disparaît

## Vue diagnostique

![Page Santé de SSHive — vue diagnostique avec tuiles ok/warning/critical et liste priorisée](../../../assets/screenshots/health-page.png)

*Vue diagnostique : les services sont regroupés par niveau de criticité, les problèmes les plus urgents en tête de liste.*

La page **Santé** affiche une table avec :

| Colonne | Signification |
|---------|--------------|
| Service | Nom du service |
| Type | GitHub, GitLab, SSH, etc. |
| Santé | Pastille + niveau (ok/info/warning/critical) |
| Clef | Fingerprint de la clef attachée (ou "—") |
| Âge | Jours depuis génération (ex : "45 jours") |
| Protection | ✓ passphrase / ⚠ pas de passphrase |
| Token API | ✓ valide / ⚠ absent / ✗ invalide (GitHub/GitLab only) |
| Déploiement | ✓ ok / ⚠ en attente / ✗ erreur |

Cliquez sur une ligne pour voir le détail complet du service et les actions disponibles.

## Alertes en temps réel

### Alerte santé critique

Si l'un de vos services atteint **critical**, une pastille **pulsante rouge** apparaît dans la barre latérale, à côté de **Santé**. Cela vous avertit immédiatement.

L'alerte disparaît quand tous les services sont en ok/info/warning.

## Configurer les seuils

Allez à **Paramètres** → **Sécurité** pour ajuster :

- **Rotation warning threshold** (défaut 90 jours) — nombre de jours avant qu'une clef passe en warning
- **Minimum passphrase length** (défaut 12 caractères) — longueur minimale requise pour les passphrases

## Audit complet

Consultez le journal d'audit pour voir toutes les actions :

```bash
cat ~/.config/sshive/audit.log
```

Exemple :

```
2026-05-11T14:35:22Z [sshive] Generated SSH key ed25519 for service GitHub
2026-05-11T14:35:45Z [sshive] Deployed key to GitHub via API (verified: true)
2026-05-11T16:20:10Z [sshive] Rotated key for service Production
2026-05-12T09:10:33Z [sshive] Revoked key from GitLab
```

## Bonnes pratiques

**Consulter Santé régulièrement** — chaque semaine, vérifiez la page Santé pour détecter les alertes.

**Réagir aux avertissements** — une alerte warning signifie que vous devez agir dans les prochains jours.

**Rotations planifiées** — ne pas attendre que le seuil soit atteint ; renouveler les clefs tous les 60-90 jours pour une meilleure sécurité.

**Passphrases fortes** — toujours >= 12 caractères, idéalement 16+ (phrases de passe conseillées).

**Sauvegarde YubiKey** — après générer une SK-Ed25519, marquez immédiatement comme sauvegardée pour éviter les faux positifs.

**Zéro critical en production** — s'il y a un service critical, c'est bloquant ; mettez en place une rotation pour le résoudre.

---

Voir aussi :
- [Services](/guide/services/) — créer et gérer les services
- [Clefs SSH](/guide/keys/) — générer et renouveler les clefs
- [Tokens API](/guide/tokens/) — configurer les tokens GitHub/GitLab
