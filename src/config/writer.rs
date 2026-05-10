#![allow(dead_code)]
use std::path::Path;

use tokio::fs;

use crate::config::loader::config_path;
use crate::config::model::Config;
use crate::error::AppError;

pub async fn save_config(config: &Config) -> Result<Config, AppError> {
  let path = config_path()?;
  save_to_path(config, &path).await?;
  Ok(config.clone())
}

/// Version owned pour Task::perform.
pub async fn save_config_owned(config: Config) -> Result<Config, AppError> {
  save_config(&config).await
}

/// Sauvegarde atomique :
/// 1. Sérialise en YAML en mémoire
/// 2. Écrit dans `.yaml.tmp` (même répertoire — rename atomique garanti)
/// 3. chmod 0600
/// 4. rename()
pub async fn save_to_path(config: &Config, path: &Path) -> Result<(), AppError> {
  let yaml = serde_yaml::to_string(config).map_err(|e| AppError::YamlSerialize(e.to_string()))?;

  // CRITIQUE : même répertoire que la destination pour rename atomique (pas /tmp)
  let tmp_path = path.with_extension("yaml.tmp");

  fs::write(&tmp_path, yaml.as_bytes())
    .await
    .map_err(|e| AppError::Io {
      path: tmp_path.clone(),
      message: e.to_string(),
    })?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600)).map_err(|e| {
      AppError::Io {
        path: tmp_path.clone(),
        message: e.to_string(),
      }
    })?;
  }

  fs::rename(&tmp_path, path).await.map_err(|e| AppError::Io {
    path: path.to_path_buf(),
    message: e.to_string(),
  })
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tempfile::TempDir;

  use super::*;

  #[tokio::test]
  async fn sauvegarde_atomique_cree_le_fichier() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    save_to_path(&Config::default(), &path).await.unwrap();
    assert!(path.exists());
  }

  #[tokio::test]
  async fn aucun_fichier_tmp_residuel() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    save_to_path(&Config::default(), &path).await.unwrap();
    assert!(!dir.path().join("config.yaml.tmp").exists());
  }

  #[tokio::test]
  #[cfg(unix)]
  async fn permissions_600_appliquees() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    save_to_path(&Config::default(), &path).await.unwrap();
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(
      mode, 0o600,
      "permissions attendues 0600, obtenues {:04o}",
      mode
    );
  }

  #[tokio::test]
  async fn round_trip_avec_service() {
    use crate::config::loader::load_from_path;
    use crate::config::model::{DeployMode, Service, ServiceType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");

    let mut config = Config::default();
    config.services.push(Service {
      id: Uuid::new_v4(),
      name: "Test".to_string(),
      service_type: ServiceType::Manual,
      params: Default::default(),
      active_key: None,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    });

    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();
    assert_eq!(loaded.services[0].name, "Test");
    assert_eq!(loaded.services[0].id, config.services[0].id);
  }

  #[tokio::test]
  async fn repertoire_inexistant_retourne_erreur_io() {
    let path = std::path::Path::new("/chemin/inexistant/garanti/config.yaml");
    let result = save_to_path(&Config::default(), path).await;
    assert!(matches!(result, Err(AppError::Io { .. })));
  }
}
