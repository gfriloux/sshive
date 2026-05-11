use async_trait::async_trait;
use serde_json::json;

use super::{DeployContext, KeyDeployer};
use crate::deployer::http::{HttpClient, ReqwestHttpClient};
use crate::error::AppError;
use crate::secrets::model::ApiToken;

pub struct GitLabApiDeployer {
  base_url: String,
  token: Option<ApiToken>,
  client: Box<dyn HttpClient>,
}

impl GitLabApiDeployer {
  pub fn new(base_url: String, token: Option<ApiToken>) -> Self {
    Self {
      base_url,
      token,
      client: Box::new(ReqwestHttpClient::default()),
    }
  }

  #[cfg(test)]
  pub fn with_client(
    base_url: String,
    token: Option<ApiToken>,
    client: Box<dyn HttpClient>,
  ) -> Self {
    Self {
      base_url,
      token,
      client,
    }
  }

  fn auth_header(&self) -> Result<String, AppError> {
    let token = self.token.as_ref().ok_or_else(|| AppError::Validation {
      field: "token".into(),
      reason: "Token GitLab requis pour le déploiement via API.".into(),
    })?;
    Ok(token.expose().to_string())
  }

  fn keys_url(&self) -> String {
    format!("{}/api/v4/user/keys", self.base_url.trim_end_matches('/'))
  }

  fn user_url(&self) -> String {
    format!("{}/api/v4/user", self.base_url.trim_end_matches('/'))
  }

  /// Sonde le token via GET /api/v4/user — vérifie qu'il est valide et autorisé.
  pub async fn probe_token(&self) -> Result<(), AppError> {
    let token = self.auth_header()?;
    let resp = self
      .client
      .get_json(&self.user_url(), &[("Private-Token", token.as_str())])
      .await?;
    match resp.status {
      200 => Ok(()),
      401 | 403 => Err(AppError::ApiUnauthorized {
        service: "GitLab".into(),
      }),
      status => Err(AppError::ApiError {
        service: "GitLab".into(),
        status,
        message: resp.body["message"]
          .as_str()
          .unwrap_or("erreur inconnue")
          .to_string(),
      }),
    }
  }
}

