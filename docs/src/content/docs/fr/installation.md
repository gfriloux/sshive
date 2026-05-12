---
title: Installation
description: Installez SSHive sur votre système Linux avec Nix.
---

SSHive se distribue via **flake Nix** et est conçu pour fonctionner sur NixOS et tout système Linux avec Nix.

## Prérequis

- **Nix 2.13+** ([installer Nix](https://nixos.org/download.html))
- **GPG 2.2+** — déjà inclus dans la plupart des distributions Linux
- **SSH configuré** — clefs publiques/privées existantes dans `~/.ssh/`

## Installation rapide

Pour exécuter SSHive sans l'installer de manière permanente :

```bash
nix run github:gfriloux/sshive
```

C'est la meilleure façon de l'essayer pour la première fois.

## Installation persistante (recommandé)

Pour installer SSHive dans votre profil utilisateur :

```bash
nix profile install github:gfriloux/sshive
```

Lancez ensuite :

```bash
sshive
```

L'application crée automatiquement le répertoire `~/.config/sshive/` au premier lancement.

## Développement local

Si vous souhaitez développer ou compiler à partir de la source :

```bash
git clone https://github.com/gfriloux/sshive.git
cd sshive
nix develop
```

Vous êtes maintenant dans un shell de développement avec Rust, Cargo et toutes les dépendances. Pour lancer l'application :

```bash
cargo run --release
```

## Configuration initiale

Au premier lancement, SSHive crée la structure suivante :

```
~/.config/sshive/
├── config.yaml         # Configuration services et clefs
├── audit.log           # Journal des actions (chmod 0600)
└── secrets.yaml.gpg    # Tokens API chiffrés avec GPG
```

L'application scanne aussi automatiquement `~/.ssh/` pour découvrir vos clefs publiques existantes (ed25519 et sk-ed25519).

### Sélectionner une clef GPG

Au premier lancement, SSHive vous demandera de sélectionner une clef GPG pour chiffrer les secrets. Vous pouvez :

1. **Sélectionner une clef existante** — si vous avez déjà une clef GPG
2. **Créer une nouvelle clef** — SSHive peut générer une clef Ed25519 directement en GUI
3. **Utiliser le terminal** — exécuter `gpg --generate-key` si vous préférez

### Backend Pinentry

SSHive détecte automatiquement votre environnement de bureau et sélectionne le bon agent de saisie de passphrase :

- **GNOME** → `pinentry-gnome3`
- **KDE** → `pinentry-qt`
- **Autres** → `pinentry-gtk-2` (fallback)

Vous pouvez forcer un backend spécifique avec :

```bash
export SSHIVE_PINENTRY=pinentry-gnome3
sshive
```

## Mise à jour

Pour mettre à jour vers la dernière version :

```bash
nix profile upgrade
```

Ou si vous utilisez `nix run` :

```bash
nix run --update-input sshive github:gfriloux/sshive
```

## Dépannage

### SSHive ne démarre pas

- Vérifiez que GPG est fonctionnel : `gpg --list-keys`
- Vérifiez les logs : `~/.config/sshive/audit.log`
- Assurez-vous que vous avez au moins une clef GPG disponible

### Passphrases ne s'affichent pas

- Vérifiez que `pinentry` est installé : `which pinentry-gnome3` (ou votre backend)
- Essayez de forcer un backend : `SSHIVE_PINENTRY=pinentry-gtk-2 sshive`

### Problèmes de permissions

SSHive crée tous les fichiers en mode `0600` (lecture/écriture utilisateur seul). Si vous voyez des erreurs de permission :

```bash
# Vérifiez les permissions du répertoire de config
ls -la ~/.config/sshive/
```

Elles doivent être `drw-------` (0700). Si ce n'est pas le cas :

```bash
chmod 700 ~/.config/sshive/
chmod 600 ~/.config/sshive/*
```

## Prochaines étapes

Une fois installé, suivez le [Démarrage rapide](/quickstart/) pour ajouter votre premier service.
