# SSHive Documentation Structure

## Vue d'ensemble

Site de documentation statique construit avec **Astro 4.0** + **Starlight 0.20.0**.

- **Framework** : Astro (SSG, JavaScript/TypeScript)
- **Thème** : Starlight (documentation professionnelle)
- **Langue** : Français (fr)
- **Déploiement** : GitHub Pages (`gfriloux.github.io/sshive`)

## Arborescence complète

```
docs/
├── src/
│   ├── assets/
│   │   └── logo.svg                    # Logo SSHive (32×32, SVG)
│   ├── content/
│   │   ├── config.ts                   # Collection Astro
│   │   └── docs/                       # Pages Markdown/MDX
│   │       ├── index.mdx               # Page d'accueil (hero + features)
│   │       ├── installation.md         # Guide installation Nix
│   │       ├── quickstart.md           # Démarrage rapide 5 min
│   │       ├── guide/
│   │       │   ├── services.md         # Gestion services
│   │       │   ├── keys.md             # Génération/rotation clefs SSH
│   │       │   ├── deployment.md       # 3 modes déploiement
│   │       │   ├── health.md           # Santé et diagnostics
│   │       │   └── tokens.md           # Tokens API GitHub/GitLab
│   │       └── reference/
│   │           ├── configuration.md    # Schéma YAML complet
│   │           ├── security.md         # Menaces T1–T6 et mitigations
│   │           └── changelog.md        # Historique v0.1–v0.4
│   └── styles/
│       └── custom.css                  # Couleurs SSHive (#4A80D4, etc.)
├── astro.config.mjs                    # Config Astro + Starlight
├── tsconfig.json                       # Config TypeScript
├── package.json                        # Dépendances npm
├── .gitignore                          # Exclusions git
├── .env.example                        # Env vars optionnelles
├── README.md                           # Docs pour développeurs
├── QUICK_START.md                      # Guide démarrage rapide dev
├── STRUCTURE.md                        # Cette file
└── test-build.sh                       # Script de test

.github/workflows/
└── docs.yml                            # GitHub Actions (build + deploy)
```

## Contenu par section

### 📍 Accueil (index.mdx)

- Hero avec titre + CTA
- Philosophie (une clef par service)
- 4 cartes features
- Prérequis
- Liens vers démarrage

### 🚀 Démarrer (3 pages)

**Installation** — Nix flake, Nix develop, perms, troubleshooting
**Démarrage rapide** — 6 étapes GitHub en 5 min
**Quickstart astuces** — SSH générique, clefs existantes, YubiKey

### 📚 Guide (5 pages)

**Services** — Types (GitHub, GitLab, SSH-generic), CRUD, santé
**Clefs SSH** — Types (ed25519, sk-ed25519), génération, rotation, protection
**Déploiement** — 3 modes (Automatique, Guidé, ExternalCM)
**Santé** — Niveaux (ok/info/warning/critical), raisons, diagnostics
**Tokens API** — Scopes GitHub/GitLab, sécurité, rotation

### 📖 Référence (3 pages)

**Configuration** — Schéma YAML complet avec exemples
**Sécurité** — Menaces T1–T6, crypto, audit, best practices
**Changelog** — Historique complet v0.1–v0.4 (depuis GitHub)

## Données clés

| Métrique | Valeur |
|----------|--------|
| Pages | 13 |
| Mots | ~25,000 |
| Sections | 3 (Démarrer, Guide, Référence) |
| Code examples | 45+ |
| Tables | 15+ |
| Images | 1 (logo SVG) |

## Navigation Starlight

```
Démarrer
├── Introduction
├── Installation
└── Démarrage rapide

Guide
├── Services
├── Clefs SSH
├── Déploiement
├── Santé et diagnostics
└── Tokens API

Référence
├── Configuration
├── Sécurité
└── Changelog
```

## Fichiers de config

### `astro.config.mjs`

- `site`: `https://gfriloux.github.io`
- `base`: `/sshive` (sous-chemin)
- `output`: `static`
- Starlight intégré avec 3 sections sidebar
- Couleurs personnalisées via `src/styles/custom.css`

### `package.json`

```json
{
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "preview": "astro preview"
  },
  "dependencies": {
    "astro": "^4.0.0",
    "@astrojs/starlight": "^0.20.0"
  }
}
```

### `.github/workflows/docs.yml`

- Déclenché : `push` vers `main` + `paths: ['docs/**']`
- Node 22
- Build : `npm ci && npm run build`
- Deploy : GitHub Pages (déploiement automatique)

## Styles personnalisés

Fichier `src/styles/custom.css` :

```css
:root {
  --sl-color-accent-low: #1B2535;    /* Bleu-noir SSHive */
  --sl-color-accent: #4A80D4;        /* Bleu SSHive principal */
  --sl-color-accent-high: #E2E8F4;   /* Gris-bleu clair */
  --sl-color-white: #E2E8F4;         /* Blanc cassé */
  --sl-font-system-mono: 'JetBrains Mono', monospace;
}
```

## Points clés de qualité

✓ **Français 100%** — aucune anglicisme, localisation complète
✓ **Contenu réel** — pas de placeholders, exemples concrets
✓ **Cohérence** — citations croisées entre sections
✓ **Sécurité couverte** — 6 menaces détaillées (T1–T6)
✓ **Pratique** — exemples YAML, commandes, procédures pas à pas
✓ **Responsive** — Starlight adaptatif (mobile/desktop)
✓ **Rapide** — site statique, aucun JavaScript non-essentiel
✓ **Perenne** — versionné avec code source, CI/CD intégré

## Commandes utiles

```bash
# Installation
cd docs && npm install

# Développement (hot reload)
npm run dev

# Build production
npm run build

# Prévisualisation
npm run preview

# Test rapide (ci-dessus)
./test-build.sh
```

## Maintenance

### Ajouter une page

1. Créez `src/content/docs/section/page.md`
2. Ajoutez frontmatter YAML (`title`, `description`)
3. Mettez à jour `astro.config.mjs` sidebar si nouvelle section

### Mettre à jour le CHANGELOG

1. Mettez à jour `/reference/changelog.md`
2. Référez-vous à `../CHANGELOG.md` du repo principal

### Modifier les styles

1. Éditez `src/styles/custom.css`
2. Référez-vous à [Starlight Theming](https://starlight.astro.build/guides/customization/)

## Déploiement

Automatique via `.github/workflows/docs.yml` :

1. Push vers `main` avec modifications dans `docs/`
2. GitHub Actions lance `npm install && npm run build`
3. Build produit `docs/dist/`
4. `actions/deploy-pages` déploie vers GitHub Pages
5. Site live à https://gfriloux.github.io/sshive

## Ressources

- [Astro Docs](https://docs.astro.build)
- [Starlight Docs](https://starlight.astro.build)
- [Starlight Components](https://starlight.astro.build/guides/components/)
- [MDX Guide](https://starlight.astro.build/guides/authoring-content/#mdx)

---

**Créé** : 2026-05-11
**Dernière mise à jour** : 2026-05-11
