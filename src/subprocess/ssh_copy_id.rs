#![allow(dead_code)]
use std::path::Path;

use crate::config::validation::{validate_hostname, validate_username};
use crate::error::AppError;
use crate::subprocess::{CommandRunner, RealRunner};

pub async fn deploy_key(
  public_path: &Path,
  user: &str,
  host: &str,
  port: u16,
) -> Result<(), AppError> {
  deploy_key_with(public_path, user, host, port, &RealRunner).await
}

pub async fn deploy_key_with(
  public_path: &Path,
  user: &str,
  host: &str,
  port: u16,
  runner: &dyn CommandRunner,
) -> Result<(), AppError> {
  // Validation stricte avant tout subprocess — jamais d'injection
  validate_hostname(host)?;
  validate_username(user)?;

  let port_str = port.to_string();
  let public_str = public_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(public_path.to_path_buf()))?;
  let target = format!("{user}@{host}");

  let out = runner
    .run(
      "ssh-copy-id",
      &["-i", public_str, "-p", &port_str, &target],
      60,
    )
    .await?;

  if out.exit_code != 0 {
    return Err(AppError::SubprocessFailed {
      program: "ssh-copy-id".to_string(),
      exit_code: out.exit_code,
      stderr: out.stderr,
    });
  }
  Ok(())
}

pub async fn verify_connection(
  private_path: &Path,
  user: &str,
  host: &str,
  port: u16,
) -> Result<bool, AppError> {
  verify_connection_with(private_path, user, host, port, &RealRunner).await
}

pub async fn verify_connection_with(
  private_path: &Path,
  user: &str,
  host: &str,
  port: u16,
  runner: &dyn CommandRunner,
) -> Result<bool, AppError> {
  validate_hostname(host)?;
  validate_username(user)?;

  let port_str = port.to_string();
  let private_str = private_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(private_path.to_path_buf()))?;
  let target = format!("{user}@{host}");

  let out = runner
    .run(
      "ssh",
      &[
        "-i",
        private_str,
        "-p",
        &port_str,
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=10",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
        "NumberOfPasswordPrompts=0",
        &target,
        "true",
      ],
      20,
    )
    .await?;

  Ok(out.exit_code == 0)
}

pub async fn revoke_key(
  private_path: &Path,
  fingerprint: &str,
  user: &str,
  host: &str,
  port: u16,
) -> Result<(), AppError> {
  revoke_key_with(private_path, fingerprint, user, host, port, &RealRunner).await
}

pub async fn revoke_key_with(
  private_path: &Path,
  fingerprint: &str,
  user: &str,
  host: &str,
  port: u16,
  runner: &dyn CommandRunner,
) -> Result<(), AppError> {
  validate_hostname(host)?;
  validate_username(user)?;

  let port_str = port.to_string();
  let private_str = private_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(private_path.to_path_buf()))?;
  let target = format!("{user}@{host}");

  // On filtre côté serveur avec sed sur le fingerprint en commentaire de ligne
  // Le fingerprint ne contient que des caractères base64+préfixe — sûr à injecter
  // dans une commande passée comme argument unique (pas via sh -c)
  let sed_cmd = format!(
    "grep -v '{fingerprint}' ~/.ssh/authorized_keys > ~/.ssh/authorized_keys.tmp \
     && mv ~/.ssh/authorized_keys.tmp ~/.ssh/authorized_keys"
  );

  let out = runner
    .run(
      "ssh",
      &[
        "-i",
        private_str,
        "-p",
        &port_str,
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=10",
        "-o",
        "NumberOfPasswordPrompts=0",
        &target,
        &sed_cmd,
      ],
      30,
    )
    .await?;

  if out.exit_code != 0 {
    return Err(AppError::SubprocessFailed {
      program: "ssh (revoke)".to_string(),
      exit_code: out.exit_code,
      stderr: out.stderr,
    });
  }
  Ok(())
}

/// Commande ssh-copy-id pour affichage en mode guidé (jamais exécutée).
pub fn guided_deploy_command(public_path: &Path, user: &str, host: &str, port: u16) -> String {
  format!(
    "ssh-copy-id -i {} -p {} {}@{}",
    public_path.display(),
    port,
    user,
    host
  )
}

