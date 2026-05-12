---
title: Tokens API
description: Configurer et gérer les tokens API GitHub et GitLab.
---

SSHive utilise des **tokens API** pour accéder à GitHub et GitLab afin de déployer et révoquer vos clefs SSH automatiquement, sans avoir à les copier manuellement.

## Pourquoi les tokens sont nécessaires

Les tokens permettent à SSHive de :

1. **Déployer les clefs** via l'API (au lieu de `ssh-copy-id`)
2. **Révoquer les clefs** à distance (en cas de compromission)
3. **Vérifier le succès** du déploiement immédiatement

Sans token, vous êtes limité au **mode Guidé** (copier-coller les commandes).

## Sécurité des tokens

SSHive traite les tokens avec attention :

- **Chiffrement GPG** — tous les tokens sont immédiatement chiffrés et stockés dans `~/.config/sshive/secrets.yaml.gpg`
- **Jamais en mémoire inutilement** — les tokens ne sont déchiffrés que quand ils sont utilisés
- **Jamais dans les logs** — les tokens ne sont jamais affichés dans `audit.log` ou les messages d'erreur
- **Révocation facile** — supprimez le token du service dans SSHive pour le "oublier" localement

Voir la [Politique de sécurité](/reference/security/) pour plus de détails.

## Tokens GitHub

### Obtenir un token GitHub

