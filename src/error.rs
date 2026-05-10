use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppError {
  #[error("Impossible de trouver ~/.config")]
  NoConfigDir,

  #[error("Impossible de trouver ~")]
  NoHomeDir,

  #[error("Erreur I/O sur {path} : {message}")]
  Io { path: PathBuf, message: String },

  #[error("YAML invalide dans {path} : {message}")]
  YamlParse { path: PathBuf, message: String },

  #[error("Erreur de sérialisation YAML : {0}")]
  YamlSerialize(String),

  #[error("Erreur glob : {0}")]
  Glob(String),
}
