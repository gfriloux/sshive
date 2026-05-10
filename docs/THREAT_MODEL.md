# Threat Model — SSHive

## Contexte et hypothèses

**Cible :** desktop personnel Linux mono-utilisateur.

**Hypothèses :**
- L'OS et le kernel sont de confiance (pas d'attaquant root local)
- LUKS recommandé sur la partition root (non requis)
- L'utilisateur est le seul propriétaire de la session
- SSHive est lancé depuis une session utilisateur normale (pas root)

**Hors scope :**
- Attaquant disposant des droits root
- Attaque physique avec accès matériel (cold boot, bus sniffing)
- Compromission du kernel Linux
- Environnement multi-utilisateurs
- Side-channels électromagnétiques ou temporels
- Sécurité des serveurs distants gérés par SSHive

---

## T1 — Vol ou copie du disque à froid

**Vecteur :** laptop volé éteint, disque cloné, backup non chiffré.

**Impact potentiel :** accès aux clefs SSH privées et aux tokens API.

**Mitigations SSHive :**
- Clefs SSH privées chiffrées avec passphrase (bcrypt-pbkdf + AES-256-CTR)
- Tokens API chiffrés dans `secrets.yaml.gpg` (GPG)
- `config.yaml` : pas de secret, seulement des métadonnées (0600)

**Mitigations recommandées (hors SSHive) :**
- Chiffrement intégral du disque (LUKS)
- Passphrase forte sur les clefs SSH (≥ 12 caractères, imposé par SSHive)

**Risque résiduel :** passphrase faible cassable hors-ligne. bcrypt-pbkdf ralentit les attaques par dictionnaire mais ne les bloque pas.

---

## T2 — Processus malveillant du même UID

**Vecteur :** application compromise tournant sous le même utilisateur (browser, extension, dépendance compromise, script malveillant).

**Impact potentiel :** lecture des fichiers `~/.ssh/*`, lecture de `config.yaml`, ptrace de SSHive.

**Mitigations SSHive :**
- `PR_SET_DUMPABLE 0` au démarrage → bloque `ptrace`, `/proc/<pid>/mem`, core dumps lisibles
- `PR_SET_PTRACER 0` → Yama LSM (si activé sur le système)
- Clefs SSH privées chiffrées → un processus qui lit le fichier obtient du chiffré
- Aucune passphrase dans les arguments de processus (`ssh-key` crate depuis v0.3.0)

**Risque résiduel :** un processus même-UID peut lire les fichiers `~/.ssh/*` (permissions 0600 ne protègent que d'autres utilisateurs). Si `gpg-agent` est en cache, les secrets GPG sont accessibles via l'agent. Ce risque est inhérent à l'architecture Unix mono-utilisateur.

---

## T3 — Fuite via swap ou hibernation

**Vecteur :** passphrase ou token en RAM swappée sur disque non chiffré, ou image d'hibernation en clair.

**Impact potentiel :** récupération de secrets en clair après redémarrage ou depuis un disque cloné.

**Mitigations SSHive (v0.3.0) :**
- `zeroize` au Drop sur `Passphrase` et buffers sensibles
- `mlock` sur `Passphrase` (pages verrouillées en RAM, non swappables)
- `PR_SET_DUMPABLE 0` → core dumps non accessibles

**Risque résiduel :**
- L'image d'hibernation (`/var/lib/systemd/sleep/hibernate.img`) contient la RAM entière — si elle n'est pas chiffrée séparément, les pages mlockées de SSHive y apparaissent
- Seul `mlock` protect les pages actives, pas l'hibernation

**Mitigations recommandées (hors SSHive) :**
- Swap chiffré (LUKS ou `zram` avec chiffrement)
- Hibernation chiffrée (kernel `swsusp` avec LUKS)
- `vm.swappiness=0` si performance le permet

---

## T4 — Fuite via logs ou observabilité

**Vecteur :** `tracing`, `journald`, fichiers de logs capturés par un backup cloud ou lus par un processus tiers.

**Impact potentiel :** token API ou passphrase apparaissant en clair dans les logs.

**Mitigations SSHive :**
- `ApiToken` : type wrapper avec `Debug` obfusqué (`ApiToken(***)`)
- `#[tracing::instrument(skip(token))]` sur toutes les fonctions sensibles
- `Passphrase` : pas de `Debug` ou `Display`
- `reqwest` : header `Authorization` jamais passé à `tracing::info!`
- Aucun subprocess ne reçoit de secret en argv depuis v0.3.0 (ssh-key crate)

**Risque résiduel :** bug humain dans une future contribution. À couvrir par revue de code et test de non-régression.

---

## T5 — MITM sur les API GitHub/GitLab

**Vecteur :** proxy malveillant, autorité de certification compromise, réseau hostile (réseau public, MITM corporate).

**Impact potentiel :** vol du token API Bearer en transit, injection d'une fausse clef SSH sur le compte.

**Mitigations SSHive :**
- TLS 1.2+ avec `rustls` et `webpki-roots` (CA Mozilla)
- Hostname verification stricte (par défaut `rustls`)
- Pas de redirection sur les endpoints d'authentification (`Policy::none()`)
- Warning visible si `HTTP_PROXY` ou `HTTPS_PROXY` est défini

**Risque résiduel :**
- Compromission d'une autorité de certification publique (rare, hors scope desktop perso)
- Proxy corporate MITM légitime : SSHive avertit mais n'empêche pas

---

## T6 — Manipulation des fichiers de configuration

**Vecteur :** processus malveillant même-UID modifiant `config.yaml` ou `secrets.yaml.gpg` pour rediriger des opérations SSH vers un serveur attaquant.

**Impact potentiel :** déploiement d'une clef SSH vers un serveur non prévu, révocation d'une clef légitime.

**Mitigations SSHive :**
- `config.yaml` : permissions 0600 (lecture/écriture par owner uniquement)
- `secrets.yaml.gpg` : chiffré GPG avec AEAD-like MDC — toute modification détectée au déchiffrement
- Validation stricte des inputs (hostname, username, port) avant tout subprocess
- Écriture atomique (tmp + rename) — pas de fichier partiellement écrit exploitable

**Risque résiduel :** `config.yaml` n'est pas signé — si un attaquant peut le modifier (T2), il peut aussi remplacer les paramètres de connexion. La signature de `config.yaml` est hors scope v0.3.0.

---

## Risques résiduels documentés

| Risque | Catégorie | Prévu pour |
|--------|-----------|------------|
| `config.yaml` non signé | T6 | v0.4.0 si pertinent |
| Hibernation non couverte par mlock | T3 | Recommandation utilisateur |
| Proxy MITM corporate | T5 | Hors scope |
| Passphrase via `-N` (génération sk-ed25519 uniquement) | T2 | Évaluation v0.4.0 |

---

## Hors scope explicite

Ces menaces ne sont pas dans le modèle de menace de SSHive :

- **Attaquant root local** : peut lire tous les fichiers, modifier le kernel, contourner `PR_SET_DUMPABLE`
- **Attaque physique avec accès matériel** : cold boot attack, bus sniffing, analyse électromagnétique
- **Compromission du kernel** : rootkit, module malveillant
- **Sécurité des serveurs distants** : SSHive gère les clefs côté client uniquement
- **Multi-utilisateurs** : SSHive est un outil personnel mono-utilisateur
- **Side-channels** : timing, cache, consommation électrique
