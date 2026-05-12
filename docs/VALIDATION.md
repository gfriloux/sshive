# SSHive Documentation — Guide de Validation

Ce guide explique comment valider que la documentation Astro + Starlight est correctement structurée et construite.

## 1. Vérifier la structure des fichiers

### Via script automatisé (recommandé)

```bash
cd /home/kuri/Apps/github/gfriloux/sshive/docs
bash validate-structure.sh
```

Output attendu : tous les fichiers avec ✓

### Manuel

Vérifiez que tous ces fichiers existent :

```
docs/
├── astro.config.mjs
├── package.json
├── tsconfig.json
├── .gitignore
├── .env.example
├── src/
│   ├── content/
│   │   ├── config.ts
│   │   └── docs/
│   │       ├── index.mdx
│   │       ├── installation.md
│   │       ├── quickstart.md
│   │       ├── guide/
│   │       │   ├── services.md
│   │       │   ├── keys.md
│   │       │   ├── deployment.md
│   │       │   ├── health.md
│   │       │   └── tokens.md
│   │       └── reference/
│   │           ├── configuration.md
│   │           ├── security.md
│   │           └── changelog.md
│   ├── assets/
│   │   └── logo.svg
│   └── styles/
│       └── custom.css
├── README.md
├── QUICK_START.md
├── STRUCTURE.md
└── .github/workflows/
    └── docs.yml (16 lignes)
```

**Total** : 27 fichiers

## 2. Tester le build

### Installation

```bash
cd /home/kuri/Apps/github/gfriloux/sshive/docs
npm install
```

Vérifiez que :
- ✓ `node_modules/` est créé
- ✓ `package-lock.json` est créé
- ✓ Les dépendances astro et @astrojs/starlight sont installées

### Build production

```bash
npm run build
```

Output attendu :
```
✓ Completed in XXXms
```

Vérifiez que :
- ✓ Le dossier `dist/` est créé
- ✓ Contient des fichiers HTML (index.html, installation/index.html, etc.)
- ✓ Contient des assets (CSS, JS minifiés)

**Taille attendue** : 2-5 MB (dépendant des optimisations)

### Serveur développement

```bash
npm run dev
```

Output attendu :
```
  ▶ Local:    http://localhost:3000/sshive
```

Vérifiez que :
- ✓ Le serveur se lance sans erreur
- ✓ URL `/sshive` accessible (branding + navigation latérale)
- ✓ Modification d'un fichier `.md` recharge en direct (hot reload)

### Prévisualisation

```bash
npm run preview
```

Vérifiez que :
- ✓ Prévisualise la build de production localement
- ✓ URL `/sshive` fonctionnelle
- ✓ Tous les liens internes fonctionnent

## 3. Valider le contenu

### Frontmatter (YAML)

Chaque page Markdown doit avoir :

```yaml
---
title: "Titre de la page"
description: "Courte description"
---
```

Vérifiez avec :

```bash
grep -r "^---$" src/content/docs/*.md | head -20
```

### Navigation Starlight

Vérifiez que `astro.config.mjs` contient la structure :

```javascript
sidebar: [
  { label: 'Démarrer', items: [...] },
  { label: 'Guide', items: [...] },
  { label: 'Référence', items: [...] },
]
```

Vérifiez que :
- ✓ 3 sections principales
- ✓ Guide a 5 items
- ✓ Référence a 3 items
- ✓ Tous les `link` pointent à des fichiers existants

### Liens internes

Vérifiez que les liens internes sont corrects :

```bash
grep -r "\[.*\](/" src/content/docs/ | head -10
```

Exemple valide : `[Sécurité](/reference/security/)`

### Contenu vérifiée

Vérifiez qu'il n'y a pas de "TODO" ou "PLACEHOLDER" :

```bash
grep -i "TODO\|PLACEHOLDER\|FIXME" src/content/docs/*.md
```

Output attendu : aucune occurrence

## 4. Vérifier la configuration

### Astro

Vérifiez que `astro.config.mjs` contient :

```javascript
site: 'https://gfriloux.github.io'
base: '/sshive'
output: 'static'
```

### Starlight

Vérifiez que `starlight()` contient :

```javascript
title: 'SSHive'
description: 'Gestionnaire de clefs SSH — une clef par service.'
logo: { src: './src/assets/logo.svg' }
defaultLocale: 'root'
locales: { root: { label: 'Français', lang: 'fr' } }
```

### Styles personnalisés

Vérifiez que `src/styles/custom.css` contient les couleurs SSHive :

```bash
grep --color "sl-color-accent" src/styles/custom.css
```

## 5. Test de déploiement GitHub Actions

Vérifiez que `.github/workflows/docs.yml` existe et contient :

```yaml
name: Deploy docs to GitHub Pages
on:
  push:
    branches: [main]
    paths: ['docs/**']
  workflow_dispatch:
```

Vérifiez que les étapes sont présentes :
- ✓ `actions/checkout@v4`
- ✓ `actions/setup-node@v4` (node 22)
- ✓ `npm ci && npm run build`
- ✓ `actions/upload-pages-artifact@v3`
- ✓ `actions/deploy-pages@v4`

## 6. Checklist finale

- [ ] `npm install` réussit sans erreur
- [ ] `npm run build` produit `dist/` sans erreur
- [ ] `npm run dev` se lance et ouvre `http://localhost:3000/sshive`
- [ ] Tous les liens de navigation fonctionnent
- [ ] Pas de "TODO", "PLACEHOLDER" ou erreurs Markdown
- [ ] Couleurs personnalisées appliquées (bleu #4A80D4)
- [ ] Logo SVG chargé correctement
- [ ] `.github/workflows/docs.yml` est présent
- [ ] Tous les chemins sont en français
- [ ] Aucun fichier contient de contenu placeholder

## 7. Commandes de test rapide

```bash
# Aller au dossier docs
cd /home/kuri/Apps/github/gfriloux/sshive/docs

# Valider structure
bash validate-structure.sh

# Installer dépendances
npm install

# Test rapide (node 22)
npm run build 2>&1 | tail -20

# Afficher taille du dist
du -sh dist/

# Vérifier pas de TODO
grep -ri "TODO\|PLACEHOLDER" src/

# Vérifier structure frontmatter
find src/content/docs -name "*.md" -o -name "*.mdx" | xargs head -5
```

## Prochaines étapes

1. **Push vers main** avec changements dans `docs/`
2. **GitHub Actions** se déclenche automatiquement
3. **Site live** à https://gfriloux.github.io/sshive en ~2-3 minutes

---

Tous les tests doivent passer pour que le site soit prêt au déploiement.
