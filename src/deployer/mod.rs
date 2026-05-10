#![allow(dead_code)]

pub mod github;
pub mod gitlab;
pub mod http;
pub mod ssh;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::config::model::{Service, ServiceType, SshKey};
use crate::error::AppError;
use crate::secrets::model::{ApiToken, Secrets};
use crate::subprocess::CommandRunner;

// ── DeployContext — données résolues pour une opération de déploiement ────

/// Données concrètes extraites de `Service` + `SshKey` + `Secrets`.
/// Propriété complète pour éviter les problèmes de lifetime dans async.
pub struct DeployContext {
  pub service_id: Uuid,
  pub host: String,
  pub user: String,
  pub port: u16,
  pub public_key_path: Option<PathBuf>,
  pub private_key_path: Option<PathBuf>,
  pub fingerprint: String,
  pub key_comment: String,
  pub token: Option<ApiToken>,
}

impl DeployContext {
  pub fn build(service: &Service, key: &SshKey, token: Option<ApiToken>) -> Result<Self, AppError> {
    let host = service.params.url.clone().unwrap_or_default();
    let user = service.params.user.clone().unwrap_or_default();
    let port = service.params.port.unwrap_or(22);

    if host.is_empty() || user.is_empty() {
      return Err(AppError::Validation {
        field: "service.params".into(),
        reason: "Hôte et utilisateur requis pour le déploiement SSH.".into(),
      });
    }

    Ok(Self {
      service_id: service.id,
      host,
      user,
      port,
      public_key_path: key.public_path.clone(),
      private_key_path: key.private_path.clone(),
      fingerprint: key.fingerprint.clone(),
      key_comment: key.comment.clone(),
      token,
    })
  }

  /// Contexte minimal pour les services API (GitHub/GitLab) sans params SSH.
  pub fn build_api(service: &Service, key: &SshKey, token: Option<ApiToken>) -> Self {
    Self {
      service_id: service.id,
      host: service.params.url.clone().unwrap_or_default(),
      user: String::new(),
      port: 0,
      public_key_path: key.public_path.clone(),
      private_key_path: key.private_path.clone(),
      fingerprint: key.fingerprint.clone(),
      key_comment: key.comment.clone(),
      token,
    }
  }
}

// ── KeyDeployer — trait unifié SSH + API ─────────────────────────────────

#[async_trait]
pub trait KeyDeployer: Send + Sync {
  /// Déploie la clef. Retourne la référence distante (id GitHub/GitLab) ou None pour SSH.
  async fn deploy(&self, ctx: &DeployContext) -> Result<Option<String>, AppError>;

  /// Révoque une clef par sa référence distante (None = fingerprint pour SSH).
  async fn revoke(&self, ctx: &DeployContext, remote_ref: Option<&str>) -> Result<(), AppError>;

  /// Vérifie que la clef est active sur la destination.
  async fn verify(&self, ctx: &DeployContext) -> Result<bool, AppError>;

  /// Commande pour le mode guidé — None si non applicable.
  fn guided_command(&self, ctx: &DeployContext) -> Option<String>;
}

// ── Factory ────────────────────────────────────────────────────────────────

/// Sélectionne l'implémentation selon le type de service.
pub fn deployer_for(service: &Service, secrets: &Secrets) -> Box<dyn KeyDeployer> {
  let token = secrets.token_for(service);
  match service.service_type {
    ServiceType::GitHub => Box::new(github::GitHubApiDeployer::new(token)),
    ServiceType::GitLab => Box::new(gitlab::GitLabApiDeployer::new(
      "https://gitlab.com".into(),
      token,
    )),
    ServiceType::GitLabSelfHosted => {
      let base = service
        .params
        .url
        .clone()
        .unwrap_or_else(|| "https://gitlab.com".into());
      Box::new(gitlab::GitLabApiDeployer::new(base, token))
    }
    ServiceType::SshGeneric | ServiceType::Manual => Box::new(ssh::SshCopyIdDeployer::new()),
  }
}

// ── FakeDeployer pour les tests ───────────────────────────────────────────

#[cfg(test)]
pub mod fake {
  use std::collections::VecDeque;
  use std::sync::Mutex;

  use async_trait::async_trait;

  use super::{DeployContext, KeyDeployer};
  use crate::error::AppError;

  pub struct FakeDeployer {
    deploy_responses: Mutex<VecDeque<Result<Option<String>, AppError>>>,
    revoke_responses: Mutex<VecDeque<Result<(), AppError>>>,
    verify_responses: Mutex<VecDeque<Result<bool, AppError>>>,
  }

  impl FakeDeployer {
    pub fn deploy_succeeds(remote_ref: Option<&str>) -> Self {
      Self {
        deploy_responses: Mutex::new(vec![Ok(remote_ref.map(|s| s.to_string()))].into()),
        revoke_responses: Mutex::new(vec![Ok(())].into()),
        verify_responses: Mutex::new(vec![Ok(true)].into()),
      }
    }

    pub fn deploy_fails(msg: &str) -> Self {
      Self {
        deploy_responses: Mutex::new(
          vec![Err(AppError::SubprocessFailed {
            program: "fake".into(),
            exit_code: 1,
            stderr: msg.to_string(),
          })]
          .into(),
        ),
        revoke_responses: Mutex::new(vec![].into()),
        verify_responses: Mutex::new(vec![].into()),
      }
    }
  }

  #[async_trait]
  impl KeyDeployer for FakeDeployer {
    async fn deploy(&self, _ctx: &DeployContext) -> Result<Option<String>, AppError> {
      self
        .deploy_responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeDeployer: plus de réponses deploy")
    }

    async fn revoke(&self, _ctx: &DeployContext, _ref: Option<&str>) -> Result<(), AppError> {
      self
        .revoke_responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeDeployer: plus de réponses revoke")
    }

    async fn verify(&self, _ctx: &DeployContext) -> Result<bool, AppError> {
      self
        .verify_responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeDeployer: plus de réponses verify")
    }

    fn guided_command(&self, _ctx: &DeployContext) -> Option<String> {
      Some("fake-deploy-command".into())
    }
  }
}

// Référence non utilisée dans le type — Arc<dyn CommandRunner> utilisé dans ssh.rs
#[allow(dead_code)]
fn _unused_runner_import(_: Arc<dyn CommandRunner>) {}
