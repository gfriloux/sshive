/// Régression v0.1.0 — vérifie que les fonctionnalités read-only (chargement
/// config YAML, scan des clefs SSH, modèle de données) continuent de fonctionner
/// correctement en v0.2.0.
#[cfg(test)]
mod tests {
  use chrono::NaiveDate;
  use tempfile::TempDir;
  use uuid::Uuid;

  use crate::config::loader::load_from_path;
  use crate::config::model::{Config, DeployMode, KeyType, Service, ServiceType, SshKey};
  use crate::config::ssh_scanner::scan_from_dir;
  use crate::config::writer::save_to_path;

  // ── Helpers ─────────────────────────────────────────────────────

  fn write_yaml(dir: &TempDir, yaml: &str) -> std::path::PathBuf {
    let path = dir.path().join("config.yaml");
    std::fs::write(&path, yaml).unwrap();
    path
  }

  fn write_pub(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
  }

  fn make_service(service_type: ServiceType) -> Service {
    Service {
      id: Uuid::new_v4(),
      name: "Test".into(),
      service_type,
      params: Default::default(),
      active_key: None,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    }
  }

  // Clef ed25519 réelle (parseable par ssh_key::PublicKey::from_openssh)
  const ED25519_PUB: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GkZH test@host\n";

  // ── Config loader ────────────────────────────────────────────────

