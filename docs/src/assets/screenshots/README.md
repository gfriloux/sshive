# Screenshots attendus

Ce dossier accueille les captures d'écran de l'interface SSHive intégrées dans la documentation.

Pour générer les captures, lance l'application en mode développement :

```sh
just dev
```

## Liste des fichiers attendus

| Fichier | Description |
|---|---|
| `services-list.png` | Vue principale — liste des services avec pastilles de santé |
| `detail-panel.png` | Panneau de détail d'un service (clef, déploiements, historique) |
| `health-page.png` | Page Santé — vue diagnostique complète avec alertes |
| `deploy-modal.png` | Modal de déploiement — modes automatique, guidé, externe |
| `add-service-modal.png` | Modal d'ajout d'un service — formulaire multi-étapes |

## Conventions

- Format PNG, fond natif de l'application (pas de fond blanc forcé)
- Résolution : 1280 × 800 minimum
- Données fictives ou anonymisées (pas de tokens réels, pas d'IPs internes)
- Nom de fichier en kebab-case, exactement comme indiqué dans le tableau ci-dessus
