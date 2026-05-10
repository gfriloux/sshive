use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub services: Vec<Service>,
  #[serde(default)]
  pub keys: Vec<SshKey>,
  #[serde(default)]
  pub gpg: GpgConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpgConfig {
  /// Fingerprint 40-hex de la clef GPG utilisée pour chiffrer secrets.yaml.gpg
  pub key_fingerprint: Option<String>,
  /// Chemin vers secrets.yaml.gpg (défaut : ~/.config/sshive/secrets.yaml.gpg)
  pub secrets_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
  #[serde(default = "Uuid::new_v4")]
  pub id: Uuid,
  pub name: String,
  pub service_type: ServiceType,
  #[serde(default)]
  pub params: ServiceParams,
  pub active_key: Option<Uuid>,
  pub pending_key: Option<Uuid>,
  pub created_at: NaiveDate,
  pub last_rotation: Option<NaiveDate>,
  #[serde(default)]
  pub deploy_mode: DeployMode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeployMode {
  #[default]
  #[serde(rename = "automatic")]
  Automatic,
  #[serde(rename = "guided")]
  Guided,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceParams {
  pub url: Option<String>,
  pub user: Option<String>,
  pub port: Option<u16>,
  pub token_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
  #[serde(rename = "github")]
  GitHub,
  #[serde(rename = "gitlab")]
  GitLab,
  #[serde(rename = "gitlab-self-hosted")]
  GitLabSelfHosted,
  #[serde(rename = "ssh-generic")]
  SshGeneric,
  #[serde(rename = "manual")]
  Manual,
}

impl std::fmt::Display for ServiceType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::GitHub => write!(f, "GitHub"),
      Self::GitLab => write!(f, "GitLab"),
      Self::GitLabSelfHosted => write!(f, "GitLab (self-hosted)"),
      Self::SshGeneric => write!(f, "SSH générique"),
      Self::Manual => write!(f, "Manuel"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
  #[serde(rename = "ed25519")]
  Ed25519,
  #[serde(rename = "sk-ed25519")]
  SkEd25519,
}

impl std::fmt::Display for KeyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Ed25519 => write!(f, "ed25519"),
      Self::SkEd25519 => write!(f, "sk-ed25519"),
    }
  }
}

