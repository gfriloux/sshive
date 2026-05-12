# SSHive Documentation Site

Site de documentation statique pour SSHive, construit avec [Astro](https://astro.build) + [Starlight](https://starlight.astro.build).

## Structure

```
docs/
├── src/
│   ├── content/
│   │   └── docs/          # Contenu Markdown des pages
│   │       ├── index.mdx  # Page d'accueil
│   │       ├── installation.md
│   │       ├── quickstart.md
│   │       ├── guide/
│   │       └── reference/
│   ├── assets/
│   │   └── logo.svg       # Logo SSHive
│   └── styles/
│       └── custom.css     # Styles personnalisés
├── astro.config.mjs       # Configuration Astro
├── package.json
└── tsconfig.json
```

## Installation locale

```bash
cd docs
npm install
```

## Développement

```bash
npm run dev
```

Lance un serveur de développement à `http://localhost:3000/sshive` avec hot reload.

## Construction

```bash
npm run build
```

Produit un site statique optimisé dans `docs/dist/`.

## Prévisualisation

```bash
npm run preview
```

Prévisualise le site construit localement.

## Déploiement

Le site est déployé automatiquement sur GitHub Pages via `.github/workflows/docs.yml` à chaque push sur `main` contenant des changements dans `docs/`.

**URL** : https://gfriloux.github.io/sshive

## Configuration

### Astro Config (`astro.config.mjs`)

- `site`: `https://gfriloux.github.io` — URL de base du GitHub Pages
- `base`: `/sshive` — sous-chemin du repo
- `output`: `static` — site statique (GitHub Pages compatible)

### Starlight Config

- `title`: "SSHive"
- `logo`: SVG personnalisé depuis `src/assets/logo.svg`
- `sidebar`: structure de navigation à 3 niveaux (Démarrer, Guide, Référence)
- `customCss`: couleurs personnalisées dans `src/styles/custom.css`
- `locales`: French (`fr`) seule

## Éditer le contenu

Ajoutez ou modifiez les fichiers Markdown dans `src/content/docs/` :

```markdown
---
title: Titre de la page
description: Courte description (affichée dans les métadonnées)
---

# Titre

Contenu Markdown...
```

Le sidebar se met à jour automatiquement selon la structure `astro.config.mjs`.

## Styles personnalisés

Modifiez `src/styles/custom.css` pour ajuster les couleurs :

```css
:root {
  --sl-color-accent: #4A80D4;  /* Bleu SSHive */
}
```

Voir [Starlight Theming](https://starlight.astro.build/guides/customization/) pour toutes les variables.

## Intégration avec le dépôt

- **Branche** : `main`
- **Chemin** : `docs/`
- **Workflow** : `.github/workflows/docs.yml` (build + deploy automatique)

## Ressources

- [Astro Documentation](https://docs.astro.build)
- [Starlight Documentation](https://starlight.astro.build)
- [Starlight Components](https://starlight.astro.build/guides/components/)
