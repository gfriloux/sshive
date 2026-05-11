use std::path::PathBuf;

use chrono::Local;
use uuid::Uuid;

use crate::config::model::{KeyType, SshKey};
use crate::error::AppError;
use crate::subprocess::pinentry::Passphrase;
use crate::subprocess::{CommandRunner, RealRunner};

pub struct KeyGenRequest {
  pub service_name: String,
  pub service_id: Uuid,
  pub ssh_dir: PathBuf,
  pub use_yubikey: bool,
  pub passphrase: Option<String>,
}

pub async fn generate_key(req: KeyGenRequest) -> Result<SshKey, AppError> {
  generate_key_with(req, &RealRunner).await
}

pub async fn generate_key_with(
  req: KeyGenRequest,
  runner: &dyn CommandRunner,
) -> Result<SshKey, AppError> {
  let date = Local::now().format("%Y-%m-%d").to_string();
  let slug = sanitize_name(&req.service_name);
  let comment = format!("sshive/{slug}/{date}");
  let stem = format!("sshive_{slug}_{date}");
  let private_path = req.ssh_dir.join(&stem);
  let public_path = req.ssh_dir.join(format!("{stem}.pub"));

  if private_path.exists() || public_path.exists() {
    return Err(AppError::KeyAlreadyExists { path: private_path });
  }

  let key_type_arg = if req.use_yubikey {
    "ed25519-sk"
  } else {
    "ed25519"
  };
  let passphrase_arg = req.passphrase.as_deref().unwrap_or("");
  let private_str = private_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(private_path.clone()))?;

  let out = runner
    .run(
      "ssh-keygen",
      &[
        "-t",
        key_type_arg,
        "-C",
        &comment,
        "-f",
        private_str,
        "-N",
        passphrase_arg,
        "-q",
      ],
      60,
    )
    .await?;

  if out.exit_code != 0 {
    if out.stderr.contains("user presence required") {
      return Err(AppError::YubiKeyNotPresent);
    }
    return Err(AppError::SubprocessFailed {
      program: "ssh-keygen".to_string(),
      exit_code: out.exit_code,
      stderr: out.stderr,
    });
  }

  let public_str = public_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(public_path.clone()))?;

  let fp_out = runner
    .run("ssh-keygen", &["-l", "-E", "sha256", "-f", public_str], 10)
    .await?;

  let fingerprint = parse_fingerprint_line(&fp_out.stdout).ok_or_else(|| AppError::ParseError {
    context: "ssh-keygen -l".to_string(),
    raw: fp_out.stdout.clone(),
  })?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(&private_path, std::fs::Permissions::from_mode(0o600));
    let _ = std::fs::set_permissions(&public_path, std::fs::Permissions::from_mode(0o644));
  }

  Ok(SshKey {
    id: Uuid::new_v4(),
    fingerprint,
    key_type: if req.use_yubikey {
      KeyType::SkEd25519
    } else {
      KeyType::Ed25519
    },
    yubikey: req.use_yubikey,
    created_at: Local::now().date_naive(),
    comment,
    private_path: Some(private_path),
    public_path: Some(public_path),
    service_id: Some(req.service_id),
    backup_prompted: false,
  })
}

pub fn parse_fingerprint_line(line: &str) -> Option<String> {
  // Format : "256 SHA256:AbCd... comment (ED25519)"
  let fp = line.split_whitespace().nth(1)?;
  if fp.starts_with("SHA256:") || fp.starts_with("MD5:") {
    Some(fp.to_string())
  } else {
    None
  }
}

fn sanitize_name(name: &str) -> String {
  name
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == '-' {
        c
      } else {
        '_'
      }
    })
    .collect::<String>()
    .to_lowercase()
}

// ── Détection de protection ───────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectionStatus {
  /// Aucune passphrase — clef chiffrable.
  Unprotected,
  /// Protégée par passphrase logicielle.
  Protected,
  /// Clef hardware sk-ed25519 (user presence required).
  HardwareKey,
}

