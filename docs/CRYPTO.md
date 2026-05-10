# Cryptography in SSHive

## Scope

SSHive protects three categories of data:

| Artefact | Protection | Emplacement |
|----------|------------|-------------|
| Clefs SSH privées | Passphrase (bcrypt-pbkdf + AES-256-CTR) | `~/.ssh/sshive_*` |
| Secrets API (tokens) | Chiffrement GPG | `~/.config/sshive/secrets.yaml.gpg` |
| Transit API GitHub/GitLab | TLS 1.2+ | Connexions sortantes uniquement |
| Config services | Aucun (pas de secret) | `~/.config/sshive/config.yaml` (0600) |

---

## Algorithmes SSH

### Clefs générées par SSHive

**ed25519** (défaut recommandé)

- Courbe Edwards25519, définie dans [RFC 8709](https://www.rfc-editor.org/rfc/rfc8709)
- Taille de clef fixe (256 bits) — pas de paramètre faible possible
- Signature rapide et compacte
- Recommandé par l'ANSSI (référentiel clefs 2023) et le NIST SP 800-186

**sk-ed25519** (clef hardware FIDO2)

- Variante matérielle : la clef privée ne quitte jamais le dispositif YubiKey/Nitrokey
- Le fichier sur disque contient uniquement un *credential handle* (référence opaque)
- Chaque opération de signature nécessite la présence physique de l'utilisateur (touch)

### Algorithmes refusés

| Algorithme | Raison |
|-----------|--------|
| DSA | Retiré d'OpenSSH 7.0+ — clef de 1024 bits fixe |
| RSA < 3072 bits | Vulnérable aux attaques sub-exponentielles modernes |
| ECDSA P-256/P-384 | Paramètres NIST contestés, sensible à l'entropie du RNG (RFC 6979) |
| RSA 4096 | Accepté en mode legacy pour compatibilité avec des serveurs anciens, non généré par SSHive |

---

## Chiffrement des clefs privées au repos

**Implémentation :** crate Rust [`ssh-key`](https://crates.io/crates/ssh-key) avec feature `encryption` (v0.3.0+)

**Format :** OpenSSH private key format (openssh-key-v1)

**KDF :** bcrypt-pbkdf avec 16 rounds (paramètre par défaut d'OpenSSH 9.x)

**Chiffrement :** AES-256-CTR

**Passphrase :** collectée via `pinentry` (processus séparé, mémoire isolée) — minimum 12 caractères imposé par SSHive

**Avantages :**
- Aucun subprocess pour les opérations sensibles — la passphrase n'apparaît jamais dans `/proc/<pid>/cmdline`
- Interopérable avec OpenSSH standard
- Bibliothèque auditée (RustCrypto)

**Limitation connue (v0.3.0) :** la passphrase transite brièvement en mémoire Rust avant d'être zéroïsée. Les pages mémoire ne sont pas mlockées (protection contre le swap) — prévu pour v0.4.0.

---

## Stockage des secrets applicatifs

**Format :** JSON sérialisé, chiffré GPG (`secrets.yaml.gpg`)

**Outil :** `gpg2` via subprocess (pas de binding natif)

**Raisons du choix GPG :**
- Présence quasi-universelle sur desktop Linux
- Intégration native avec `gpg-agent` (cache de passphrase, support hardware)
- Support YubiKey/smartcard natif via `scd`
- Aucune dépendance cryptographique supplémentaire à auditer

**Limitations GPG connues :**
- RFC 4880 (OpenPGP) a des quirks de parsing — `gpg2` les gère correctement
- L'algorithme symétrique dépend de la configuration gpg-agent locale (AES-256 par défaut)

---

## TLS sortant (API GitHub/GitLab)

**Implémentation :** `reqwest` avec backend `rustls` + `webpki-roots`

**Version minimale :** TLS 1.2 (TLS 1.3 préféré — par défaut avec rustls 0.23)

**Certificats :** bundle Mozilla WebPKI (mis à jour avec chaque release SSHive)

**Pas de TLS pinning :** GitHub et GitLab font tourner leurs propres rotations de certificats — le pinning créerait des ruptures de service.

**Redirections :** désactivées sur les endpoints d'authentification (`Policy::none()`)

---

## Sources d'aléa

SSHive utilise exclusivement `OsRng` (syscall `getrandom` via la crate `rand_core`) pour toutes les opérations cryptographiques :

- Génération de clefs SSH (`PrivateKey::random`)
- Vecteurs d'initialisation lors du chiffrement

`thread_rng()` n'est jamais utilisé pour du matériel cryptographique.

---

## Politique de rotation

SSHive n'impose pas de rotation forcée, mais recommande :

| Type | Recommandation |
|------|---------------|
| Clefs SSH de service | 12–24 mois (configurable via `health.rotation_warning_days`) |
| Tokens API GitHub/GitLab | Selon la politique du provider (GitHub : 1 an maximum recommandé) |
| Clef GPG de l'utilisateur | Hors du périmètre SSHive |

Le seuil d'avertissement de rotation est configurable dans `config.yaml` :

```yaml
health:
  rotation_warning_days: 90  # défaut
```

---

## Protection de la mémoire

| Mécanisme | Statut | Notes |
|----------|--------|-------|
| `zeroize` au Drop | ✅ v0.2.0 | Struct `Passphrase`, buffers sensibles |
| `PR_SET_DUMPABLE 0` | ✅ v0.2.0 | Bloque core dumps et ptrace même-UID |
| `PR_SET_PTRACER 0` | ✅ v0.2.0 | Yama LSM |
| `mlock` sur Passphrase | ✅ v0.3.0 | `Box<[u8; 256]>` verrouillé en RAM |
| `mlock` complet (mlockall) | ❌ Hors scope | Trop coûteux pour une app GUI |
| Swap chiffré | Recommandé | Responsabilité utilisateur (LUKS) |

---

## Ce que SSHive ne fait PAS

- **Aucun chiffrement custom** — toute la crypto passe par des bibliothèques auditées
- **Aucun KDF maison** — bcrypt-pbkdf fourni par `ssh-key`/OpenSSH
- **Aucun contournement de gpg-agent ou ssh-agent** — SSHive délègue aux agents système
- **Aucune dérivation de master password** — un secret = une clef GPG existante
- **Aucun stockage de passphrase persistant** — zéroïsée après utilisation
