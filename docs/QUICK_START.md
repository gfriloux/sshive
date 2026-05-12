# Quick Start — Documentation SSHive

## Installation rapide

```bash
cd docs
npm install
```

## Développement local

```bash
npm run dev
```

Ouvre http://localhost:3000/sshive dans votre navigateur. Le site se met à jour en temps réel lors de vos modifications.

## Build pour production

```bash
npm run build
```

Produit un site statique dans `dist/`.

## Prévisualization de la build

```bash
npm run preview
```

Teste la build de production localement.

## Ajouter une nouvelle page

1. Créez un fichier `.md` ou `.mdx` dans `src/content/docs/`
2. Ajoutez le frontmatter YAML :

```yaml
---
title: Titre de la page
description: Courte description
---

# Contenu
```

3. Ajoutez l'entrée au sidebar dans `astro.config.mjs`

## Structure

```
src/content/docs/
├── index.mdx              # Page d'accueil
├── installation.md
├── quickstart.md
├── guide/
│   ├── services.md
│   ├── keys.md
│   ├── deployment.md
│   ├── health.md
│   └── tokens.md
└── reference/
    ├── configuration.md
    ├── security.md
    └── changelog.md
```

## Déploiement

Automatique sur GitHub Pages via `.github/workflows/docs.yml` à chaque push vers `main` avec modifications dans `docs/`.

URL finale : https://gfriloux.github.io/sshive

## Ressources

- [Astro Docs](https://docs.astro.build)
- [Starlight Docs](https://starlight.astro.build)
- [Markdown Components](https://starlight.astro.build/guides/components/)