/// Clef SSH gérée par SSHive.
/// Les clefs découvertes via scan `~/.ssh/*.pub` (non gérées) ont
/// `private_path` et `public_path` à None.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
  #[serde(default = "Uuid::new_v4")]
  pub id: Uuid,
  pub fingerprint: String,
  pub key_type: KeyType,
  #[serde(default)]
  pub yubikey: bool,
  pub created_at: NaiveDate,
  pub comment: String,
  /// Chemin de la clef privée (None pour les clefs scannées)
  #[serde(default)]
  pub private_path: Option<PathBuf>,
  /// Chemin de la clef publique (None pour les clefs scannées)
  #[serde(default)]
  pub public_path: Option<PathBuf>,
  /// UUID du service associé (None pour les clefs orphelines)
  #[serde(default)]
  pub service_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── Compatibilité v0.1.0 ──────────────────────────────────────

  #[test]
  fn config_v010_sans_uuid_deserialisable() {
    // Une config v0.1.0 sans champs UUID ni gpg doit se désérialiser correctement.
    // Les UUID sont auto-générés via #[serde(default = "Uuid::new_v4")].
    let yaml = r#"
services:
  - name: "GitHub perso"
    service_type: github
    active_key: null
    pending_key: null
    created_at: "2025-01-15"
    last_rotation: null
keys:
  - fingerprint: "SHA256:abc123"
    key_type: ed25519
    yubikey: false
    created_at: "2025-01-15"
    comment: "sshive/github/2025-01-15"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.services[0].name, "GitHub perso");
    assert_eq!(config.keys[0].key_type, KeyType::Ed25519);
    // GPG config absente → défaut
    assert!(config.gpg.key_fingerprint.is_none());
    // Champs v0.2.0 absents → défauts
    assert_eq!(config.services[0].deploy_mode, DeployMode::Automatic);
    assert!(config.keys[0].private_path.is_none());
  }

  #[test]
  fn config_v010_champs_v020_ignores() {
    // Un champ futur inconnu ne casse pas la désérialisation
    let yaml = "services: []\nkeys: []\nchamp_futur: valeur";
    assert!(serde_yaml::from_str::<Config>(yaml).is_ok());
  }

  // ── Tests v0.1.0 conservés ────────────────────────────────────

  #[test]
  fn config_vide_deserialisable() {
    let config: Config = serde_yaml::from_str("{}").unwrap();
    assert!(config.services.is_empty());
    assert!(config.keys.is_empty());
  }

  #[test]
  fn config_default_round_trip() {
    let config = Config::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
    assert!(parsed.services.is_empty());
    assert!(parsed.keys.is_empty());
  }

  #[test]
  fn service_type_toutes_variantes() {
    for (s, expected) in [
      ("github", ServiceType::GitHub),
      ("gitlab", ServiceType::GitLab),
      ("gitlab-self-hosted", ServiceType::GitLabSelfHosted),
      ("ssh-generic", ServiceType::SshGeneric),
      ("manual", ServiceType::Manual),
    ] {
      let parsed: ServiceType = serde_yaml::from_str(s).unwrap();
      assert_eq!(parsed, expected, "variante échouée : {s}");
    }
  }

  #[test]
  fn service_type_invalide_erreur() {
    let result: Result<ServiceType, _> = serde_yaml::from_str("ftp-unknown");
    assert!(result.is_err());
  }

  #[test]
  fn key_type_toutes_variantes() {
    for (s, expected) in [
      ("ed25519", KeyType::Ed25519),
      ("sk-ed25519", KeyType::SkEd25519),
    ] {
      let parsed: KeyType = serde_yaml::from_str(s).unwrap();
      assert_eq!(parsed, expected, "variante échouée : {s}");
    }
  }

  #[test]
  fn service_type_display() {
    assert_eq!(ServiceType::GitHub.to_string(), "GitHub");
    assert_eq!(
      ServiceType::GitLabSelfHosted.to_string(),
      "GitLab (self-hosted)"
    );
    assert_eq!(ServiceType::SshGeneric.to_string(), "SSH générique");
  }

  #[test]
  fn key_type_display() {
    assert_eq!(KeyType::Ed25519.to_string(), "ed25519");
    assert_eq!(KeyType::SkEd25519.to_string(), "sk-ed25519");
  }

  // ── Nouveaux tests v0.2.0 ─────────────────────────────────────

  #[test]
  fn service_round_trip_avec_uuid() {
    let yaml = r#"
services:
  - id: "550e8400-e29b-41d4-a716-446655440000"
    name: "Prod server"
    service_type: ssh-generic
    params:
      url: "prod.example.com"
      user: "deploy"
      port: 2222
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
    deploy_mode: guided
keys: []
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.services[0].deploy_mode, DeployMode::Guided);
    assert_eq!(
      config.services[0].id.to_string(),
      "550e8400-e29b-41d4-a716-446655440000"
    );

    let re_yaml = serde_yaml::to_string(&config).unwrap();
    let config2: Config = serde_yaml::from_str(&re_yaml).unwrap();
    assert_eq!(config.services[0].id, config2.services[0].id);
  }

  #[test]
  fn gpg_config_round_trip() {
    let yaml = r#"
services: []
keys: []
gpg:
  key_fingerprint: "ABCDEF1234567890ABCDEF1234567890ABCDEF12"
  secrets_path: null
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(
      config.gpg.key_fingerprint.as_deref(),
      Some("ABCDEF1234567890ABCDEF1234567890ABCDEF12")
    );
  }

  #[test]
  fn ssh_key_avec_chemins_round_trip() {
    let yaml = r#"
services: []
keys:
  - fingerprint: "SHA256:xyz789"
    key_type: ed25519
    yubikey: false
    created_at: "2026-01-01"
    comment: "sshive/prod/2026-01-01"
    private_path: "/home/user/.ssh/sshive_prod_2026-01-01"
    public_path: "/home/user/.ssh/sshive_prod_2026-01-01.pub"
    service_id: "550e8400-e29b-41d4-a716-446655440000"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert!(config.keys[0].private_path.is_some());
    assert!(config.keys[0].service_id.is_some());
  }

  #[test]
  fn deploy_mode_defaut_est_automatic() {
    let yaml = r#"
services:
  - name: "Test"
    service_type: manual
    active_key: null
    pending_key: null
    created_at: "2026-01-01"
    last_rotation: null
keys: []
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.services[0].deploy_mode, DeployMode::Automatic);
  }
}
