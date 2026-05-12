#![allow(dead_code)]
pub mod model;

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::config::loader::config_dir;
use crate::error::AppError;
use crate::secrets::model::Secrets;
use crate::subprocess::gpg;

pub fn secrets_path_default() -> Result<PathBuf, AppError> {
  config_dir().map(|d| d.join("secrets.yaml.gpg"))
}

/// Version owned pour Task::perform.
pub async fn load_or_create(gpg_fingerprint: String, path: PathBuf) -> Result<Secrets, AppError> {
  load_or_create_ref(&gpg_fingerprint, &path).await
}

/// Déchiffre et charge les secrets, ou crée un fichier vide chiffré.
pub async fn load_or_create_ref(gpg_fingerprint: &str, path: &Path) -> Result<Secrets, AppError> {
  if path.exists() {
    crate::config::loader::warn_permissions(path);
  }
  if !path.exists() {
    let empty = Secrets::default();
    save_ref(&empty, gpg_fingerprint, path).await?;
    return Ok(empty);
  }

  let plaintext = gpg::decrypt(path).await?;

  if plaintext.is_empty() {
    return Ok(Secrets::default());
  }

  serde_json::from_slice(&plaintext).map_err(|e| AppError::ParseError {
    context: "secrets.yaml.gpg".to_string(),
    raw: e.to_string(),
  })
}

/// Chiffre et sauvegarde les secrets de façon atomique.
/// Pipeline : serde_json → gpg::encrypt(stdin) → .gpg.tmp → rename
/// Jamais de plaintext intermédiaire sur disque.
pub async fn save_ref(
  secrets: &Secrets,
  gpg_fingerprint: &str,
  path: &Path,
) -> Result<(), AppError> {
  let plaintext =
    serde_json::to_vec(secrets).map_err(|e| AppError::YamlSerialize(e.to_string()))?;

  let tmp_path = path.with_extension("yaml.gpg.tmp");

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).await.map_err(|e| AppError::Io {
      path: parent.to_path_buf(),
      message: e.to_string(),
    })?;
  }

  gpg::encrypt(&plaintext, gpg_fingerprint, &tmp_path).await?;

  fs::rename(&tmp_path, path).await.map_err(|e| AppError::Io {
    path: path.to_path_buf(),
    message: e.to_string(),
  })
}
