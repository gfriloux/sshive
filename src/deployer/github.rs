use async_trait::async_trait;
use serde_json::json;

use super::{DeployContext, KeyDeployer};
use crate::deployer::http::{HttpClient, ReqwestHttpClient};
use crate::error::AppError;
use crate::secrets::model::ApiToken;

const GITHUB_API: &str = "https://api.github.com";

pub struct GitHubApiDeployer {
  token: Option<ApiToken>,
  client: Box<dyn HttpClient>,
}

impl GitHubApiDeployer {
  pub fn new(token: Option<ApiToken>) -> Self {
    Self {
      token,
      client: Box::new(ReqwestHttpClient::default()),
    }
  }

  #[cfg(test)]
  pub fn with_client(token: Option<ApiToken>, client: Box<dyn HttpClient>) -> Self {
    Self { token, client }
  }

  fn auth_header(&self) -> Result<String, AppError> {
    let token = self.token.as_ref().ok_or_else(|| AppError::Validation {
      field: "token".into(),
      reason: "Token GitHub requis pour le déploiement via API.".into(),
    })?;
    Ok(format!("Bearer {}", token.expose()))
  }
}

#[async_trait]
impl KeyDeployer for GitHubApiDeployer {
  #[tracing::instrument(skip(self, ctx), fields(service_id = %ctx.service_id))]
  async fn deploy(&self, ctx: &DeployContext) -> Result<Option<String>, AppError> {
    let auth = self.auth_header()?;

    let pub_path = ctx
      .public_key_path
      .as_ref()
      .ok_or_else(|| AppError::Validation {
        field: "key.public_path".into(),
        reason: "Chemin de la clef publique requis.".into(),
      })?;

    let pub_content = tokio::fs::read_to_string(pub_path)
      .await
      .map_err(|e| AppError::Io {
        path: pub_path.clone(),
        message: e.to_string(),
      })?;

    let resp = self
      .client
      .post_json(
        &format!("{GITHUB_API}/user/keys"),
        &[
          ("Authorization", auth.as_str()),
          ("Accept", "application/vnd.github+json"),
          ("X-GitHub-Api-Version", "2022-11-28"),
        ],
        json!({
          "title": &ctx.key_comment,
          "key": pub_content.trim(),
        }),
      )
      .await?;

    match resp.status {
      201 => {
        let id = resp.body["id"].as_u64().map(|n| n.to_string());
        Ok(id)
      }
      422 => Err(AppError::ApiKeyAlreadyPresent {
        service: ctx.service_id.to_string(),
      }),
      401 | 403 => Err(AppError::ApiUnauthorized {
        service: ctx.service_id.to_string(),
      }),
      s => Err(AppError::ApiError {
        service: ctx.service_id.to_string(),
        status: s,
        message: resp.body["message"]
          .as_str()
          .unwrap_or("erreur inconnue")
          .to_string(),
      }),
    }
  }

  #[tracing::instrument(skip(self, ctx), fields(service_id = %ctx.service_id))]
  async fn revoke(&self, ctx: &DeployContext, remote_ref: Option<&str>) -> Result<(), AppError> {
    let id = remote_ref.ok_or_else(|| AppError::Validation {
      field: "remote_ref".into(),
      reason: "Identifiant de la clef GitHub requis pour la révocation.".into(),
    })?;

    let auth = self.auth_header()?;
    let resp = self
      .client
      .delete(
        &format!("{GITHUB_API}/user/keys/{id}"),
        &[
          ("Authorization", auth.as_str()),
          ("Accept", "application/vnd.github+json"),
          ("X-GitHub-Api-Version", "2022-11-28"),
        ],
      )
      .await?;

    match resp.status {
      204 | 404 => Ok(()), // 404 = déjà supprimée, idempotent
      401 | 403 => Err(AppError::ApiUnauthorized {
        service: ctx.service_id.to_string(),
      }),
      s => Err(AppError::ApiError {
        service: ctx.service_id.to_string(),
        status: s,
        message: resp.body["message"]
          .as_str()
          .unwrap_or("erreur inconnue")
          .to_string(),
      }),
    }
  }

  #[tracing::instrument(skip(self, ctx), fields(service_id = %ctx.service_id))]
  async fn verify(&self, ctx: &DeployContext) -> Result<bool, AppError> {
    let auth = self.auth_header()?;
    let resp = self
      .client
      .get_json(
        &format!("{GITHUB_API}/user/keys"),
        &[
          ("Authorization", auth.as_str()),
          ("Accept", "application/vnd.github+json"),
          ("X-GitHub-Api-Version", "2022-11-28"),
        ],
      )
      .await?;

    if !resp.is_success() {
      return Ok(false);
    }

    // GitHub ne retourne pas le fingerprint — on compare par titre (commentaire de la clef)
    let found = resp.body.as_array().is_some_and(|keys| {
      keys
        .iter()
        .any(|k| k["title"].as_str() == Some(&ctx.key_comment))
    });

    Ok(found)
  }

