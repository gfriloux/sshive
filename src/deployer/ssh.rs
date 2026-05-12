use std::sync::Arc;

use async_trait::async_trait;

use super::{DeployContext, KeyDeployer};
use crate::error::AppError;
use crate::subprocess::ssh_copy_id::{
  deploy_key_with, guided_deploy_command, revoke_key_with, verify_connection_with,
};
use crate::subprocess::{CommandRunner, RealRunner};

pub struct SshCopyIdDeployer {
  runner: Arc<dyn CommandRunner>,
}

impl Default for SshCopyIdDeployer {
  fn default() -> Self {
    Self::new()
  }
}

impl SshCopyIdDeployer {
  pub fn new() -> Self {
    Self {
      runner: Arc::new(RealRunner),
    }
  }

  #[cfg(test)]
  pub fn with_runner(runner: Arc<dyn CommandRunner>) -> Self {
    Self { runner }
  }
}

#[async_trait]
impl KeyDeployer for SshCopyIdDeployer {
  async fn deploy(&self, ctx: &DeployContext) -> Result<Option<String>, AppError> {
    let pub_path = ctx
      .public_key_path
      .as_ref()
      .ok_or_else(|| AppError::Validation {
        field: "key.public_path".into(),
        reason: "Chemin de la clef publique requis pour le déploiement SSH.".into(),
      })?;

    deploy_key_with(
      pub_path,
      &ctx.user,
      &ctx.host,
      ctx.port,
      self.runner.as_ref(),
    )
    .await?;
    Ok(None) // SSH n'a pas d'id distant
  }

  async fn revoke(&self, ctx: &DeployContext, _remote_ref: Option<&str>) -> Result<(), AppError> {
    let priv_path = ctx
      .private_key_path
      .as_ref()
      .ok_or_else(|| AppError::Validation {
        field: "key.private_path".into(),
        reason: "Chemin de la clef privée requis pour la révocation SSH.".into(),
      })?;

    revoke_key_with(
      priv_path,
      &ctx.fingerprint,
      &ctx.user,
      &ctx.host,
      ctx.port,
      self.runner.as_ref(),
    )
    .await
  }

  async fn verify(&self, ctx: &DeployContext) -> Result<bool, AppError> {
    let priv_path = ctx
      .private_key_path
      .as_ref()
      .ok_or_else(|| AppError::Validation {
        field: "key.private_path".into(),
        reason: "Chemin de la clef privée requis pour la vérification SSH.".into(),
      })?;

    verify_connection_with(
      priv_path,
      &ctx.user,
      &ctx.host,
      ctx.port,
      self.runner.as_ref(),
    )
    .await
  }

  fn guided_command(&self, ctx: &DeployContext) -> Option<String> {
    ctx
      .public_key_path
      .as_ref()
      .map(|pub_path| guided_deploy_command(pub_path, &ctx.user, &ctx.host, ctx.port))
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::*;
  use crate::subprocess::fake::FakeRunner;

  fn make_ctx(host: &str, user: &str) -> DeployContext {
    DeployContext {
      service_id: uuid::Uuid::new_v4(),
      host: host.to_string(),
      user: user.to_string(),
      port: 22,
      public_key_path: Some("/tmp/key.pub".into()),
      private_key_path: Some("/tmp/key".into()),
      fingerprint: "SHA256:test".into(),
      key_comment: "test".into(),
      token: None,
    }
  }

  #[tokio::test]
  async fn deploy_succes_retourne_none() {
    let runner = Arc::new(FakeRunner::succeeds_with(""));
    let deployer = SshCopyIdDeployer::with_runner(runner);
    let ctx = make_ctx("prod.example.com", "deploy");
    assert_eq!(deployer.deploy(&ctx).await.unwrap(), None);
  }

  #[tokio::test]
  async fn deploy_echec_retourne_erreur() {
    let runner = Arc::new(FakeRunner::fails_with(1, "Connection refused"));
    let deployer = SshCopyIdDeployer::with_runner(runner);
    let ctx = make_ctx("prod.example.com", "deploy");
    assert!(deployer.deploy(&ctx).await.is_err());
  }

  #[tokio::test]
  async fn deploy_sans_public_path_retourne_validation_error() {
    let runner = Arc::new(FakeRunner::new(vec![]));
    let deployer = SshCopyIdDeployer::with_runner(runner);
    let mut ctx = make_ctx("prod.example.com", "deploy");
    ctx.public_key_path = None;
    assert!(matches!(
      deployer.deploy(&ctx).await,
      Err(crate::error::AppError::Validation { .. })
    ));
  }

  #[tokio::test]
  async fn verify_succes_retourne_true() {
    let runner = Arc::new(FakeRunner::succeeds_with(""));
    let deployer = SshCopyIdDeployer::with_runner(runner);
    let ctx = make_ctx("prod.example.com", "deploy");
    assert!(deployer.verify(&ctx).await.unwrap());
  }

  #[tokio::test]
  async fn revoke_succes() {
    let runner = Arc::new(FakeRunner::succeeds_with(""));
    let deployer = SshCopyIdDeployer::with_runner(runner);
    let ctx = make_ctx("prod.example.com", "deploy");
    assert!(deployer.revoke(&ctx, None).await.is_ok());
  }

  #[test]
  fn guided_command_contient_ssh_copy_id() {
    let deployer = SshCopyIdDeployer::new();
    let ctx = make_ctx("prod.example.com", "deploy");
    let cmd = deployer.guided_command(&ctx).unwrap();
    assert!(cmd.contains("ssh-copy-id"));
    assert!(cmd.contains("prod.example.com"));
  }

  #[test]
  fn guided_command_sans_public_path_retourne_none() {
    let deployer = SshCopyIdDeployer::new();
    let mut ctx = make_ctx("prod.example.com", "deploy");
    ctx.public_key_path = None;
    assert!(deployer.guided_command(&ctx).is_none());
  }
}
