use std::path::{Path, PathBuf};

use tokio::fs;

use crate::config::model::Config;
use crate::error::AppError;

const MAX_CONFIG_SIZE: usize = 1_048_576; // 1 Mo

pub fn config_dir() -> Result<PathBuf, AppError> {
  dirs::config_dir()
    .map(|p| p.join("sshive"))
    .ok_or(AppError::NoConfigDir)
}

pub fn config_path() -> Result<PathBuf, AppError> {
  config_dir().map(|d| d.join("config.yaml"))
}

pub async fn load_or_create() -> Result<Config, AppError> {
  let dir = config_dir()?;
  let path = config_path()?;

  fs::create_dir_all(&dir).await.map_err(|e| AppError::Io {
    path: dir.clone(),
    message: e.to_string(),
  })?;

  #[cfg(unix)]
  set_permissions_dir(&dir);

  load_from_path(&path).await
}

pub async fn load_from_path(path: &Path) -> Result<Config, AppError> {
  match fs::read_to_string(path).await {
    Ok(content) => {
      if content.len() > MAX_CONFIG_SIZE {
        return Err(AppError::Io {
          path: path.to_path_buf(),
          message: "fichier trop volumineux (> 1 Mo)".to_string(),
        });
      }

      #[cfg(unix)]
      warn_permissions(path);

      serde_yaml::from_str::<Config>(&content).map_err(|e| AppError::YamlParse {
        path: path.to_path_buf(),
        message: e.to_string(),
      })
    }

    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
      let empty = Config::default();
      let yaml =
        serde_yaml::to_string(&empty).map_err(|e| AppError::YamlSerialize(e.to_string()))?;

      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| AppError::Io {
          path: parent.to_path_buf(),
          message: e.to_string(),
        })?;
      }

      fs::write(path, yaml.as_bytes())
        .await
        .map_err(|e| AppError::Io {
          path: path.to_path_buf(),
          message: e.to_string(),
        })?;

      #[cfg(unix)]
      set_permissions_file(path);

      Ok(empty)
    }

    Err(e) => Err(AppError::Io {
      path: path.to_path_buf(),
      message: e.to_string(),
    }),
  }
}

#[cfg(unix)]
fn warn_permissions(path: &Path) {
  use std::os::unix::fs::PermissionsExt;
  if let Ok(meta) = std::fs::metadata(path) {
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
      tracing::warn!(
        "{} est lisible par d'autres utilisateurs (mode {:04o}) — recommandé : chmod 600",
        path.display(),
        mode
      );
    }
  }
}

#[cfg(unix)]
fn set_permissions_file(path: &Path) {
  use std::os::unix::fs::PermissionsExt;
  let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(unix)]
fn set_permissions_dir(path: &Path) {
  use std::os::unix::fs::PermissionsExt;
  let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tempfile::TempDir;

  use super::*;

  #[tokio::test]
  async fn charge_config_valide() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    fs::write(&path, "services: []\nkeys: []\n").unwrap();
    assert!(load_from_path(&path).await.is_ok());
  }

  #[tokio::test]
  async fn cree_fichier_si_absent() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    assert!(!path.exists());
    let result = load_from_path(&path).await;
    assert!(result.is_ok());
    assert!(path.exists());
    assert!(result.unwrap().services.is_empty());
  }

  #[tokio::test]
  async fn yaml_invalide_retourne_erreur_typee() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    fs::write(&path, "{ this is: [not valid yaml}}}").unwrap();
    assert!(matches!(
      load_from_path(&path).await,
      Err(AppError::YamlParse { .. })
    ));
  }

  #[tokio::test]
  async fn fichier_vide_retourne_config_defaut() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    fs::write(&path, "").unwrap();
    let result = load_from_path(&path).await;
    assert!(result.is_ok());
    assert!(result.unwrap().services.is_empty());
  }

  #[tokio::test]
  async fn fichier_trop_grand_retourne_erreur() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    fs::write(&path, "x".repeat(1_048_577)).unwrap();
    assert!(matches!(
      load_from_path(&path).await,
      Err(AppError::Io { .. })
    ));
  }

  #[tokio::test]
  #[cfg(unix)]
  async fn permissions_insuffisantes_retourne_erreur() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    fs::write(&path, "services: []\nkeys: []\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let result = load_from_path(&path).await;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(result, Err(AppError::Io { .. })));
  }

  #[tokio::test]
  async fn cree_repertoire_parent_si_absent() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sous/repertoire/config.yaml");
    assert!(!path.exists());
    let result = load_from_path(&path).await;
    assert!(result.is_ok());
    assert!(path.exists());
  }
}
