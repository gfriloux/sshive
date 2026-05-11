/// Régression v0.2.0 → v0.3.0 — vérifie que les configs v0.2.0 (sans les nouveaux
/// champs v0.3.0) se chargent correctement et que les valeurs par défaut s'appliquent.
#[cfg(test)]
mod tests {
  use chrono::NaiveDate;
  use tempfile::TempDir;
  use uuid::Uuid;

  use crate::config::loader::load_from_path;
  use crate::config::model::{Config, DeployMode, Deployment, ServiceType};
  use crate::config::writer::save_to_path;

  fn write_yaml(dir: &TempDir, yaml: &str) -> std::path::PathBuf {
    let path = dir.path().join("config.yaml");
    std::fs::write(&path, yaml).unwrap();
    path
  }

  // ── Rétrocompatibilité modèle v0.2.0 ─────────────────────────────

  #[tokio::test]
  async fn config_v020_sans_deployments_charge_avec_vecteur_vide() {
    let yaml = r#"
services:
  - id: "550e8400-e29b-41d4-a716-446655440010"
    name: "GitHub perso"
    service_type: github
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
    deploy_mode: automatic
keys: []
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert!(
      config.services[0].deployments.is_empty(),
      "deployments doit être vide pour une config v0.2.0"
    );
  }

  #[tokio::test]
  async fn config_v020_sans_health_config_defaut_90j() {
    let yaml = "services: []\nkeys: []";
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert_eq!(
      config.health.rotation_warning_days, 90,
      "rotation_warning_days doit valoir 90 par défaut"
    );
  }

  #[tokio::test]
  async fn config_v020_health_config_personnalisable() {
    let yaml = "services: []\nkeys: []\nhealth:\n  rotation_warning_days: 30";
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert_eq!(config.health.rotation_warning_days, 30);
  }

  // ── Round-trip Deployment ─────────────────────────────────────────

  #[tokio::test]
  async fn deployment_remote_ref_preservee_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let key_id = Uuid::new_v4();

    let mut config = Config::default();
    config.services.push(crate::config::model::Service {
      id: Uuid::new_v4(),
      name: "GitLab CI".into(),
      service_type: ServiceType::GitLab,
      params: Default::default(),
      active_key: Some(key_id),
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
      deploy_mode: DeployMode::Automatic,
      deployments: vec![Deployment {
        key_id,
        deployed_at: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
        remote_ref: Some("12345678".into()),
        last_verified: Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
      }],
    });

    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();

    assert_eq!(loaded.services[0].deployments.len(), 1);
    let dep = &loaded.services[0].deployments[0];
    assert_eq!(dep.remote_ref.as_deref(), Some("12345678"));
    assert_eq!(dep.key_id, key_id);
    assert!(dep.last_verified.is_some());
  }

  #[tokio::test]
  async fn deployment_sans_remote_ref_accepte() {
    let yaml = r#"
services:
  - id: "660e8400-e29b-41d4-a716-446655440011"
    name: "Prod SSH"
    service_type: ssh-generic
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
    deployments:
      - key_id: "770e8400-e29b-41d4-a716-446655440012"
        deployed_at: "2026-05-10"
keys: []
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert!(config.services[0].deployments[0].remote_ref.is_none());
    assert!(config.services[0].deployments[0].last_verified.is_none());
  }

  // ── Compatibilité token_ref ────────────────────────────────────────

  #[tokio::test]
  async fn config_v020_avec_token_ref_charge_correctement() {
    let yaml = r#"
services:
  - id: "880e8400-e29b-41d4-a716-446655440013"
    name: "GitHub pro"
    service_type: github
    params:
      token_ref: "github_pro"
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
keys: []
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    assert_eq!(
      config.services[0].params.token_ref.as_deref(),
      Some("github_pro")
    );
  }

  // ── Invariants clefs v0.2.0 ────────────────────────────────────────

  #[tokio::test]
  async fn cles_v020_sans_nouveaux_champs_chargees_correctement() {
    let yaml = r#"
services: []
keys:
  - fingerprint: "SHA256:AbCdEfGhIjKlMnOpQrStUvWxYz01234567890abc"
    key_type: ed25519
    yubikey: false
    created_at: "2026-01-01"
    comment: "sshive/github/2026-01-01"
"#;
    let dir = TempDir::new().unwrap();
    let config = load_from_path(&write_yaml(&dir, yaml)).await.unwrap();
    let key = &config.keys[0];
    assert!(key.private_path.is_none());
    assert!(key.public_path.is_none());
    assert!(key.service_id.is_none());
    // UUID généré automatiquement
    assert_ne!(key.id, Uuid::nil());
  }

  // ── HealthSnapshot depuis config v0.2.0 ───────────────────────────

  #[test]
  fn health_snapshot_depuis_config_v020_ne_panique_pas() {
    use crate::health::HealthSnapshot;
    use std::collections::HashMap;

    let mut config = Config::default();
    config.services.push(crate::config::model::Service {
      id: Uuid::new_v4(),
      name: "Test".into(),
      service_type: ServiceType::SshGeneric,
      params: Default::default(),
      active_key: None,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    });

    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), None);
    assert_eq!(snap.services.len(), 1);
  }
}
