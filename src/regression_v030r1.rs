/// Régression v0.3.0 → v0.3.0_r1 — vérifie que les configs v0.2.0/v0.3.0 (sans les
/// nouveaux champs r1) se chargent correctement et que les valeurs par défaut s'appliquent.
#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use chrono::NaiveDate;
  use tempfile::TempDir;
  use uuid::Uuid;

  use crate::config::loader::load_from_path;
  use crate::config::model::{Config, DeployMode, KeyType, ServiceParams, ServiceType, SshKey};
  use crate::config::writer::save_to_path;
  use crate::health::{HealthLevel, HealthReason, HealthSnapshot};
  use crate::secrets::model::Secrets;

  fn write_yaml(dir: &TempDir, yaml: &str) -> std::path::PathBuf {
    let path = dir.path().join("config.yaml");
    std::fs::write(&path, yaml).unwrap();
    path
  }

  fn make_sshkey(id: Uuid, yubikey: bool) -> SshKey {
    SshKey {
      id,
      fingerprint: format!("SHA256:test{}", id),
      key_type: if yubikey {
        KeyType::SkEd25519
      } else {
        KeyType::Ed25519
      },
      yubikey,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      comment: "test".into(),
      private_path: None,
      public_path: None,
      service_id: None,
      backup_prompted: false,
    }
  }

  // ── backup_prompted — rétrocompatibilité ──────────────────────────────────

  #[tokio::test]
  async fn config_v030_sans_backup_prompted_charge_false_par_defaut() {
    let yaml = r#"
services: []
keys:
  - id: "550e8400-e29b-41d4-a716-446655440001"
    fingerprint: "SHA256:AbCd"
    key_type: sk-ed25519
    yubikey: true
    created_at: "2026-01-01"
    comment: "yubikey@host"
    service_id: null
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert!(
      !config.keys[0].backup_prompted,
      "backup_prompted doit être false par défaut"
    );
  }

  #[tokio::test]
  async fn backup_prompted_true_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let mut config = Config::default();
    config.keys.push(SshKey {
      id: Uuid::new_v4(),
      fingerprint: "SHA256:XyZw".into(),
      key_type: KeyType::SkEd25519,
      yubikey: true,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      comment: "yk".into(),
      private_path: None,
      public_path: None,
      service_id: None,
      backup_prompted: true,
    });
    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();
    assert!(
      loaded.keys[0].backup_prompted,
      "backup_prompted doit survivre au round-trip"
    );
  }

  // ── DeployMode::ExternalCm — rétrocompatibilité ───────────────────────────

  #[tokio::test]
  async fn config_v030_sans_deploy_mode_charge_automatic() {
    let yaml = r#"
services:
  - name: "Prod"
    service_type: ssh-generic
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
keys: []
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert_eq!(config.services[0].deploy_mode, DeployMode::Automatic);
  }

  #[tokio::test]
  async fn external_cm_round_trip() {
    let yaml = r#"
services:
  - name: "NixOS"
    service_type: manual
    deploy_mode: external-cm
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
keys: []
"#;
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, yaml);
    let config = load_from_path(&path).await.unwrap();
    assert_eq!(config.services[0].deploy_mode, DeployMode::ExternalCm);

    // Round-trip disk
    save_to_path(&config, &path).await.unwrap();
    let reloaded = load_from_path(&path).await.unwrap();
    assert_eq!(reloaded.services[0].deploy_mode, DeployMode::ExternalCm);
  }

  // ── HealthReason::NoApiToken — comportement ──────────────────────────────

  #[test]
  fn health_github_sans_token_produit_warning() {
    let mut config = Config::default();
    let key_id = Uuid::new_v4();
    config.services.push(crate::config::model::Service {
      id: Uuid::new_v4(),
      name: "GitHub perso".into(),
      service_type: ServiceType::GitHub,
      params: ServiceParams {
        token_ref: Some("github_perso".into()),
        ..ServiceParams::default()
      },
      active_key: Some(key_id),
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    });
    // Secrets vides → token absent
    let secrets = Secrets::default();
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), Some(&secrets));
    let health = snap.services.values().next().unwrap();
    assert_eq!(health.level, HealthLevel::Warning);
    assert!(health.reasons.contains(&HealthReason::NoApiToken));
  }

  #[test]
  fn health_vault_locked_no_api_token_supprime() {
    let mut config = Config::default();
    config.services.push(crate::config::model::Service {
      id: Uuid::new_v4(),
      name: "GitHub perso".into(),
      service_type: ServiceType::GitHub,
      params: ServiceParams {
        token_ref: Some("github_perso".into()),
        ..ServiceParams::default()
      },
      active_key: Some(Uuid::new_v4()),
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    });
    // secrets = None → vault verrouillé
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), None);
    let health = snap.services.values().next().unwrap();
    assert!(!health.reasons.contains(&HealthReason::NoApiToken));
  }

  // ── HardwareKeyHandleNotBackedUp — health ────────────────────────────────

  #[test]
  fn health_sk_non_backed_up_est_info() {
    use crate::config::model::{Service, ServiceParams};
    use crate::health::compute_service_health;

    let key_id = Uuid::new_v4();
    let key = make_sshkey(key_id, true); // yubikey = true, backup_prompted = false
    let svc = Service {
      id: Uuid::new_v4(),
      name: "YubiKey service".into(),
      service_type: ServiceType::SshGeneric,
      params: ServiceParams::default(),
      active_key: Some(key_id),
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    };
    let h = compute_service_health(&svc, &[key], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Info);
    assert!(h
      .reasons
      .iter()
      .any(|r| matches!(r, HealthReason::HardwareKeyHandleNotBackedUp { .. })));
  }

  #[test]
  fn health_sk_backed_up_pas_de_raison() {
    use crate::config::model::{Service, ServiceParams};
    use crate::health::compute_service_health;

    let key_id = Uuid::new_v4();
    let mut key = make_sshkey(key_id, true);
    key.backup_prompted = true;

    let svc = Service {
      id: Uuid::new_v4(),
      name: "YubiKey service".into(),
      service_type: ServiceType::SshGeneric,
      params: ServiceParams::default(),
      active_key: Some(key_id),
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    };
    let h = compute_service_health(&svc, &[key], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Ok);
    assert!(!h
      .reasons
      .iter()
      .any(|r| matches!(r, HealthReason::HardwareKeyHandleNotBackedUp { .. })));
  }
}
