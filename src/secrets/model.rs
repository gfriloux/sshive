use std::collections::HashMap;

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::config::model::Service;

/// Token API (GitHub, GitLab…) — ne s'affiche jamais en clair dans les logs.
#[derive(Clone)]
pub struct ApiToken(SecretString);

impl ApiToken {
  pub fn new(s: String) -> Self {
    Self(SecretString::new(s))
  }

  /// Expose la valeur brute — uniquement dans les headers HTTP, jamais dans les logs.
  pub fn expose(&self) -> &str {
    self.0.expose_secret()
  }
}

impl std::fmt::Debug for ApiToken {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ApiToken(***)")
  }
}

/// Contenu de secrets.yaml.gpg — chiffré au repos via GPG.
/// Note : la protection principale est le chiffrement GPG du fichier.
/// La zéroïsation en mémoire (HashMap ne supporte pas Zeroize) est
/// une amélioration future via un type dédié.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Secrets {
  /// Tokens API indexés par `token_ref` (ex. "sshive/github_perso")
  #[serde(default)]
  pub tokens: HashMap<String, String>,
}

impl Secrets {
  /// Retourne le token associé au service, ou `None` si absent.
  /// Principe de moindre privilège : les deployers ne reçoivent pas `&Secrets` entier.
  pub fn token_for(&self, service: &Service) -> Option<ApiToken> {
    let key = service.params.token_ref.as_ref()?;
    self.tokens.get(key).map(|v| ApiToken::new(v.clone()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::model::{DeployMode, ServiceParams, ServiceType};
  use chrono::NaiveDate;
  use uuid::Uuid;

  fn make_service(token_ref: Option<&str>) -> Service {
    Service {
      id: Uuid::new_v4(),
      name: "Test".into(),
      service_type: ServiceType::GitHub,
      params: ServiceParams {
        token_ref: token_ref.map(|s| s.to_string()),
        ..Default::default()
      },
      active_key: None,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    }
  }

  #[test]
  fn token_for_retourne_token_existant() {
    let mut secrets = Secrets::default();
    secrets
      .tokens
      .insert("github_perso".into(), "ghp_abc123".into());
    let service = make_service(Some("github_perso"));
    let token = secrets.token_for(&service).unwrap();
    assert_eq!(token.expose(), "ghp_abc123");
  }

  #[test]
  fn token_for_retourne_none_si_ref_absente() {
    let secrets = Secrets::default();
    let service = make_service(None);
    assert!(secrets.token_for(&service).is_none());
  }

  #[test]
  fn token_for_retourne_none_si_token_inconnu() {
    let secrets = Secrets::default();
    let service = make_service(Some("ref_inexistante"));
    assert!(secrets.token_for(&service).is_none());
  }

  #[test]
  fn api_token_debug_masque_la_valeur() {
    let token = ApiToken::new("ghp_super_secret".into());
    let debug = format!("{:?}", token);
    assert_eq!(debug, "ApiToken(***)");
    assert!(!debug.contains("super_secret"));
  }

  #[test]
  fn api_token_expose_retourne_valeur_brute() {
    let token = ApiToken::new("ghp_abc123".into());
    assert_eq!(token.expose(), "ghp_abc123");
  }
}