#[async_trait]
impl KeyDeployer for GitLabApiDeployer {
  #[tracing::instrument(skip(self, ctx), fields(service_id = %ctx.service_id))]
  async fn deploy(&self, ctx: &DeployContext) -> Result<Option<String>, AppError> {
    let token = self.auth_header()?;

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
        &self.keys_url(),
        &[("Private-Token", token.as_str())],
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
      400 => {
        // Sur POST /api/v4/user/keys, 400 signifie toujours "clef déjà présente"
        // (le body peut être {"message":{"fingerprint":[...]}} ou {"message":"..."})
        Err(AppError::ApiKeyAlreadyPresent {
          service: ctx.service_id.to_string(),
        })
      }
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
      reason: "Identifiant de la clef GitLab requis pour la révocation.".into(),
    })?;

    let token = self.auth_header()?;
    let resp = self
      .client
      .delete(
        &format!("{}/{id}", self.keys_url()),
        &[("Private-Token", token.as_str())],
      )
      .await?;

    match resp.status {
      204 | 404 => Ok(()),
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
    let token = self.auth_header()?;
    let resp = self
      .client
      .get_json(&self.keys_url(), &[("Private-Token", token.as_str())])
      .await?;

    if !resp.is_success() {
      return Ok(false);
    }

    let found = resp.body.as_array().is_some_and(|keys| {
      keys
        .iter()
        .any(|k| k["title"].as_str() == Some(&ctx.key_comment))
    });

    Ok(found)
  }

  fn guided_command(&self, ctx: &DeployContext) -> Option<String> {
    let pub_path = ctx.public_key_path.as_ref()?;
    let api_url = self.keys_url();
    Some(format!(
      "curl -X POST {api_url} \\\n  \
       -H 'Private-Token: ***' \\\n  \
       -H 'Content-Type: application/json' \\\n  \
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
  use crate::secrets::model::ApiToken;

  fn make_ctx() -> DeployContext {
    DeployContext {
      service_id: uuid::Uuid::new_v4(),
      host: String::new(),
      user: String::new(),
      port: 0,
      public_key_path: Some("/tmp/key.pub".into()),
      private_key_path: None,
      fingerprint: "SHA256:test".into(),
      key_comment: "sshive/gitlab_ci/2026-01-01".into(),
      token: Some(ApiToken::new("glpat-test".into())),
    }
  }

  fn make_deployer(client: Box<dyn HttpClient>) -> GitLabApiDeployer {
    GitLabApiDeployer::with_client(
      "https://gitlab.com".into(),
      Some(ApiToken::new("glpat-test".into())),
      client,
    )
  }

  #[tokio::test]
  async fn deploy_succes_retourne_id() {
    let dir = tempfile::TempDir::new().unwrap();
    let pub_path = dir.path().join("key.pub");
    std::fs::write(&pub_path, "ssh-ed25519 AAAA... test").unwrap();

    let client = Box::new(FakeHttpClient::responds_with(201, json!({"id": 42})));
    let deployer = make_deployer(client);
    let mut ctx = make_ctx();
    ctx.public_key_path = Some(pub_path);

    assert_eq!(deployer.deploy(&ctx).await.unwrap(), Some("42".to_string()));
  }

  #[tokio::test]
  async fn deploy_cle_existante_400() {
    let dir = tempfile::TempDir::new().unwrap();
    let pub_path = dir.path().join("key.pub");
    std::fs::write(&pub_path, "ssh-ed25519 AAAA... test").unwrap();

    let client = Box::new(FakeHttpClient::responds_with(
      400,
      json!({"message": {"fingerprint": ["Fingerprint has already been taken"]}}),
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
  async fn deploy_token_absent_rejete_avant_http() {
    let client = Box::new(FakeHttpClient::new(vec![]));
    let deployer = GitLabApiDeployer::with_client("https://gitlab.com".into(), None, client);
    let ctx = make_ctx();
    assert!(matches!(
      deployer.deploy(&ctx).await,
      Err(AppError::Validation { .. })
    ));
  }

  #[tokio::test]
  async fn revoke_idempotent_404() {
    let client = Box::new(FakeHttpClient::responds_with(
      404,
      json!({"message": "not found"}),
    ));
    let deployer = make_deployer(client);
    assert!(deployer.revoke(&make_ctx(), Some("42")).await.is_ok());
  }

  #[test]
  fn guided_command_masque_token() {
    let deployer = make_deployer(Box::new(FakeHttpClient::new(vec![])));
    let mut ctx = make_ctx();
    ctx.public_key_path = Some("/home/user/.ssh/sshive_gitlab_2026.pub".into());
    let cmd = deployer.guided_command(&ctx).unwrap();
    assert!(cmd.contains("***"));
    assert!(!cmd.contains("glpat-test"));
    assert!(cmd.contains("api/v4/user/keys"));
  }

  #[test]
  fn base_url_self_hosted_dans_la_commande() {
    let deployer = GitLabApiDeployer::with_client(
      "https://git.example.com".into(),
      Some(ApiToken::new("tok".into())),
      Box::new(FakeHttpClient::new(vec![])),
    );
    let mut ctx = make_ctx();
    ctx.public_key_path = Some("/tmp/key.pub".into());
    let cmd = deployer.guided_command(&ctx).unwrap();
    assert!(cmd.contains("git.example.com"));
  }

  // ── Tests probe_token ─────────────────────────────────────────────────────

  fn make_probe_deployer(client: Box<dyn HttpClient>) -> GitLabApiDeployer {
    GitLabApiDeployer::with_client(
      "https://gitlab.com".into(),
      Some(ApiToken::new("glpat-test".into())),
      client,
    )
  }

  #[tokio::test]
  async fn probe_token_200_passe() {
    let client = Box::new(FakeHttpClient::responds_with(200, json!({"id": 1})));
    let deployer = make_probe_deployer(client);
    assert!(deployer.probe_token().await.is_ok());
  }

  #[tokio::test]
  async fn probe_token_401_retourne_unauthorized() {
    let client = Box::new(FakeHttpClient::responds_with(
      401,
      json!({"message": "401 Unauthorized"}),
    ));
    let deployer = make_probe_deployer(client);
    assert!(matches!(
      deployer.probe_token().await,
      Err(crate::error::AppError::ApiUnauthorized { .. })
    ));
  }

  #[tokio::test]
  async fn probe_token_403_retourne_unauthorized() {
    let client = Box::new(FakeHttpClient::responds_with(
      403,
      json!({"message": "Forbidden"}),
    ));
    let deployer = make_probe_deployer(client);
    assert!(matches!(
      deployer.probe_token().await,
      Err(crate::error::AppError::ApiUnauthorized { .. })
    ));
  }

  #[tokio::test]
  async fn probe_token_429_retourne_api_error() {
    let client = Box::new(FakeHttpClient::responds_with(
      429,
      json!({"message": "rate limit exceeded"}),
    ));
    let deployer = make_probe_deployer(client);
    assert!(matches!(
      deployer.probe_token().await,
      Err(crate::error::AppError::ApiError { status: 429, .. })
    ));
  }

  #[tokio::test]
  async fn probe_token_timeout_retourne_erreur() {
    let client = Box::new(FakeHttpClient::fails_with_timeout());
    let deployer = make_probe_deployer(client);
    assert!(matches!(
      deployer.probe_token().await,
      Err(crate::error::AppError::SubprocessTimeout { .. })
    ));
  }

  #[tokio::test]
  async fn probe_token_absent_rejete_sans_appel_http() {
    let client = Box::new(FakeHttpClient::new(vec![])); // ne doit pas être consommé
    let deployer = GitLabApiDeployer::with_client("https://gitlab.com".into(), None, client);
    assert!(matches!(
      deployer.probe_token().await,
      Err(crate::error::AppError::Validation { .. })
    ));
  }
}
