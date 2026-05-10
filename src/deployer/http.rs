use async_trait::async_trait;
use serde_json::Value;

use crate::error::AppError;

// ── Réponse HTTP simplifiée ───────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HttpResponse {
  pub status: u16,
  pub body: Value,
}

impl HttpResponse {
  pub fn is_success(&self) -> bool {
    self.status >= 200 && self.status < 300
  }
}

// ── Trait HttpClient ──────────────────────────────────────────────────────

#[async_trait]
pub trait HttpClient: Send + Sync {
  async fn post_json(
    &self,
    url: &str,
    headers: &[(&str, &str)],
    body: Value,
  ) -> Result<HttpResponse, AppError>;

  async fn delete(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, AppError>;

  async fn get_json(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, AppError>;
}

// ── Implémentation reqwest ────────────────────────────────────────────────

pub struct ReqwestHttpClient {
  client: reqwest::Client,
}

impl ReqwestHttpClient {
  pub fn new() -> Result<Self, AppError> {
    let client = reqwest::Client::builder()
      .user_agent(concat!("SSHive/", env!("CARGO_PKG_VERSION")))
      // Pas de redirection sur les endpoints auth
      .redirect(reqwest::redirect::Policy::none())
      .timeout(std::time::Duration::from_secs(30))
      .build()
      .map_err(|e| AppError::SubprocessSpawn {
        program: "reqwest::Client".into(),
        message: e.to_string(),
      })?;

    if std::env::var("HTTP_PROXY").is_ok() || std::env::var("HTTPS_PROXY").is_ok() {
      tracing::warn!(
        "Un proxy HTTP système est configuré — les tokens API transiteront par ce proxy."
      );
    }

    Ok(Self { client })
  }
}

impl Default for ReqwestHttpClient {
  fn default() -> Self {
    Self::new().expect("Impossible de créer le client HTTP")
  }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
  async fn post_json(
    &self,
    url: &str,
    headers: &[(&str, &str)],
    body: Value,
  ) -> Result<HttpResponse, AppError> {
    let mut req = self.client.post(url);
    for (k, v) in headers {
      req = req.header(*k, *v);
    }
    let resp = req.json(&body).send().await.map_err(map_reqwest_error)?;
    parse_response(resp).await
  }

  async fn delete(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, AppError> {
    let mut req = self.client.delete(url);
    for (k, v) in headers {
      req = req.header(*k, *v);
    }
    let resp = req.send().await.map_err(map_reqwest_error)?;
    parse_response(resp).await
  }

  async fn get_json(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, AppError> {
    let mut req = self.client.get(url);
    for (k, v) in headers {
      req = req.header(*k, *v);
    }
    let resp = req.send().await.map_err(map_reqwest_error)?;
    parse_response(resp).await
  }
}

async fn parse_response(resp: reqwest::Response) -> Result<HttpResponse, AppError> {
  let status = resp.status().as_u16();
  let body: Value = resp.json().await.unwrap_or(Value::Null);
  Ok(HttpResponse { status, body })
}

fn map_reqwest_error(e: reqwest::Error) -> AppError {
  if e.is_timeout() {
    AppError::SubprocessTimeout {
      program: "HTTP".into(),
      timeout_secs: 30,
    }
  } else {
    AppError::SubprocessIo {
      program: "HTTP".into(),
      message: e.to_string(),
    }
  }
}

// ── FakeHttpClient pour les tests ─────────────────────────────────────────

#[cfg(test)]
pub mod fake {
  use std::collections::VecDeque;
  use std::sync::Mutex;

  use async_trait::async_trait;
  use serde_json::Value;

  use super::{HttpClient, HttpResponse};
  use crate::error::AppError;

  pub struct FakeHttpClient {
    responses: Mutex<VecDeque<Result<HttpResponse, AppError>>>,
  }

  impl FakeHttpClient {
    pub fn new(responses: Vec<Result<HttpResponse, AppError>>) -> Self {
      Self {
        responses: Mutex::new(responses.into()),
      }
    }

    pub fn responds_with(status: u16, body: Value) -> Self {
      Self::new(vec![Ok(HttpResponse { status, body })])
    }

    pub fn fails_with_timeout() -> Self {
      Self::new(vec![Err(AppError::SubprocessTimeout {
        program: "HTTP".into(),
        timeout_secs: 30,
      })])
    }

    pub fn sequence(responses: Vec<Result<HttpResponse, AppError>>) -> Self {
      Self::new(responses)
    }
  }

  #[async_trait]
  impl HttpClient for FakeHttpClient {
    async fn post_json(
      &self,
      _url: &str,
      _headers: &[(&str, &str)],
      _body: Value,
    ) -> Result<HttpResponse, AppError> {
      self
        .responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeHttpClient: plus de réponses")
    }

    async fn delete(
      &self,
      _url: &str,
      _headers: &[(&str, &str)],
    ) -> Result<HttpResponse, AppError> {
      self
        .responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeHttpClient: plus de réponses")
    }

    async fn get_json(
      &self,
      _url: &str,
      _headers: &[(&str, &str)],
    ) -> Result<HttpResponse, AppError> {
      self
        .responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeHttpClient: plus de réponses")
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;
  use fake::FakeHttpClient;

  #[test]
  fn http_response_is_success() {
    assert!(HttpResponse {
      status: 201,
      body: json!(null)
    }
    .is_success());
    assert!(HttpResponse {
      status: 200,
      body: json!(null)
    }
    .is_success());
    assert!(!HttpResponse {
      status: 401,
      body: json!(null)
    }
    .is_success());
    assert!(!HttpResponse {
      status: 422,
      body: json!(null)
    }
    .is_success());
  }

  #[tokio::test]
  async fn fake_post_retourne_reponse_configuree() {
    let client = FakeHttpClient::responds_with(201, json!({"id": 42}));
    let resp = client
      .post_json("https://example.com", &[], json!({}))
      .await
      .unwrap();
    assert_eq!(resp.status, 201);
    assert_eq!(resp.body["id"], 42);
  }

  #[tokio::test]
  async fn fake_timeout_retourne_erreur() {
    let client = FakeHttpClient::fails_with_timeout();
    assert!(matches!(
      client
        .post_json("https://example.com", &[], json!({}))
        .await,
      Err(AppError::SubprocessTimeout { .. })
    ));
  }

  #[tokio::test]
  async fn fake_sequence_consomme_dans_l_ordre() {
    let client = FakeHttpClient::sequence(vec![
      Ok(HttpResponse {
        status: 201,
        body: json!({"id": 1}),
      }),
      Ok(HttpResponse {
        status: 200,
        body: json!({"ok": true}),
      }),
    ]);
    let r1 = client.post_json("u", &[], json!({})).await.unwrap();
    let r2 = client.get_json("u", &[]).await.unwrap();
    assert_eq!(r1.status, 201);
    assert_eq!(r2.status, 200);
  }
}