  #[tokio::test]
  async fn config_vide_retourne_defaut() {
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, "{}");
    let config = load_from_path(&path).await.unwrap();
    assert!(config.services.is_empty());
    assert!(config.keys.is_empty());
  }

  #[tokio::test]
  async fn service_github_minimal_charge() {
    let yaml = r#"
services:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    name: "GitHub perso"
    service_type: github
    active_key: null
    created_at: "2026-01-01"
    deploy_mode: automatic
"#;
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, yaml);
    let config = load_from_path(&path).await.unwrap();

    assert_eq!(config.services.len(), 1);
    let svc = &config.services[0];
    assert_eq!(svc.name, "GitHub perso");
    assert_eq!(svc.service_type, ServiceType::GitHub);
    assert!(svc.active_key.is_none());
  }

  #[tokio::test]
  async fn uuid_stable_apres_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let fixed_id: Uuid = "550e8400-e29b-41d4-a716-446655440001".parse().unwrap();

    let mut config = Config::default();
    config.services.push(Service {
      id: fixed_id,
      ..make_service(ServiceType::Manual)
    });

    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();
    assert_eq!(loaded.services[0].id, fixed_id);
  }

  #[tokio::test]
  async fn tous_les_types_de_service_round_trip() {
    let types = [
      ServiceType::GitHub,
      ServiceType::GitLab,
      ServiceType::GitLabSelfHosted,
      ServiceType::SshGeneric,
      ServiceType::Manual,
    ];
    for st in types {
      let dir = TempDir::new().unwrap();
      let path = dir.path().join("config.yaml");
      let mut config = Config::default();
      config.services.push(make_service(st.clone()));
      save_to_path(&config, &path).await.unwrap();
      let loaded = load_from_path(&path).await.unwrap();
      assert_eq!(loaded.services[0].service_type, st);
    }
  }

  #[tokio::test]
  async fn active_key_uuid_persiste() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let key_id: Uuid = "660e8400-e29b-41d4-a716-446655440002".parse().unwrap();

    let mut config = Config::default();
    let mut svc = make_service(ServiceType::GitLab);
    svc.active_key = Some(key_id);
    config.services.push(svc);

    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();
    assert_eq!(loaded.services[0].active_key, Some(key_id));
  }

  // ── Scanner de clefs SSH ──────────────────────────────────────────

  #[tokio::test]
  async fn scanner_detecte_clef_ed25519() {
    let dir = TempDir::new().unwrap();
    write_pub(&dir, "id_ed25519.pub", ED25519_PUB);

    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 1);
    assert!(!keys[0].fingerprint.is_empty());
    assert_eq!(keys[0].comment, "test@host");
    assert_eq!(keys[0].key_type, KeyType::Ed25519);
  }

  #[tokio::test]
  async fn scanner_ignore_clefs_privees() {
    let dir = TempDir::new().unwrap();
    write_pub(&dir, "id_ed25519", "-----BEGIN OPENSSH PRIVATE KEY-----\n");
    write_pub(&dir, "id_ed25519.pub", ED25519_PUB);

    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 1);
  }

  #[tokio::test]
  async fn scanner_repertoire_vide() {
    let dir = TempDir::new().unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert!(keys.is_empty());
  }

  #[tokio::test]
  async fn scanner_fichier_corrompu_ignore_sans_panique() {
    let dir = TempDir::new().unwrap();
    write_pub(&dir, "bad.pub", "not a valid public key\n");
    write_pub(&dir, "good.pub", ED25519_PUB);

    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 1);
  }

  // ── Modèle de données ─────────────────────────────────────────────

  #[test]
  fn service_type_display() {
    assert_eq!(ServiceType::GitHub.to_string(), "GitHub");
    assert_eq!(ServiceType::GitLab.to_string(), "GitLab");
    assert_eq!(
      ServiceType::GitLabSelfHosted.to_string(),
      "GitLab (self-hosted)"
    );
    assert_eq!(ServiceType::SshGeneric.to_string(), "SSH générique");
    assert_eq!(ServiceType::Manual.to_string(), "Manuel");
  }

  #[test]
  fn key_type_display() {
    assert_eq!(KeyType::Ed25519.to_string(), "ed25519");
    assert_eq!(KeyType::SkEd25519.to_string(), "sk-ed25519");
  }

  #[tokio::test]
  async fn ecriture_atomique_pas_de_tmp_residuel() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    save_to_path(&Config::default(), &path).await.unwrap();
    assert!(path.exists());
    assert!(!dir.path().join("config.yaml.tmp").exists());
  }

  #[tokio::test]
  #[cfg(unix)]
  async fn ecriture_permissions_600() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    save_to_path(&Config::default(), &path).await.unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
  }

  // ── Compatibilité v0.1.0 → v0.2.0 ────────────────────────────────

  #[tokio::test]
  async fn config_v010_sans_uuid_acceptee() {
    // v0.1.0 pouvait générer des configs sans champ id sur les services.
    // Le champ est désormais `#[serde(default = "Uuid::new_v4")]`.
    let yaml = r#"
services:
  - name: "Ancien service"
    service_type: ssh-generic
    active_key: null
    created_at: "2025-06-01"
"#;
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, yaml);
    let config = load_from_path(&path).await.unwrap();
    // Un UUID doit avoir été généré automatiquement
    assert_ne!(config.services[0].id, Uuid::nil());
  }

  #[tokio::test]
  async fn config_v010_sans_deploy_mode_acceptee() {
    // v0.1.0 n'avait pas de champ deploy_mode.
    let yaml = r#"
services:
  - id: "770e8400-e29b-41d4-a716-446655440003"
    name: "Prod"
    service_type: manual
    active_key: null
    created_at: "2025-06-01"
"#;
    let dir = TempDir::new().unwrap();
    let path = write_yaml(&dir, yaml);
    let config = load_from_path(&path).await.unwrap();
    // La valeur par défaut automatic doit s'appliquer
    assert_eq!(config.services[0].deploy_mode, DeployMode::Automatic);
  }

  #[tokio::test]
  async fn clef_ssh_en_config_keys_preservee_au_round_trip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let key_id = Uuid::new_v4();

    let mut config = Config::default();
    config.keys.push(SshKey {
      id: key_id,
      fingerprint: "SHA256:AbCdEfGhIjKlMnOpQrStUvWxYz01234567890abc".into(),
      key_type: KeyType::Ed25519,
      yubikey: false,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      comment: "test@machine".into(),
      private_path: None,
      public_path: None,
      service_id: None,
    });

    save_to_path(&config, &path).await.unwrap();
    let loaded = load_from_path(&path).await.unwrap();
    assert_eq!(loaded.keys.len(), 1);
    assert_eq!(loaded.keys[0].id, key_id);
    assert_eq!(
      loaded.keys[0].fingerprint,
      "SHA256:AbCdEfGhIjKlMnOpQrStUvWxYz01234567890abc"
    );
  }
}