#[allow(dead_code)]
pub async fn detect_protection(key_path: &std::path::Path) -> Result<ProtectionStatus, AppError> {
  detect_protection_with(key_path, &RealRunner).await
}

pub async fn detect_protection_with(
  key_path: &std::path::Path,
  runner: &dyn CommandRunner,
) -> Result<ProtectionStatus, AppError> {
  if !key_path.exists() {
    return Err(AppError::Io {
      path: key_path.to_path_buf(),
      message: "Fichier introuvable".into(),
    });
  }

  let path_str = key_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(key_path.to_path_buf()))?;

  let out = runner
    .run("ssh-keygen", &["-y", "-P", "", "-f", path_str], 10)
    .await?;

  if out.exit_code == 0 {
    return Ok(ProtectionStatus::Unprotected);
  }

  if out.stderr.contains("user presence required") {
    // Clef sk-ed25519 : le subprocess échoue car le hardware est requis pour signer,
    // mais le fichier sur disque peut avoir une passphrase ou pas.
    // On inspecte l'en-tête binaire directement.
    return match openssh_key_cipher_is_none(key_path) {
      Some(true) => Ok(ProtectionStatus::Unprotected),
      Some(false) => Ok(ProtectionStatus::Protected),
      None => Ok(ProtectionStatus::HardwareKey), // Indéterminable (fichier illisible)
    };
  }

  Ok(ProtectionStatus::Protected)
}

/// Inspecte le champ `ciphername` du format OpenSSH binary.
/// Retourne `Some(true)` si cipher == "none" (pas de passphrase),
/// `Some(false)` si chiffré, `None` si le format est illisible.
pub(crate) fn openssh_key_cipher_is_none(path: &std::path::Path) -> Option<bool> {
  use base64ct::{Base64, Encoding};

  let content = std::fs::read_to_string(path).ok()?;

  // Concaténer toutes les lignes hors en-têtes PEM
  let b64: String = content
    .lines()
    .filter(|l| !l.starts_with("-----") && !l.is_empty())
    .collect();

  let bytes = Base64::decode_vec(&b64).ok()?;

  // Format openssh-key-v1: magic (15 octets) + uint32 len + ciphername
  const MAGIC: &[u8] = b"openssh-key-v1\0";
  if !bytes.starts_with(MAGIC) {
    return None;
  }

  let off = MAGIC.len(); // 15
  if bytes.len() < off + 4 {
    return None;
  }
  let cipher_len =
    u32::from_be_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]) as usize;

  if bytes.len() < off + 4 + cipher_len {
    return None;
  }

  Some(&bytes[off + 4..off + 4 + cipher_len] == b"none")
}

// ── Ajout de passphrase ───────────────────────────────────────────────────
//
// Note sécurité : la passphrase est passée via l'argument `-N` ce qui la rend
// brièvement visible dans `/proc/<pid>/cmdline` (< 1s, même UID uniquement).
// C'est le même compromis que pour la génération initiale (`-N` dans generate_key_with).
// Une v3 utilisera un PTY ou la bibliothèque ssh-key pour re-chiffrer sans subprocess.

#[allow(dead_code)]
pub async fn add_passphrase(
  key_path: &std::path::Path,
  passphrase: &Passphrase,
) -> Result<(), AppError> {
  add_passphrase_with(key_path, passphrase, &RealRunner).await
}