  fn guided_command(&self, ctx: &DeployContext) -> Option<String> {
    let pub_path = ctx.public_key_path.as_ref()?;
    Some(format!(
      "curl -X POST https://api.github.com/user/keys \\\n  \
       -H 'Authorization: Bearer ***' \\\n  \
       -H 'Accept: application/vnd.github+json' \\\n  \
       -d '{{\"title\":\"{}\",\"key\":\"$(cat {})\"}}' ",
      ctx.key_comment,
      pub_path.display()
    ))
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;
  use crate::deployer::http::fake::FakeHttpClient;
  use crate::deployer::DeployContext;

  fn make_ctx() -> DeployContext {
    DeployContext {
      service_id: uuid::Uuid::new_v4(),
      host: String::new(),
      user: String::new(),
      port: 0,
      public_key_path: Some("/tmp/key.pub".into()),
      private_key_path: None,
      fingerprint: "SHA256:test".into(),
      key_comment: "sshive/github_perso/2026-01-01".into(),
      token: Some(crate::secrets::model::ApiToken::new("ghp_test".into())),
    }
  }

  fn make_deployer(client: Box<dyn HttpClient>) -> GitHubApiDeployer {
    GitHubApiDeployer::with_client(
      Some(crate::secrets::model::ApiToken::new("ghp_test".into())),
      client,
    )
  }

  #[tokio::test]
  async fn deploy_succes_retourne_id() {
    // Crée un vrai fichier pour le test
    let dir = tempfile::TempDir::new().unwrap();
    let pub_path = dir.path().join("key.pub");
    std::fs::write(&pub_path, "ssh-ed25519 AAAA... test").unwrap();

    let client = Box::new(FakeHttpClient::responds_with(201, json!({"id": 12345678})));
    let deployer = make_deployer(client);
    let mut ctx = make_ctx();
    ctx.public_key_path = Some(pub_path);

    let result = deployer.deploy(&ctx).await.unwrap();
    assert_eq!(result, Some("12345678".to_string()));
  }

  #[tokio::test]
  async fn deploy_cle_existante_retourne_already_present() {
    let dir = tempfile::TempDir::new().unwrap();
    let pub_path = dir.path().join("key.pub");
    std::fs::write(&pub_path, "ssh-ed25519 AAAA... test").unwrap();

    let client = Box::new(FakeHttpClient::responds_with(
      422,
      json!({"message": "key already in use"}),
    ));
    let deployer = make_deployer(client);
    let mut ctx = make_ctx();
    ctx.public_key_path = Some(pub_path);

    assert!(matches!(
      deployer.deploy(&ctx).await,
      Err(AppError::ApiKeyAlreadyPresent { .. })
    ));
  }

  #[tokio::test]
  async fn deploy_token_invalide_retourne_unauthorized() {
    let dir = tempfile::TempDir::new().unwrap();
    let pub_path = dir.path().join("key.pub");
    std::fs::write(&pub_path, "ssh-ed25519 AAAA... test").unwrap();

    let client = Box::new(FakeHttpClient::responds_with(
      401,
      json!({"message": "Bad credentials"}),
    ));
    let deployer = make_deployer(client);
    let mut ctx = make_ctx();
    ctx.public_key_path = Some(pub_path);

    assert!(matches!(
      deployer.deploy(&ctx).await,
      Err(AppError::ApiUnauthorized { .. })
    ));
  }

  #[tokio::test]
  async fn deploy_token_absent_rejete_avant_http() {
    let client = Box::new(FakeHttpClient::new(vec![])); // ne doit pas être consommé
    let deployer = GitHubApiDeployer::with_client(None, client);
    let ctx = make_ctx();
    assert!(matches!(
      deployer.deploy(&ctx).await,
      Err(AppError::Validation { .. })
    ));
  }

  #[tokio::test]
  async fn revoke_succes_204() {
    let client = Box::new(FakeHttpClient::responds_with(204, json!(null)));
    let deployer = make_deployer(client);
    let ctx = make_ctx();
    assert!(deployer.revoke(&ctx, Some("12345678")).await.is_ok());
  }

  #[tokio::test]
  async fn revoke_idempotent_404() {
    let client = Box::new(FakeHttpClient::responds_with(
      404,
      json!({"message": "not found"}),
    ));
    let deployer = make_deployer(client);
    let ctx = make_ctx();
    assert!(deployer.revoke(&ctx, Some("12345678")).await.is_ok());
  }

  #[tokio::test]
  async fn timeout_reseau_retourne_erreur() {
    let client = Box::new(FakeHttpClient::fails_with_timeout());
    let deployer = make_deployer(client);
    let ctx = make_ctx();
    assert!(matches!(
      deployer.revoke(&ctx, Some("123")).await,
      Err(AppError::SubprocessTimeout { .. })
    ));
  }

  #[test]
  fn guided_command_masque_token() {
    let deployer = make_deployer(Box::new(FakeHttpClient::new(vec![])));
    let mut ctx = make_ctx();
    ctx.public_key_path = Some("/home/user/.ssh/sshive_github_2026.pub".into());
    let cmd = deployer.guided_command(&ctx).unwrap();
    assert!(cmd.contains("***"));
    assert!(!cmd.contains("ghp_test"));
    assert!(cmd.contains("api.github.com"));
  }
}
