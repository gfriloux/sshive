---
title: Démarrage rapide
description: Ajouter un premier service GitHub en 5 minutes.
---

Ce guide vous montre comment ajouter votre premier service (GitHub) et déployer une clef SSH en 5 minutes.

## Étape 1 : Lancez SSHive

```bash
sshive
```

Vous verrez l'interface avec trois colonnes : **Navigation** (gauche), **Services** (centre), **Détail** (droite).

## Étape 2 : Créez un nouveau service

1. Cliquez sur **"Ajouter mon premier service"** dans la liste des services
2. Remplissez les informations du service :
   - **Nom** : `GitHub` (ou n'importe quel nom unique)
   - **Type** : `GitHub`
   - **Hostname** : `github.com` (auto-rempli)

3. Cliquez **Suivant**

## Étape 3 : Configurez le déploiement

1. Choisissez **Mode automatique** — SSHive utilisera l'API GitHub pour déployer
2. Collez votre **token GitHub** avec le scope `admin:public_key`

**Comment obtenir un token GitHub :**

- Allez à [github.com/settings/tokens](https://github.com/settings/tokens)
- Cliquez **Generate new token (classic)**
- Scopes : cochez uniquement **`admin:public_key`**
- Générez et copiez le token dans SSHive

3. Cliquez **Suivant**

## Étape 4 : Générez une clef SSH

1. Choisissez **Générer une nouvelle clef**
2. Type : **ed25519** (par défaut, recommandé)
3. Passphrase : saisissez une phrase de passe ≥ 12 caractères
4. Cliquez **Générer**

SSHive chiffre la passphrase avec GPG et génère la clef. Vous verrez le fingerprint affiché.

## Étape 5 : Déployez la clef

1. Cliquez **Déployer**
2. SSHive envoie la clef à GitHub via l'API
3. Une vérification post-déploiement est effectuée
4. Un message "✓ Connexion vérifiée" confirme le succès

Bravo ! Votre première clef est maintenant déployée sur GitHub.

## Étape 6 : Vérifiez la santé

1. Cliquez sur **Santé** dans la barre latérale
2. Vous verrez votre service GitHub listé avec le statut **ok** (point vert)

## Prochaines étapes

- **Ajouter un serveur SSH** — voir [Services](/guide/services/)
- **Comprendre les modes de déploiement** — voir [Déploiement](/guide/deployment/)
- **Renouveler une clef** — voir [Clefs SSH](/guide/keys/)
- **Gérer les tokens API** — voir [Tokens API](/guide/tokens/)

## Astuces

### Tester avec un service SSH générique (pas de compte public requis)

Si vous n'avez pas encore de compte GitHub, vous pouvez tester avec un service SSH générique :

1. Créez un nouveau service avec type **SSH générique**
2. Remplissez les informations de connexion :
   - **Hostname** : `127.0.0.1` ou votre machine locale
   - **Username** : `root`
   - **Port** : `22`
3. Générez et déployez une clef comme ci-dessus

### Copier une clef publique existante

Si vous avez déjà des clefs dans `~/.ssh/`, SSHive les découvre automatiquement. Vous pouvez les rattacher à un service sans les régénérer :

1. Ouvrez le détail du service
2. Cherchez la clef dans la liste **Clefs SSH disponibles**
3. Cliquez **Sélectionner** pour l'attacher

### YubiKey (clef de sécurité matérielle)

Pour générer une clef sk-ed25519 (requiert un YubiKey ou autre FIDO2 USB) :

1. Dans l'étape de génération de clef, choisissez **SK-Ed25519** au lieu de ed25519
2. Suivez les instructions pour appuyer sur le bouton YubiKey quand demandé
3. La clef privée est sauvegardée dans `~/.ssh/` — sauvegardez le fichier en lieu sûr

---

Vous êtes prêt. Consultez les [Guides détaillés](/guide/services/) pour en savoir plus sur chaque fonctionnalité.