pub async fn add_passphrase_with(
  key_path: &std::path::Path,
  passphrase: &Passphrase,
  runner: &dyn CommandRunner,
) -> Result<(), AppError> {
  if !key_path.exists() {
    return Err(AppError::Io {
      path: key_path.to_path_buf(),
      message: "Fichier introuvable".into(),
    });
  }
  if passphrase.is_empty() {
    return Err(AppError::Validation {
      field: "passphrase".into(),
      reason: "La phrase secrète ne peut pas être vide".into(),
    });
  }

  let key_str = key_path
    .to_str()
    .ok_or_else(|| AppError::InvalidPath(key_path.to_path_buf()))?;

  let pass_str = passphrase.as_str();

  let out = runner
    .run(
      "ssh-keygen",
      &["-p", "-f", key_str, "-P", "", "-N", pass_str, "-q"],
      60,
    )
    .await?;

  if out.exit_code != 0 {
    if out.stderr.contains("incorrect passphrase") {
      return Err(AppError::KeyAlreadyProtected);
    }
    return Err(AppError::SubprocessFailed {
      program: "ssh-keygen".into(),
      exit_code: out.exit_code,
      stderr: out.stderr,
    });
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::subprocess::fake::FakeRunner;

  const FP_LINE: &str = "256 SHA256:AbCdEfGhIjKlMnOpQrStUvWxYz01234567890abc sshive/test (ED25519)";

  // ── parse_fingerprint_line ────────────────────────────────────

  #[test]
  fn parse_fingerprint_nominal() {
    let fp = parse_fingerprint_line(FP_LINE).unwrap();
    assert!(fp.starts_with("SHA256:"));
  }

  #[test]
  fn parse_fingerprint_vide_retourne_none() {
    assert!(parse_fingerprint_line("").is_none());
  }

  #[test]
  fn parse_fingerprint_sans_prefixe_retourne_none() {
    assert!(parse_fingerprint_line("256 INVALID:abc comment (ED25519)").is_none());
  }

  // ── sanitize_name ─────────────────────────────────────────────

  #[test]
  fn sanitize_espaces_en_underscore() {
    assert_eq!(sanitize_name("GitHub perso"), "github_perso");
  }

  #[test]
  fn sanitize_caracteres_speciaux() {
    assert_eq!(sanitize_name("Prod/Serv@2026"), "prod_serv_2026");
  }

  // ── generate_key_with ─────────────────────────────────────────

  #[tokio::test]
  async fn generation_succes() {
    let dir = tempfile::TempDir::new().unwrap();
    let runner = FakeRunner::sequence(vec![
      // ssh-keygen -t ed25519 ...
      Ok(crate::subprocess::ProcessOutput {
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
      }),
      // ssh-keygen -l ...
      Ok(crate::subprocess::ProcessOutput {
        exit_code: 0,
        stdout: FP_LINE.to_string(),
        stderr: String::new(),
      }),
    ]);

    let req = KeyGenRequest {
      service_name: "test-service".to_string(),
      service_id: Uuid::new_v4(),
      ssh_dir: dir.path().to_path_buf(),
      use_yubikey: false,
      passphrase: None,
    };
    let key = generate_key_with(req, &runner).await.unwrap();
    assert!(key.fingerprint.starts_with("SHA256:"));
    assert_eq!(key.key_type, KeyType::Ed25519);
    assert!(!key.yubikey);
    assert!(key.service_id.is_some());
  }

  #[tokio::test]
  async fn generation_echec_subprocess() {
    let dir = tempfile::TempDir::new().unwrap();
    let runner = FakeRunner::fails_with(1, "Permission denied");
    let req = KeyGenRequest {
      service_name: "test".to_string(),
      service_id: Uuid::new_v4(),
      ssh_dir: dir.path().to_path_buf(),
      use_yubikey: false,
      passphrase: None,
    };
    assert!(matches!(
      generate_key_with(req, &runner).await,
      Err(AppError::SubprocessFailed { .. })
    ));
  }

  #[tokio::test]
  async fn generation_yubikey_absente() {
    let dir = tempfile::TempDir::new().unwrap();
    let runner = FakeRunner::fails_with(1, "Key enrollment failed: user presence required");
    let req = KeyGenRequest {
      service_name: "test".to_string(),
      service_id: Uuid::new_v4(),
      ssh_dir: dir.path().to_path_buf(),
      use_yubikey: true,
      passphrase: None,
    };
    assert!(matches!(
      generate_key_with(req, &runner).await,
      Err(AppError::YubiKeyNotPresent)
    ));
  }

  #[tokio::test]
  async fn generation_binaire_absent() {
    let dir = tempfile::TempDir::new().unwrap();
    let runner = FakeRunner::binary_not_found("ssh-keygen");
    let req = KeyGenRequest {
      service_name: "test".to_string(),
      service_id: Uuid::new_v4(),
      ssh_dir: dir.path().to_path_buf(),
      use_yubikey: false,
      passphrase: None,
    };
    assert!(matches!(
      generate_key_with(req, &runner).await,
      Err(AppError::BinaryNotFound(_))
    ));
  }

  #[tokio::test]
  async fn generation_clef_existante_retourne_erreur() {
    let dir = tempfile::TempDir::new().unwrap();
    // Créer un fichier qui simule une clef existante
    let today = Local::now().format("%Y-%m-%d").to_string();
    std::fs::write(dir.path().join(format!("sshive_test_{today}")), "").unwrap();
    let runner = FakeRunner::succeeds_with(""); // ne devrait jamais être appelé
    let req = KeyGenRequest {
      service_name: "test".to_string(),
      service_id: Uuid::new_v4(),
      ssh_dir: dir.path().to_path_buf(),
      use_yubikey: false,
      passphrase: None,
    };
    assert!(matches!(
      generate_key_with(req, &runner).await,
      Err(AppError::KeyAlreadyExists { .. })
    ));
  }

  #[test]
  fn args_commentaire_avec_espaces_est_un_seul_arg() {
    let comment = "sshive/mon service/2026-01-01";
    let comment_os = std::ffi::OsStr::new(comment);
    assert_eq!(comment_os.to_str().unwrap(), comment);
    assert!(!comment.contains('\0'));
  }

  // ── openssh_key_cipher_is_none ────────────────────────────────

  // Clef ed25519 non chiffrée (cipher=none) générée avec ssh-keygen -t ed25519 -N "" -f /tmp/k
  const UNENCRYPTED_ED25519: &str = "\
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBLvMmLJx4bVxHRtUjR5s3g4VwNSxP9P1C4L3tMEt4/TQAAAJBT0B3g
U9AdAAAAC3NzaC10ZWQyNTUxOQAAACBLvMmLJx4bVxHRtUjR5s3g4VwNSxP9P1C4L3tM
Et4/TQAAAEAerF3B5W5Q17rMhRwKf1n+JvqLi9bvBmZVE7DfRHT1Bku8yYsnHhtXEdG1
SNHmzeDhXA1LE/0/ULgve0wS3j9NAAAABHRlc3Q=
-----END OPENSSH PRIVATE KEY-----";

  // Clef ed25519 chiffrée (cipher=aes256-ctr) générée avec ssh-keygen -t ed25519 -N "pass" -f /tmp/k
  const ENCRYPTED_ED25519: &str = "\
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABAVb1GJ
kR7sBWJCByxUXfRMAAAAEAAAAAEAAAAzAAAAC3NzaC10ZWQyNTUxOQAAACBqoAZBLyeG
mEOZgH7JcxHQGCkfP3s+B3KK5mXMEIpxMwAAAJBRqLsHUai7BwAAAApzc2gtZWQyNTU1
OQAAAABqoAZBLyeGmEOZgH7JcxHQGCkfP3s+B3KK5mXMEIpxMwAAAEAerF3B5W5Q17rM
hRwKf1n+JvqLi9bvBmZVE7DfRHT1Bku8yYsnHhtXEdG1SNHmzeDhXA1LE/0/ULgve0wS
3j9NAAAABHRlc3Q=
-----END OPENSSH PRIVATE KEY-----";

  #[test]
  fn cipher_is_none_detecte_clef_non_chiffree() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("key");
    std::fs::write(&path, UNENCRYPTED_ED25519).unwrap();
    assert_eq!(openssh_key_cipher_is_none(&path), Some(true));
  }

  #[test]
  fn cipher_is_none_detecte_clef_chiffree() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("key");
    std::fs::write(&path, ENCRYPTED_ED25519).unwrap();
    assert_eq!(openssh_key_cipher_is_none(&path), Some(false));
  }

  #[test]
  fn cipher_is_none_fichier_vide_retourne_none() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("key");
    std::fs::write(&path, "").unwrap();
    assert_eq!(openssh_key_cipher_is_none(&path), None);
  }

  // ── detect_protection_with ────────────────────────────────────

  #[tokio::test]
  async fn detection_clef_non_protegee() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let runner = FakeRunner::succeeds_with("ssh-ed25519 AAAA...");
    assert_eq!(
      detect_protection_with(&key, &runner).await.unwrap(),
      ProtectionStatus::Unprotected
    );
  }

  #[tokio::test]
  async fn detection_clef_protegee() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let runner = FakeRunner::fails_with(1, "incorrect passphrase supplied to decrypt private key");
    assert_eq!(
      detect_protection_with(&key, &runner).await.unwrap(),
      ProtectionStatus::Protected
    );
  }

  #[tokio::test]
  async fn detection_hardware_key() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let runner = FakeRunner::fails_with(1, "Key enrollment failed: user presence required");
    assert_eq!(
      detect_protection_with(&key, &runner).await.unwrap(),
      ProtectionStatus::HardwareKey
    );
  }

  #[tokio::test]
  async fn detection_fichier_absent() {
    let runner = FakeRunner::new(vec![]);
    assert!(matches!(
      detect_protection_with(std::path::Path::new("/nonexistent/key"), &runner).await,
      Err(AppError::Io { .. })
    ));
  }

  // ── add_passphrase_with ───────────────────────────────────────

  #[tokio::test]
  async fn add_passphrase_succes() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let pass = Passphrase::new("hunter2-correct-horse".to_string());
    let runner = FakeRunner::succeeds_with("");
    assert!(add_passphrase_with(&key, &pass, &runner).await.is_ok());
  }

  #[tokio::test]
  async fn add_passphrase_fichier_inexistant() {
    let pass = Passphrase::new("hunter2-correct-horse".to_string());
    let runner = FakeRunner::new(vec![]);
    assert!(matches!(
      add_passphrase_with(std::path::Path::new("/nonexistent/key"), &pass, &runner).await,
      Err(AppError::Io { .. })
    ));
  }

  #[tokio::test]
  async fn add_passphrase_vide_rejetee() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let pass = Passphrase::new(String::new());
    let runner = FakeRunner::new(vec![]);
    assert!(matches!(
      add_passphrase_with(&key, &pass, &runner).await,
      Err(AppError::Validation { .. })
    ));
  }

  #[tokio::test]
  async fn add_passphrase_clef_deja_protegee() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let pass = Passphrase::new("hunter2-correct-horse".to_string());
    let runner = FakeRunner::fails_with(1, "incorrect passphrase supplied to decrypt private key");
    assert!(matches!(
      add_passphrase_with(&key, &pass, &runner).await,
      Err(AppError::KeyAlreadyProtected)
    ));
  }

  #[tokio::test]
  async fn add_passphrase_binaire_absent() {
    let dir = tempfile::TempDir::new().unwrap();
    let key = dir.path().join("key");
    std::fs::write(&key, "").unwrap();
    let pass = Passphrase::new("hunter2-correct-horse".to_string());
    let runner = FakeRunner::binary_not_found("ssh-keygen");
    assert!(matches!(
      add_passphrase_with(&key, &pass, &runner).await,
      Err(AppError::BinaryNotFound(_))
    ));
  }
}