1. Allez à [github.com/settings/tokens](https://github.com/settings/tokens)
2. Cliquez **Generate new token (classic)**
3. Donnez-lui un nom : `SSHive`
4. **Scopes à cocher (IMPORTANT)** :
   - ✓ `admin:public_key` — **SEUL scope à cocher**
   
   Ce scope inclut implicitement `read:public_key`, ce qui suffit pour déployer et vérifier les clefs.

5. Cliquez **Generate token**
6. **Copiez le token immédiatement** — GitHub ne le montrera qu'une fois

### Configurer le token dans SSHive

**Lors de la création d'un service GitHub :**

1. À l'**Étape 2**, collez le token dans le champ **Token API**
2. Cliquez **Suivant**

**Pour un service GitHub existant :**

1. Sélectionnez le service dans la liste
2. Cliquez **Éditer**
3. À l'étape 2, cliquez **Configurer le token**
4. Collez le nouveau token (il remplace l'ancien)
5. Cliquez **Valider**

### Scope détaillé

Le scope `admin:public_key` inclut :

- `read:public_key` — lire les clefs publiques de l'utilisateur
- `write:public_key` — ajouter, supprimer les clefs publiques

C'est exactement ce que SSHive nécessite. **N'accordez pas de scopes supplémentaires.**

### Tester le token

SSHive teste automatiquement le token avant de l'utiliser :

1. Valide le format du token (commence par `ghp_` ou `github_pat_`)
2. Appel API `GET /user` pour vérifier l'authentification
3. Alerte **DeployBlocker** si le test échoue

## Tokens GitLab.com

### Obtenir un token GitLab

**Pour GitLab.com :**

1. Allez à [gitlab.com/profile/personal_access_tokens](https://gitlab.com/profile/personal_access_tokens)
2. Cliquez **Add new token**
3. Nom : `SSHive`
4. **Scopes à cocher (IMPORTANT)** :
   - ✓ `api` — accès à l'API (inclut `/user/keys`)

5. Cliquez **Create personal access token**
6. **Copiez le token immédiatement** — GitLab ne le montrera qu'une fois

### Configurer le token dans SSHive

Même procédure que pour GitHub :

1. Créez ou éditez un service GitLab.com
2. À l'étape 2, collez le token dans le champ **Token API**
3. Validez

## Tokens GitLab auto-hébergé

### Obtenir un token (instance privée)

Pour une instance GitLab privée (ex : `https://gitlab.example.com`) :

1. Allez à `https://gitlab.example.com/profile/personal_access_tokens`
2. Cliquez **Add new token**
3. Nom : `SSHive`
4. **Scope** : ✓ `api`
5. Cliquez **Create personal access token**
6. Copiez le token

### Configurer dans SSHive

1. Créez un service GitLab auto-hébergé
2. Entrez l'**URL de base** : `https://gitlab.example.com`
3. À l'étape 2, collez le token
4. Validez

SSHive utilisera l'URL pour faire les appels API automatiquement.

## Gestion des tokens

### Lister les tokens configurés

Allez à **Paramètres** → **Données** pour voir la liste des services avec tokens configurés.

Les tokens eux-mêmes ne sont pas affichés (pour la sécurité), seulement leur présence est indiquée.

### Supprimer un token

1. Sélectionnez le service dans la liste
2. Cliquez **Éditer**
3. À l'étape 2, cliquez **Supprimer le token**
4. Validez

Le token local est oublié. Le token sur GitHub/GitLab reste valide jusqu'à ce que vous le révoquiez manuellement sur le site.

### Révoquer un token sur le site

Si vous avez compromis un token ou souhaitez l'arrêter :

**GitHub :**

1. Allez à [github.com/settings/tokens](https://github.com/settings/tokens)
2. Trouvez le token `SSHive`
3. Cliquez **Delete**

**GitLab.com :**

1. Allez à [gitlab.com/profile/personal_access_tokens](https://gitlab.com/profile/personal_access_tokens)
2. Trouvez le token
3. Cliquez la poubelle

La révocation est immédiate. Les futures tentatives de SSHive échoueront.

## Rotation de tokens

Il est bonne pratique de **changer les tokens régulièrement** (tous les 6-12 mois).

**Procédure :**

1. Générez un nouveau token sur GitHub/GitLab
2. Allez à SSHive et mettez à jour le token du service (Éditer → Configurer le token)
3. Vérifiez que le déploiement fonctionne
4. Allez à GitHub/GitLab et supprimez l'ancien token

## Dépannage

### "Token invalide" ou "Authentification échouée"

- Vérifiez que vous avez copié l'**intégralité du token** (pas de caractères manquants)
- Vérifiez que le token n'a **pas encore expiré**
- Vérifiez les **scopes** : GitHub doit avoir `admin:public_key`, GitLab doit avoir `api`

### Service GitHub/GitLab aparaît en "warning : NoApiToken"

Cela signifie qu'aucun token n'est configuré. Allez à **Éditer** le service et ajoutez le token.

### Vérification du token échoue avec "DeployBlocker"

- Vérifiez la **connectivité Internet** (SSHive appelle l'API)
- Vérifiez que le token n'a **pas expiré** ou a été révoqué
- Vérifiez que votre compte GitHub/GitLab n'a pas de problème (2FA bloquant, etc.)

## Bonnes pratiques

**Un token par service, ou un partagé ?** — Pour la simplicité, vous pouvez utiliser le même token pour tous vos services GitHub. Pour l'isolation, générez un token par service (pas nécessaire pour la sécurité, juste la traçabilité).

**Passphrases fortes sur les tokens** — les tokens sont stockés chiffrés avec GPG, donc leur sécurité dépend de votre passphrase GPG.

**Révoquez immédiatement en cas de fuite** — si vous exposez accidentellement un token, allez sur GitHub/GitLab et supprimez-le.

**Stockez nulle part ailleurs** — n'ajoutez jamais un token à `config.yaml` ou un fichier non chiffré. SSHive le fait pour vous (secrets.yaml.gpg).

**Rotation régulière** — tous les 6-12 mois, générez un nouveau token et remplacez l'ancien dans SSHive.

---

Voir aussi :
- [Services](/guide/services/) — créer des services GitHub/GitLab
- [Sécurité](/reference/security/) — politique et philosophie de sécurité
- [Configuration](/reference/configuration/) — structure des fichiers de configuration