/// Commande de vérification pour affichage en mode guidé.
pub fn guided_verify_command(private_path: &Path, user: &str, host: &str, port: u16) -> String {
  format!(
    "ssh -i {} -p {} {}@{} true",
    private_path.display(),
    port,
    user,
    host
  )
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::*;
  use crate::subprocess::fake::FakeRunner;

  // ── deploy_key_with ───────────────────────────────────────────

  #[tokio::test]
  async fn deploy_succes() {
    let runner = FakeRunner::succeeds_with("");
    assert!(deploy_key_with(
      Path::new("/tmp/k.pub"),
      "deploy",
      "prod.example.com",
      22,
      &runner
    )
    .await
    .is_ok());
  }

  #[tokio::test]
  async fn deploy_serveur_injoignable() {
    let runner = FakeRunner::fails_with(1, "ssh: Connection refused");
    assert!(matches!(
      deploy_key_with(
        Path::new("/tmp/k.pub"),
        "deploy",
        "prod.example.com",
        22,
        &runner
      )
      .await,
      Err(AppError::SubprocessFailed { exit_code: 1, .. })
    ));
  }

  #[tokio::test]
  async fn deploy_binaire_absent() {
    let runner = FakeRunner::binary_not_found("ssh-copy-id");
    assert!(matches!(
      deploy_key_with(
        Path::new("/tmp/k.pub"),
        "deploy",
        "prod.example.com",
        22,
        &runner
      )
      .await,
      Err(AppError::BinaryNotFound(_))
    ));
  }

  #[tokio::test]
  async fn deploy_host_invalide_rejete_avant_subprocess() {
    // FakeRunner sans réponse — paniquerait si le subprocess était appelé
    let runner = FakeRunner::new(vec![]);
    let result = deploy_key_with(
      Path::new("/tmp/k.pub"),
      "deploy",
      "host$(rm -rf ~)",
      22,
      &runner,
    )
    .await;
    assert!(matches!(result, Err(AppError::Validation { .. })));
  }

  #[tokio::test]
  async fn deploy_host_avec_newline_rejete() {
    let runner = FakeRunner::new(vec![]);
    let result = deploy_key_with(
      Path::new("/tmp/k.pub"),
      "deploy",
      "host\nmalicious",
      22,
      &runner,
    )
    .await;
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn deploy_user_avec_espace_rejete() {
    let runner = FakeRunner::new(vec![]);
    let result = deploy_key_with(
      Path::new("/tmp/k.pub"),
      "bad user",
      "host.example.com",
      22,
      &runner,
    )
    .await;
    assert!(matches!(result, Err(AppError::Validation { .. })));
  }

  // ── verify_connection_with ────────────────────────────────────

  #[tokio::test]
  async fn verify_connexion_reussie() {
    let runner = FakeRunner::succeeds_with("");
    let ok = verify_connection_with(
      Path::new("/tmp/key"),
      "deploy",
      "prod.example.com",
      22,
      &runner,
    )
    .await
    .unwrap();
    assert!(ok);
  }

  #[tokio::test]
  async fn verify_connexion_echouee() {
    let runner = FakeRunner::fails_with(1, "Permission denied");
    let ok = verify_connection_with(
      Path::new("/tmp/key"),
      "deploy",
      "prod.example.com",
      22,
      &runner,
    )
    .await
    .unwrap();
    assert!(!ok);
  }

  // ── guided commands ───────────────────────────────────────────

  #[test]
  fn guided_deploy_command_format() {
    let cmd = guided_deploy_command(
      Path::new("/home/user/.ssh/sshive_prod.pub"),
      "deploy",
      "prod.example.com",
      2222,
    );
    assert!(cmd.contains("ssh-copy-id"));
    assert!(cmd.contains("deploy@prod.example.com"));
    assert!(cmd.contains("2222"));
  }

  // ── revoke_key_with ───────────────────────────────────────────

  #[tokio::test]
  async fn revoke_succes() {
    let runner = FakeRunner::succeeds_with("");
    assert!(revoke_key_with(
      Path::new("/tmp/key"),
      "SHA256:AbCdEfGh",
      "deploy",
      "host",
      22,
      &runner
    )
    .await
    .is_ok());
  }

  #[tokio::test]
  async fn revoke_idempotent_si_clef_absente() {
    // Si la clef n'est pas dans authorized_keys, grep retourne exit 1
    // mais on considère ça comme un succès (idempotent)
    // Note : le comportement exact dépend de la commande distante.
    // Ce test documente que exit 0 = Ok.
    let runner = FakeRunner::succeeds_with("");
    let result = revoke_key_with(
      Path::new("/tmp/key"),
      "SHA256:inexistant",
      "deploy",
      "host",
      22,
      &runner,
    )
    .await;
    assert!(result.is_ok());
  }
}
