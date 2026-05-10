#![allow(dead_code)]
use std::path::Path;

use crate::error::AppError;
use crate::subprocess::{CommandRunner, RealRunner};

#[derive(Debug, Clone)]
pub struct GpgKeyInfo {
  pub fingerprint: String,
  pub uid: String,
  pub expires: Option<chrono::NaiveDate>,
}

pub async fn list_secret_keys() -> Result<Vec<GpgKeyInfo>, AppError> {
  list_secret_keys_with(&RealRunner).await
}

pub async fn list_secret_keys_with(
  runner: &dyn CommandRunner,
) -> Result<Vec<GpgKeyInfo>, AppError> {
  let out = runner
    .run(
      "gpg",
      &["--list-secret-keys", "--with-colons", "--with-fingerprint"],
      10,
    )
    .await?;

  if out.exit_code != 0 {
    return Err(classify_gpg_error("list-secret-keys", &out.stderr));
  }
  Ok(parse_gpg_colons(&out.stdout))
}

/// Chiffre `plaintext` via stdin → jamais de fichier en clair sur disque.
pub async fn encrypt(
  plaintext: &[u8],
  recipient_fp: &str,
  dest_path: &Path,
) -> Result<(), AppError> {
  use tokio::io::AsyncWriteExt;

  let dest_str = dest_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(dest_path.to_path_buf()))?;

  let mut child = tokio::process::Command::new("gpg")
    .args([
      "--batch",
      "--yes",
      "--quiet",
      "--cipher-algo",
      "AES256",
      "--digest-algo",
      "SHA512",
      "--compress-algo",
      "none", // pas de compression — oracle CRIME-like
      "--encrypt",
      "--recipient",
      recipient_fp,
      "--output",
      dest_str,
    ])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true)
    .spawn()
    .map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        AppError::BinaryNotFound("gpg".to_string())
      } else {
        AppError::SubprocessSpawn {
          program: "gpg".to_string(),
          message: e.to_string(),
        }
      }
    })?;

  if let Some(mut stdin) = child.stdin.take() {
    stdin.write_all(plaintext).await.ok();
    // drop ferme stdin → gpg commence le chiffrement
  }

  let output = tokio::time::timeout(std::time::Duration::from_secs(30), child.wait_with_output())
    .await
    .map_err(|_| AppError::SubprocessTimeout {
      program: "gpg --encrypt".to_string(),
      timeout_secs: 30,
    })?
    .map_err(|e| AppError::SubprocessIo {
      program: "gpg".to_string(),
      message: e.to_string(),
    })?;

  if !output.status.success() {
    return Err(classify_gpg_error(
      "encrypt",
      &String::from_utf8_lossy(&output.stderr),
    ));
  }

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(dest_path, std::fs::Permissions::from_mode(0o600));
  }

  Ok(())
}

pub async fn decrypt(src_path: &Path) -> Result<Vec<u8>, AppError> {
  decrypt_with(src_path, &RealRunner).await
}

pub async fn decrypt_with(
  src_path: &Path,
  runner: &dyn CommandRunner,
) -> Result<Vec<u8>, AppError> {
  let src_str = src_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(src_path.to_path_buf()))?;

  let out = runner
    .run("gpg", &["--batch", "--decrypt", "--quiet", src_str], 30)
    .await?;

  if out.exit_code != 0 {
    return Err(classify_gpg_error("decrypt", &out.stderr));
  }
  Ok(out.stdout.into_bytes())
}

fn classify_gpg_error(operation: &str, stderr: &str) -> AppError {
  if stderr.contains("No secret key") {
    AppError::GpgNoSecretKey
  } else if stderr.contains("decryption failed") {
    AppError::GpgDecryptFailed
  } else if stderr.contains("can't connect to the agent") || stderr.contains("IPC connect") {
    AppError::GpgAgentUnavailable
  } else {
    AppError::GpgFailed {
      operation: operation.to_string(),
      stderr: stderr.to_string(),
    }
  }
}

/// Parse la sortie `--with-colons` de gpg.
fn parse_gpg_colons(output: &str) -> Vec<GpgKeyInfo> {
  let mut keys = Vec::new();
  let mut current_fp: Option<String> = None;

  for line in output.lines() {
    // Split illimité — splitn(10) capturerait tout le reste dans fields[9]
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 10 {
      continue;
    }
    match fields[0] {
      "fpr" => {
        current_fp = Some(fields[9].to_string());
      }
      "uid" => {
        if let Some(fp) = current_fp.take() {
          // Le champ uid (index 9) contient "Prénom Nom <email>"
          let uid = fields[9].to_string();
          if !uid.is_empty() {
            keys.push(GpgKeyInfo {
              fingerprint: fp,
              uid,
              expires: None,
            });
          } else {
            // uid vide : remettre le fingerprint en attente
            current_fp = Some(fp);
          }
        }
      }
      _ => {}
    }
  }
  keys
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::subprocess::fake::FakeRunner;

  const GPG_COLONS_OUTPUT: &str = "\
sec:u:4096:1:ABCDEF12345678:1609459200:::-:::scESC:::::::0:\n\
fpr:::::::::ABCDEF1234567890ABCDEF1234567890ABCDEF12:\n\
uid:u::::1609459200::HASHVAL::Guillaume <guillaume@example.com>:::::::::0:\n\
ssb:u:4096:1:SUBKEY12345678:1609459200::::::e::::::0:\n";

  #[test]
  fn parse_colons_extrait_cle_et_uid() {
    let keys = parse_gpg_colons(GPG_COLONS_OUTPUT);
    assert_eq!(keys.len(), 1);
    assert_eq!(
      keys[0].fingerprint,
      "ABCDEF1234567890ABCDEF1234567890ABCDEF12"
    );
    assert!(keys[0].uid.contains("Guillaume"));
  }

  #[test]
  fn parse_colons_vide_retourne_vide() {
    assert!(parse_gpg_colons("").is_empty());
  }

  #[tokio::test]
  async fn list_keys_succes() {
    let runner = FakeRunner::succeeds_with(GPG_COLONS_OUTPUT);
    let keys = list_secret_keys_with(&runner).await.unwrap();
    assert_eq!(keys.len(), 1);
  }

  #[tokio::test]
  async fn list_keys_gpg_absent() {
    let runner = FakeRunner::binary_not_found("gpg");
    assert!(matches!(
      list_secret_keys_with(&runner).await,
      Err(AppError::BinaryNotFound(_))
    ));
  }

  #[tokio::test]
  async fn decrypt_agent_indisponible() {
    let runner = FakeRunner::fails_with(
      2,
      "gpg: can't connect to the agent: IPC connect call failed",
    );
    assert!(matches!(
      decrypt_with(std::path::Path::new("/tmp/s.gpg"), &runner).await,
      Err(AppError::GpgAgentUnavailable)
    ));
  }

  #[tokio::test]
  async fn decrypt_mauvaise_clef() {
    let runner = FakeRunner::fails_with(2, "gpg: decryption failed: No secret key");
    assert!(matches!(
      decrypt_with(std::path::Path::new("/tmp/s.gpg"), &runner).await,
      Err(AppError::GpgNoSecretKey)
    ));
  }

  #[tokio::test]
  async fn decrypt_fichier_corrompu() {
    let runner = FakeRunner::fails_with(2, "gpg: no valid OpenPGP data found");
    assert!(matches!(
      decrypt_with(std::path::Path::new("/tmp/s.gpg"), &runner).await,
      Err(AppError::GpgFailed { .. })
    ));
  }
}
