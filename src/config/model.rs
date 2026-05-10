use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
  #[serde(default)]
  pub services: Vec<Service>,
  #[serde(default)]
  pub keys: Vec<SshKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
  pub name: String,
  pub service_type: ServiceType,
  #[serde(default)]
  pub params: ServiceParams,
  pub active_key: Option<String>,
  pub pending_key: Option<String>,
  pub created_at: NaiveDate,
  pub last_rotation: Option<NaiveDate>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
  pub fingerprint: String,
  pub key_type: KeyType,
  #[serde(default)]
  pub yubikey: bool,
  pub created_at: NaiveDate,
  pub comment: String,
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn config_complete_round_trip() {
    let yaml = r#"
services:
  - name: "GitHub perso"
    service_type: github
    params:
      token_ref: "sshive/github_perso"
    active_key: "SHA256:abc123"
    pending_key: null
    created_at: "2025-01-15"
    last_rotation: "2025-06-01"
keys:
  - fingerprint: "SHA256:abc123"
    key_type: ed25519
    yubikey: false
    created_at: "2025-01-15"
    comment: "sshive/github_perso/2025-01-15"
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.services.len(), 1);
    assert_eq!(config.services[0].name, "GitHub perso");
    assert_eq!(config.services[0].service_type, ServiceType::GitHub);
    assert_eq!(config.keys[0].key_type, KeyType::Ed25519);

    let re_yaml = serde_yaml::to_string(&config).unwrap();
    let config2: Config = serde_yaml::from_str(&re_yaml).unwrap();
    assert_eq!(config.services[0].name, config2.services[0].name);
  }

  #[test]
  fn champs_inconnus_ignores() {
    let yaml = "services: []\nkeys: []\nchamp_futur: valeur";
    assert!(serde_yaml::from_str::<Config>(yaml).is_ok());
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
  fn service_sans_clef_active_valide() {
    let yaml = r#"
services:
  - name: "Sans clef"
    service_type: manual
    active_key: null
    pending_key: null
    created_at: "2025-01-01"
    last_rotation: null
keys: []
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert!(config.services[0].active_key.is_none());
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
}
