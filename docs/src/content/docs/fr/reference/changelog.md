---
title: Changelog
description: Historique des versions et changements de SSHive.
---

Toutes les modifications notables de SSHive sont documentées ici.
Format suivant [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning suivant [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.0] - 2026-05-11

### Added

- **Design system** — palette de couleurs unifiée avec 4 niveaux de profondeur (`BACKGROUND_BASE/PANEL/CARD/ELEVATED`), grille d'espacement (4/8/12/16/20/24/32px), échelle de rayon (4/6/8px), typographie verrouillée (10–20px) ; `TEXT_MONO` pour fingerprints et chemins ; `FONT_BOLD` pour compteurs critiques
- **Top bar** — en-tête pleine largeur couvrant les trois colonnes ; affiche le nom de l'app et la version depuis `CARGO_PKG_VERSION` ; élimine le désalignement vertical entre la marque laterale et les titres de page
- **Sidebar redesign** — icônes Unicode (▣◈◉◎), libellé section "NAVIGATION", barre d'accent 2px à gauche pour élément actif (cohérent avec sélection liste), boutons sans rayon (radius 0), pastille alerte santé (pulse rouge si services Critical), compteur pour Services et SSH Keys, Settings épinglés en bas
- **Service list redesign** — lignes deux lignes (nom service + hostname/URL sous-titre) ; colonne pastille statut (8px, couleur-codée depuis `HealthSnapshot`) ; largeurs de colonne fixes remplaçant `FillPortion` instable — élimine le glitch débordement/superposition à medium window ; en-tête table `BACKGROUND_PANEL`
- **Detail panel max-width** — colonne détail limitée à 720px, alignée à gauche ; arrête la croissance illimitée sur fenêtres larges
- **Settings page** — remplace le placeholder "à venir" ; trois sections : Sécurité (seuil rotation, longueur minimale passphrase), GPG (clef active + bouton Changer, sélecteur backend pinentry), Données (bouton audit log, chemins fichiers config)
- **Animations** — spinner `◐◓◑◒` à 200ms sur Preflight / AutoDeploying / Verifying / Generating ; pulse santé critical à 30Hz via `sin(t)` opacity ; copie feedback fade-in ~100ms ; souscriptions gated sur besoin (app inactif = `Subscription::none()`)
- **GPG setup rework** — titre changé à "Protégez vos données sensibles" ; explication concept deux-phrases avant sélection clef ; bouton "Créer une clef GPG" sur chemin sans-clefs déclenche génération clef Ed25519/Cv25519 en-app ; fallback terminal conservé en option secondaire
- **"Renouveler la clef SSH"** — action anciennement "Faire pivoter la clef" renommée à français plus clair
- **Copy key button visible at rest** — "Copier la clef publique" maintenant avec bordure 1px et background card au repos ; plus invisible jusqu'au hover
- **Empty states** — liste service : bouton CTA "Ajouter mon premier service" avec explication ; clefs SSH : redirection vers Services avec bouton

### Fixed

- Service list column overflow / text overlap at medium window widths — remplacé `FillPortion` fingerprint column par fixed-width two-line name cell
- Header vertical misalignment across the three columns — résolu structurellement par le top bar ; `HEADER_HEIGHT = 64.0` constant pour remaining column sub-headers
- Sidebar border rendered on all 4 sides — remplacé par `rule::vertical` separator unique

### Changed

- `Config` gagne `security: SecurityConfig` (`min_passphrase_len: u8`, défaut 12) — `#[serde(default)]`, backward-compatible
- `GpgConfig` gagne `pinentry_backend: Option<String>` — `#[serde(default)]`, backward-compatible
- Deploy mode cards : "Utilise : ssh-copy-id" et "Utile si : bastion…" supprimés ; réécrits en langage benefit clair
- "Référence dans les secrets : {token_ref}" supprimé du step 2 formulaire service

### Compatibility

Forward et backward compatible avec tous les fichiers config précédents. Tous les nouveaux champs utilisent `#[serde(default)]`.

## [0.3.0] - 2026-05-11

### Added

- **Pure-Rust SSH key re-encryption** — crate `ssh-key` (bcrypt-pbkdf + AES-256-CTR) remplace `ssh-keygen -N` ; passphrase jamais dans les arguments du processus ou `/proc/<pid>/cmdline`
- **GitHub API connector** — déploie, révoque, et vérifie clefs ed25519 via `POST/DELETE/GET /user/keys` ; détection doublon retourne `ApiKeyAlreadyPresent` au lieu d'erreur
- **GitLab.com and self-hosted API connector** — mêmes opérations via `/api/v4/user/keys` avec en-tête `Private-Token` ; URL base self-hosted configurable
- **Post-deploy verification** — après déploiement automatique le déployeur se reconnecte via SSH/API pour confirmer la clef acceptée ; résultat affiché dans écran succès déploiement
- **Pre-flight rotation check** — avant rotation clef sur services API : validation format token local (vérif préfixe `ghp_`/`github_pat_`/`glpat-`), puis sonde `GET /user` pour confirmer autorisation ; modal `DeployBlocker` affiché avant génération clef ; "Configurer le token" redirige directement à step 2 edit service
- **Key revocation UI** — detail panel liste toutes clefs précédentes pour un service avec bouton Revoke ; appelle API ou SSH revocation path approprié
- **Health/Diagnostic view** — table page complète montrant âge clef, statut protection, token API absent, déploiement en attente, alertes rotation ; accessible depuis sidebar
- **`NoApiToken` health reason** — services GitHub/GitLab sans token API configuré apparaissent Warning ; escaladé Critical quand combiné avec `RotationOverdue` ; supprimé quand vault GPG verrouillé pour éviter faux positifs
- **`HealthReason::HardwareKeyHandleNotBackedUp`** — raison santé Info-level pour clefs sk-ed25519 où `backup_prompted` est false ; label : "Fichier clef YubiKey à sauvegarder"
- **SK (YubiKey) rotation safety** — écran avertissement pre-rotation ("YubiKey doit rester branchée pendant toute l'opération") ; vérification obligatoire pour clefs sk-ed25519 (verify failed → error state, révocation bloquée) ; échec verify sur ed25519 standard produit `Success { verified: false }`
- **SK key handle backup prompt** — après génération clef sk-ed25519, detail panel affiche avertissement persistant avec chemin fichier et bouton "Compris — j'ai sauvegardé" qui persiste `backup_prompted: true` à config
- **ExternalCm deploy mode** — nouveau `DeployMode::ExternalCm` pour services dont `authorized_keys` gérés par NixOS, Ansible, Puppet, similaires ; affiche clef publique à copier bloc monospace avec instructions step-by-step ; skip `ssh-copy-id` et post-deploy SSH verification ; badge `EXT` dans liste service ; configurable via service edit step 2 (services SSH générique seulement)
- **Deploy mode indicator in service detail** — section CONNEXION affiche ligne "Déploiement" : `Automatique (ssh-copy-id)`, `Guidé (commande à copier)`, ou `Géré externalement (NixOS, Ansible…)` ; visible même si host/user non configurés
- **Token guide in-app** — step 2 formulaire service affiche URL et scope requis création token API (GitHub: `admin:public_key` ; GitLab: `api`) ; "Ouvrir dans le navigateur" lance `xdg-open` ; "Copier l'URL" copie presse-papiers fallback
- **Cancel button in service form** — bouton "Annuler" sur trois wizard steps ; ferme immédiatement form propre ; row confirmation inline quand form modifié
- **Scrollable service detail panel** — entire service detail panel enveloppé container scrollable ; "attach existing key" list limité 240px ; text filter au-dessus list quand > 8 clefs présentes
- **Copy public key to clipboard** — bouton "Copier la clef publique" header detail clef et fingerprint row service detail ; label change "✓ Copié" 2 secondes puis revient
- **Local audit log** — append-only `~/.config/sshive/audit.log` (chmod 0600) enregistrant génération clef, révocation, suppression service avec ISO timestamp
- **"Faire pivoter la clef" disabled for externally-managed private keys** — quand `private_path` est `None` (clef gérée sops, age, similaires), bouton rotation grisé avec message explicatif
- **`ApiToken(SecretString)`** — wraps tokens sensibles ; `Debug` affiche `ApiToken(***)` seulement ; backed par crate `secrecy`
- **`mlock` on `Passphrase`** — private 256-byte pinned buffer ; locked avec `libc::mlock`, zeroized on `Drop`
- **`HttpClient` trait** — injectable pour tests ; `ReqwestHttpClient` (rustls + webpki roots, no native TLS) ; `FakeHttpClient` pour unit tests
- **`docs/CRYPTO.md`** — politique cryptographique (algorithmes, longueurs clef, cipher modes, RNG, mlock)
- **`docs/THREAT_MODEL.md`** — 6 scénarios menaces T1–T6 avec mitigations et risques résiduels acceptés
- **Regression suites** — `regression_v020.rs` (8 tests) et `regression_v030.rs` (10 tests) couvrant config round-trip, stabilité UUID clef, health computation, backward compatibility tous nouveaux champs

### Fixed

- **`active_key` silently wiped on service edit** — `SubmitServiceForm` merge maintenant champs form-controlled seulement dans service existant, préservant `active_key`, `pending_key`, `created_at`, `last_rotation`, `deployments`
- **`token_ref` orphaned on service edit** — édition service GitHub/GitLab sans retype token ne cache plus `token_ref` ; référence existante préservée
- **GitLab 400 response** — body est objet nested (`{"message": {"fingerprint": [...]}}`); maintenant correctement détecté `ApiKeyAlreadyPresent` sans unwrap panic

### Changed

- `DeployStep::Success` porte maintenant `verified: bool` ; UI distingue "✓ Connexion vérifiée" de "⚠ Vérification non effectuée"
- `DeployMode` gagne troisième variant `ExternalCm` (`"external-cm"` YAML) ; backward-compatible (`#[serde(default)]` = `Automatic`)
- `SshKey` gagne `backup_prompted: bool` (`#[serde(default)]`) ; backward-compatible
- `HealthSnapshot::compute` accepte maintenant `Option<&Secrets>` ; `None` supprime vérifications `NoApiToken` quand vault GPG verrouillé
- `compute_service_health` accepte `has_api_token: Option<bool>`
- Sidebar gagne Health navigation item
- Service config model gagne `deployments: Vec<Deployment>` et `Config.health` (`rotation_warning_days`, défaut 90)

### Removed

- **`ServiceType::Manual`** — supprimé ; configs avec `service_type: manual` auto-migrées vers `ssh-generic` via alias serde ; pas perte donnée, pas migration manuelle

### Security

- Passphrase jamais passée via `-N` à `ssh-keygen` — pure Rust re-encryption via crate `ssh-key` ferme limitation connue v0.2.0
- Toutes connexions HTTP utilisent rustls + WebPKI roots ; pas OpenSSL, pas native TLS, pas system certificate store
- Tokens API stockés `secrets.yaml.gpg` et exposés seulement via `ApiToken::expose()` à call sites ; jamais loggés
- Correction scope token GitHub à `admin:public_key` (inclut `read:public_key` nécessaire pour vérification)

### Compatibility

Forward et backward compatible avec fichiers config v0.2.0. Tous les nouveaux champs utilisent `#[serde(default)]`. L'alias `service_type: manual` assure zéro-friction migration pour configs existantes.

## [0.2.0] - 2026-05-10

### Added

- **Service CRUD** — créer, éditer et supprimer services depuis UI ; sauvegardés atomiquement à `config.yaml`
- **SSH key generation** — ed25519 et sk-ed25519 (YubiKey/FIDO2) ; passphrase obligatoire ≥ 12 chars, collectée via pinentry
- **Deploy flow** — automatique (`ssh-copy-id`) et guidé (affiche commande à copier/coller) modes
- **Key assignment** — attache clef existante `~/.ssh/*.pub` service directement depuis detail panel
- **GPG setup** — first-launch wizard pour sélectionner ou créer clef GPG pour chiffrement secrets
- **Unprotected key detection** — détecte clefs SSH privées sans passphrase et offre en ajouter une via pinentry
- **Pinentry integration** — collecte passphrase via `pinentry-gtk-2`, `pinentry-gnome3`, ou `pinentry-qt` (protocole Assuan) ; backend auto-sélectionné depuis `XDG_CURRENT_DESKTOP` ou `SSHIVE_PINENTRY` override
- **Process hardening** — `PR_SET_DUMPABLE` + `PR_SET_PTRACER` au démarrage pour bloquer core dumps et ptrace (Linux)
- **3-column layout** — sidebar, list (services ou clefs), detail/wizard panel
- **Key detail panel** — fingerprint, type, badge YubiKey, utilisation par service, avertissement sécurité si non-protégée

### Changed

- `active_key` sur `Service` référence maintenant `SshKey` par UUID stable (était string fingerprint en v0.1.0) ; UUIDs stabilisés across restarts via fingerprint matching à scan time
- SSH key scanner stocke maintenant `public_path` sur clef découverte
- Passphrase requise (≥ 12 chars) pour génération clef ed25519 et sk-ed25519

### Security

- Tous `config.yaml` et secrets writes atomiques (tmp + chmod 0600 + rename)
- `prctl(PR_SET_DUMPABLE, 0)` empêche core dumps pouvant exposer secrets
- Toutes subprocess inputs validées avant exécution (hostname, username, port)
- Pas subprocess spawné via `sh -c` — arguments toujours passés tokens discrets
- Protection fichier sk-ed25519 détectée via binary header parsing (pas subprocess, pas hardware interaction)
- Limitation connue : passphrase passée via `-N` à `ssh-keygen` (brièvement visible `/proc/<pid>/cmdline` à processes same-UID) ; planifié PTY ou library-based replacement v0.3.0

### Fixed

- Attachement clef à service persiste maintenant across restarts (UUID stabilisé par fingerprint)
- Clefs sk-ed25519 correctement détectées non-protégées quand hardware absent
- Précédemment silent subprocess failures surfacent maintenant erreurs visibles UI

## [0.1.0] - 2026-05-10

### Added

- Load `~/.config/sshive/config.yaml` au startup (créé empty si absent)
- Scan `~/.ssh/*.pub` clefs SSH publiques locales (ed25519 et sk-ed25519)
- Service list view : nom, badge type, fingerprint, rotation age, badges YubiKey/shared key
- SSH Keys view : fingerprint, type clef, indicator YubiKey, comment, count utilisation
- Dark mode UI avec Inter Variable et JetBrains Mono fonts
- Sidebar navigation (Services, SSH Keys, Settings placeholder)
- File permission check sur `config.yaml` (alerte si readable par autres utilisateurs)
- Symlink detection dans `~/.ssh/` — ignorés silencieusement
- 1 MB size limit sur `config.yaml` avant parsing
- `#![forbid(unsafe_code)]` throughout
