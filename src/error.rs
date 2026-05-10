use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppError {
  // ── Fichiers et chemins ────────────────────────────────────────
  #[error("Impossible de trouver ~/.config")]
  NoConfigDir,

  #[error("Impossible de trouver ~")]
  NoHomeDir,

  #[error("Erreur I/O sur {path} : {message}")]
  Io { path: PathBuf, message: String },

  #[error("Chemin invalide : {0}")]
  InvalidPath(PathBuf),

  // ── YAML ───────────────────────────────────────────────────────
  #[error("YAML invalide dans {path} : {message}")]
  YamlParse { path: PathBuf, message: String },

  #[error("Erreur de sérialisation YAML : {0}")]
  YamlSerialize(String),

  // ── Glob ───────────────────────────────────────────────────────
  #[error("Erreur glob : {0}")]
  Glob(String),

  // ── Subprocesses ───────────────────────────────────────────────
  #[error("Impossible de lancer {program} : {message}")]
  SubprocessSpawn { program: String, message: String },

  #[error("Timeout ({timeout_secs}s) dépassé pour {program}")]
  SubprocessTimeout { program: String, timeout_secs: u64 },

  #[error("Erreur I/O lors de l'exécution de {program} : {message}")]
  SubprocessIo { program: String, message: String },

  #[error("{program} a échoué (code {exit_code}) : {stderr}")]
  SubprocessFailed {
    program: String,
    exit_code: i32,
    stderr: String,
  },

  #[error("Binaire introuvable : {0} — vérifiez que le programme est installé")]
  BinaryNotFound(String),

  // ── Clefs SSH ──────────────────────────────────────────────────
  #[error("La clef existe déjà : {path}")]
  KeyAlreadyExists { path: PathBuf },

  #[error("Aucune clef physique (YubiKey) détectée")]
  YubiKeyNotPresent,

  #[error("La clef est déjà protégée par une phrase secrète")]
  KeyAlreadyProtected,

  // ── API Forge (GitHub, GitLab) ─────────────────────────────────
  #[error("API — token invalide pour le service {service}")]
  ApiUnauthorized { service: String },

  #[error("API — clef déjà présente sur le service {service}")]
  ApiKeyAlreadyPresent { service: String },

  #[error("API — erreur {status} sur {service} : {message}")]
  ApiError {
    service: String,
    status: u16,
    message: String,
  },

  // ── Pinentry ───────────────────────────────────────────────────
  #[error("pinentry — opération annulée par l'utilisateur")]
  PinentryUserCancelled,

  #[error("pinentry — erreur de protocole : {0}")]
  PinentryProtocolError(String),

  #[error("pinentry — aucun backend disponible (installez pinentry-gtk-2 ou pinentry-gnome3)")]
  PinentryBackendUnavailable,

  // ── GPG ────────────────────────────────────────────────────────
  #[error("GPG — opération '{operation}' échouée : {stderr}")]
  GpgFailed { operation: String, stderr: String },

  #[error("GPG — aucune clef secrète disponible")]
  GpgNoSecretKey,

  #[error("GPG — déchiffrement impossible (mauvaise clef ?)")]
  GpgDecryptFailed,

  #[error("GPG — agent non disponible (lancez gpg-agent)")]
  GpgAgentUnavailable,

  // ── Validation ─────────────────────────────────────────────────
  #[error("Valeur invalide pour {field} : {reason}")]
  Validation { field: String, reason: String },

  // ── Parsing ────────────────────────────────────────────────────
  #[error("Impossible de parser {context} : {raw}")]
  ParseError { context: String, raw: String },
}
