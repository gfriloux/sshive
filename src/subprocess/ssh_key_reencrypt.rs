#![allow(dead_code)]

/// Ré-encryption de clefs SSH privées via la crate `ssh-key` (crypto Rust pur).
/// Aucun subprocess, aucune passphrase en argv — remplace `ssh-keygen -p -N`.
use rand_core::OsRng;
use ssh_key::{LineEnding, PrivateKey};

use crate::error::AppError;
use crate::subprocess::pinentry::Passphrase;

/// Ré-encrypte une clef privée OpenSSH en mémoire.
///
/// `pem` : contenu du fichier clef privée (format OpenSSH).
/// `new_passphrase` : nouvelle phrase secrète (≥ 12 chars).
///
/// Retourne le PEM rechiffré. La clef originale reste inchangée sur disque —
/// l'appelant est responsable de l'écriture atomique.
pub fn reencrypt_key_bytes(pem: &str, new_passphrase: &Passphrase) -> Result<String, AppError> {
  if new_passphrase.len() < 12 {
    return Err(AppError::Validation {
      field: "passphrase".into(),
      reason: "La phrase secrète doit faire au moins 12 caractères.".into(),
    });
  }

  let key = PrivateKey::from_openssh(pem.trim().as_bytes()).map_err(|e| AppError::ParseError {
    context: "clef privée OpenSSH".into(),
    raw: e.to_string(),
  })?;

  let encrypted = key
    .encrypt(&mut OsRng, new_passphrase.as_bytes())
    .map_err(|e| AppError::SubprocessFailed {
      program: "ssh-key::encrypt".into(),
      exit_code: -1,
      stderr: e.to_string(),
    })?;

  let pem_out = encrypted
    .to_openssh(LineEnding::LF)
    .map_err(|e| AppError::SubprocessFailed {
      program: "ssh-key::to_openssh".into(),
      exit_code: -1,
      stderr: e.to_string(),
    })?;

  Ok(pem_out.to_string())
}

#[cfg(test)]
mod tests {
  use ssh_key::Algorithm;

  use super::*;
  use crate::subprocess::ssh_keygen::openssh_key_cipher_is_none;

  /// Génère une clef ed25519 non protégée en PEM via ssh-key (vraie clef valide).
  fn make_unencrypted_pem() -> String {
    PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
      .expect("génération clef")
      .to_openssh(LineEnding::LF)
      .expect("sérialisation PEM")
      .to_string()
  }

  #[test]
  fn reencrypt_non_protege_devient_protege() {
    let pem = make_unencrypted_pem();
    let pass = Passphrase::new("hunter2-correct-horse-battery".into());
    let result = reencrypt_key_bytes(&pem, &pass);
    assert!(result.is_ok(), "erreur inattendue : {:?}", result);
    let encrypted_pem = result.unwrap();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &encrypted_pem).unwrap();
    assert_eq!(
      openssh_key_cipher_is_none(tmp.path()),
      Some(false),
      "la clef doit être chiffrée après ré-encryption"
    );
  }

  #[test]
  fn reencrypt_round_trip_format_valide() {
    let pem = make_unencrypted_pem();
    let pass = Passphrase::new("hunter2-correct-horse-battery".into());
    let encrypted_pem = reencrypt_key_bytes(&pem, &pass).unwrap();

    // Le PEM rechiffré doit commencer par l'en-tête OpenSSH
    assert!(
      encrypted_pem.contains("-----BEGIN OPENSSH PRIVATE KEY-----"),
      "format PEM invalide"
    );

    // Le cipher doit être différent de "none"
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &encrypted_pem).unwrap();
    assert_eq!(
      openssh_key_cipher_is_none(tmp.path()),
      Some(false),
      "le cipher doit être non-none après chiffrement"
    );

    // La structure PEM doit être parseable par ssh-key
    let parsed = PrivateKey::from_openssh(encrypted_pem.trim().as_bytes());
    assert!(
      parsed.is_ok(),
      "PEM rechiffré doit être parseable : {:?}",
      parsed.err()
    );
  }

  #[test]
  fn reencrypt_passphrase_trop_courte_rejetee() {
    let pem = make_unencrypted_pem();
    let pass = Passphrase::new("court".into());
    assert!(matches!(
      reencrypt_key_bytes(&pem, &pass),
      Err(AppError::Validation { .. })
    ));
  }

  #[test]
  fn reencrypt_pem_invalide_retourne_parse_error() {
    let pass = Passphrase::new("hunter2-correct-horse-battery".into());
    assert!(matches!(
      reencrypt_key_bytes("not a key", &pass),
      Err(AppError::ParseError { .. })
    ));
  }

  #[test]
  fn reencrypt_pem_vide_retourne_parse_error() {
    let pass = Passphrase::new("hunter2-correct-horse-battery".into());
    assert!(matches!(
      reencrypt_key_bytes("", &pass),
      Err(AppError::ParseError { .. })
    ));
  }
}
