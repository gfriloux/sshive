use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ssh_key::public::KeyData;

use crate::config::model::{KeyType, SshKey};
use crate::error::AppError;

pub async fn scan_pub_keys() -> Result<Vec<SshKey>, AppError> {
  let dir = dirs::home_dir()
    .map(|h| h.join(".ssh"))
    .ok_or(AppError::NoHomeDir)?;
  scan_from_dir(&dir).await
}

pub async fn scan_from_dir(dir: &Path) -> Result<Vec<SshKey>, AppError> {
  if !dir.exists() {
    return Ok(vec![]);
  }

  let pattern = dir.join("*.pub").to_string_lossy().into_owned();
  let paths: Vec<PathBuf> = glob::glob(&pattern)
    .map_err(|e| AppError::Glob(e.to_string()))?
    .filter_map(|e| e.ok())
    .collect();

  let mut keys = Vec::new();
  for path in paths {
    match std::fs::symlink_metadata(&path) {
      Ok(meta) if meta.file_type().is_symlink() => {
        tracing::warn!("Lien symbolique ignoré : {}", path.display());
        continue;
      }
      _ => {}
    }

    match parse_pub_key_file(&path).await {
      Ok(Some(key)) => keys.push(key),
      Ok(None) => {}
      Err(e) => tracing::warn!("Impossible de lire {} : {e}", path.display()),
    }
  }

  Ok(keys)
}

async fn parse_pub_key_file(path: &Path) -> Result<Option<SshKey>, AppError> {
  let content = tokio::fs::read_to_string(path)
    .await
    .map_err(|e| AppError::Io {
      path: path.to_path_buf(),
      message: e.to_string(),
    })?;

  let trimmed = content.trim();
  if trimmed.is_empty() {
    return Ok(None);
  }

  if trimmed.contains("PRIVATE KEY") || trimmed.starts_with("-----BEGIN") {
    tracing::warn!("Clef privée détectée dans {} — ignorée", path.display());
    return Ok(None);
  }

  let pub_key = match ssh_key::PublicKey::from_openssh(trimmed) {
    Ok(k) => k,
    Err(_) => return Ok(None),
  };

  let key_type = match pub_key.key_data() {
    KeyData::Ed25519(_) => KeyType::Ed25519,
    KeyData::SkEd25519(_) => KeyType::SkEd25519,
    _ => return Ok(None),
  };

  let fingerprint = pub_key.fingerprint(ssh_key::HashAlg::Sha256).to_string();
  let comment = pub_key.comment().to_string();
  let yubikey = key_type == KeyType::SkEd25519;
  let created_at = mtime_as_naive_date(path);

  Ok(Some(SshKey {
    id: uuid::Uuid::new_v4(),
    fingerprint,
    key_type,
    yubikey,
    created_at,
    comment,
    private_path: None,
    public_path: Some(path.to_path_buf()),
    service_id: None,
    backup_prompted: false,
  }))
}

fn mtime_as_naive_date(path: &Path) -> chrono::NaiveDate {
  std::fs::metadata(path)
    .ok()
    .and_then(|m| m.modified().ok())
    .and_then(|t| {
      let secs = t.duration_since(UNIX_EPOCH).ok()?.as_secs();
      chrono::DateTime::from_timestamp(secs as i64, 0).map(|dt| dt.date_naive())
    })
    .unwrap_or_else(|| {
      chrono::NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 est une date valide")
    })
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tempfile::TempDir;

  use super::*;

  const ED25519_PUB: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GkZH test@sshive";

  const PRIVATE_KEY_PEM: &str =
    "-----BEGIN OPENSSH PRIVATE KEY-----\nfake\n-----END OPENSSH PRIVATE KEY-----";

  #[tokio::test]
  async fn parse_clef_ed25519_valide() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("id_ed25519.pub"), ED25519_PUB).unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].key_type, KeyType::Ed25519);
    assert!(!keys[0].yubikey);
    assert!(keys[0].fingerprint.starts_with("SHA256:"));
    assert_eq!(keys[0].comment, "test@sshive");
  }

  #[tokio::test]
  async fn repertoire_absent_retourne_vide() {
    let result = scan_from_dir(Path::new("/chemin/qui/nexiste/certainement/pas")).await;
    assert!(matches!(result, Ok(keys) if keys.is_empty()));
  }

  #[tokio::test]
  async fn repertoire_vide_retourne_vide() {
    let dir = TempDir::new().unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert!(keys.is_empty());
  }

  #[tokio::test]
  async fn fichier_corrompu_ignore_les_autres_charges() {
    let dir = TempDir::new().unwrap();
    fs::write(
      dir.path().join("bad.pub"),
      b"contenu\x00invalide\xffinvalid" as &[u8],
    )
    .unwrap();
    fs::write(dir.path().join("good.pub"), ED25519_PUB).unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 1);
  }

  #[tokio::test]
  async fn fichier_vide_ignore() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("empty.pub"), "").unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert!(keys.is_empty());
  }

  #[tokio::test]
  async fn clef_privee_ignoree_avec_warning() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("private.pub"), PRIVATE_KEY_PEM).unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert!(keys.is_empty());
  }

  #[tokio::test]
  async fn symlink_ignore() {
    let dir = TempDir::new().unwrap();
    let real = dir.path().join("real.pub");
    let link = dir.path().join("link.pub");
    fs::write(&real, ED25519_PUB).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    // seul le fichier réel est chargé, le symlink est ignoré
    assert_eq!(keys.len(), 1);
  }

  #[tokio::test]
  async fn plusieurs_clefs_chargees() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("key1.pub"), ED25519_PUB).unwrap();
    fs::write(
      dir.path().join("key2.pub"),
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB4JHXE9YCiFl1GIHPrQHYBn8cCQP8+qfJvKt3DQCZ3F other@sshive",
    )
    .unwrap();
    let keys = scan_from_dir(dir.path()).await.unwrap();
    assert_eq!(keys.len(), 2);
  }
}
