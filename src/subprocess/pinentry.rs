#![allow(dead_code)]

use std::path::PathBuf;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;
use zeroize::Zeroize;

use crate::error::AppError;

// ── Passphrase — buffer zéroïsé au drop ───────────────────────────────────

pub struct Passphrase(Vec<u8>);

impl Passphrase {
  pub fn new(s: String) -> Self {
    Self(s.into_bytes())
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.0
  }

  pub fn as_str(&self) -> &str {
    std::str::from_utf8(&self.0).unwrap_or("")
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }
}

impl Drop for Passphrase {
  fn drop(&mut self) {
    self.0.zeroize();
  }
}

// ── Trait PinentryRunner ───────────────────────────────────────────────────

#[async_trait]
pub trait PinentryRunner: Send + Sync {
  async fn collect(&self, title: &str, desc: &str, prompt: &str) -> Result<Passphrase, AppError>;
}

// ── Implémentation réelle ──────────────────────────────────────────────────

pub struct RealPinentryRunner;

#[async_trait]
impl PinentryRunner for RealPinentryRunner {
  async fn collect(&self, title: &str, desc: &str, prompt: &str) -> Result<Passphrase, AppError> {
    let binary = resolve_pinentry_binary()?;

    let mut child = Command::new(&binary)
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::null())
      .kill_on_drop(true)
      .spawn()
      .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
          AppError::BinaryNotFound(binary.to_string_lossy().into_owned())
        } else {
          AppError::SubprocessSpawn {
            program: binary.to_string_lossy().into_owned(),
            message: e.to_string(),
          }
        }
      })?;

    let stdin = child.stdin.take().expect("stdin piped");
    let stdout = child.stdout.take().expect("stdout piped");
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    macro_rules! read_ok {
      () => {{
        line.clear();
        reader
          .read_line(&mut line)
          .await
          .map_err(|e| AppError::PinentryProtocolError(format!("lecture : {e}")))?;
        if line.starts_with("ERR") {
          return Err(parse_assuan_error(&line));
        }
        if !line.starts_with("OK") {
          return Err(AppError::PinentryProtocolError(line.trim().to_string()));
        }
      }};
    }

    macro_rules! send {
      ($cmd:expr) => {{
        writer
          .write_all($cmd.as_bytes())
          .await
          .map_err(|e| AppError::PinentryProtocolError(format!("écriture : {e}")))?;
        writer
          .write_all(b"\n")
          .await
          .map_err(|e| AppError::PinentryProtocolError(format!("écriture newline : {e}")))?;
        writer
          .flush()
          .await
          .map_err(|e| AppError::PinentryProtocolError(format!("flush : {e}")))?;
      }};
    }

    read_ok!(); // banner OK

    send!(format!("SETTITLE {}", assuan_escape(title)));
    read_ok!();
    send!(format!("SETDESC {}", assuan_escape(desc)));
    read_ok!();
    send!(format!("SETPROMPT {}", assuan_escape(prompt)));
    read_ok!();
    send!("SETOK Valider".to_string());
    read_ok!();
    send!("SETCANCEL Annuler".to_string());
    read_ok!();

    send!("GETPIN".to_string());

    // Lire la réponse : "D <passphrase>" puis "OK", ou "ERR ..."
    line.clear();
    reader
      .read_line(&mut line)
      .await
      .map_err(|e| AppError::PinentryProtocolError(format!("lecture GETPIN : {e}")))?;

    let passphrase = if line.starts_with("ERR") {
      let _ = writer.write_all(b"BYE\n").await;
      let _ = writer.flush().await;
      let _ = child.wait().await;
      return Err(parse_assuan_error(&line));
    } else if let Some(pin) = parse_assuan_d_line(&line) {
      // Lire le "OK" suivant
      line.clear();
      reader.read_line(&mut line).await.ok();
      Passphrase::new(pin)
    } else {
      // "OK" direct sans ligne D → passphrase vide (annulation silencieuse)
      let _ = writer.write_all(b"BYE\n").await;
      let _ = writer.flush().await;
      let _ = child.wait().await;
      return Err(AppError::PinentryUserCancelled);
    };

    let _ = writer.write_all(b"BYE\n").await;
    let _ = writer.flush().await;
    let _ = child.wait().await;

    Ok(passphrase)
  }
}

// ── API publique ───────────────────────────────────────────────────────────

pub async fn collect_passphrase(title: &str, desc: &str) -> Result<Passphrase, AppError> {
  collect_passphrase_with(title, desc, &RealPinentryRunner).await
}

pub async fn collect_passphrase_with(
  title: &str,
  desc: &str,
  runner: &dyn PinentryRunner,
) -> Result<Passphrase, AppError> {
  let pass = runner.collect(title, desc, "Phrase secrète :").await?;
  if pass.is_empty() {
    return Err(AppError::Validation {
      field: "passphrase".into(),
      reason: "La phrase secrète ne peut pas être vide".into(),
    });
  }
  Ok(pass)
}

// ── Fonctions utilitaires (pub(crate) pour les tests) ─────────────────────

/// Encode les caractères spéciaux pour le protocole Assuan.
pub(crate) fn assuan_escape(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for ch in s.chars() {
    match ch {
      '%' => out.push_str("%25"),
      '\n' => out.push_str("%0A"),
      '\r' => out.push_str("%0D"),
      c => out.push(c),
    }
  }
  out
}

/// Parse la ligne "D <passphrase>" en retournant la passphrase décodée.
pub(crate) fn parse_assuan_d_line(line: &str) -> Option<String> {
  let rest = line.strip_prefix("D ")?;
  let raw = rest.trim_end_matches('\n').trim_end_matches('\r');
  if raw.is_empty() {
    return None;
  }
  Some(assuan_unescape(raw))
}

/// Retourne true si la ligne Assuan indique une annulation utilisateur.
pub(crate) fn is_assuan_cancelled(line: &str) -> bool {
  // Code GPG-AGENT 83886179 (0x5000073) = Operation cancelled
  line.contains("83886179") || line.to_lowercase().contains("cancelled")
}

fn assuan_unescape(s: &str) -> String {
  let bytes = s.as_bytes();
  let mut out: Vec<u8> = Vec::with_capacity(s.len());
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'%' && i + 2 < bytes.len() {
      if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
        if let Ok(b) = u8::from_str_radix(hex, 16) {
          out.push(b);
          i += 3;
          continue;
        }
      }
    }
    out.push(bytes[i]);
    i += 1;
  }
  String::from_utf8_lossy(&out).into_owned()
}

fn parse_assuan_error(line: &str) -> AppError {
  if is_assuan_cancelled(line) {
    AppError::PinentryUserCancelled
  } else {
    AppError::PinentryProtocolError(line.trim().to_string())
  }
}

/// Résout le binaire pinentry à utiliser selon l'environnement desktop.
fn resolve_pinentry_binary() -> Result<PathBuf, AppError> {
  if let Ok(custom) = std::env::var("SSHIVE_PINENTRY") {
    return Ok(PathBuf::from(custom));
  }

  let desktop = std::env::var("XDG_CURRENT_DESKTOP")
    .unwrap_or_default()
    .to_lowercase();

  let candidates: &[&str] =
    if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("pantheon") {
      &[
        "pinentry-gnome3",
        "pinentry-gtk-2",
        "pinentry-qt",
        "pinentry",
      ]
    } else if desktop.contains("kde") || desktop.contains("lxqt") {
      &[
        "pinentry-qt",
        "pinentry-gtk-2",
        "pinentry-gnome3",
        "pinentry",
      ]
    } else {
      &[
        "pinentry-gtk-2",
        "pinentry-gnome3",
        "pinentry-qt",
        "pinentry",
      ]
    };

  for &name in candidates {
    if which_binary(name) {
      return Ok(PathBuf::from(name));
    }
  }

  Err(AppError::PinentryBackendUnavailable)
}

fn which_binary(name: &str) -> bool {
  std::env::var_os("PATH")
    .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
    .unwrap_or(false)
}

// ── Fake pour les tests ────────────────────────────────────────────────────

#[cfg(test)]
pub mod fake {
  use async_trait::async_trait;

  use super::{Passphrase, PinentryRunner};
  use crate::error::AppError;

  pub struct FakePinentryRunner {
    response: Result<String, AppError>,
  }

  impl FakePinentryRunner {
    pub fn succeeds_with(passphrase: &str) -> Self {
      Self {
        response: Ok(passphrase.to_string()),
      }
    }

    pub fn cancelled() -> Self {
      Self {
        response: Err(AppError::PinentryUserCancelled),
      }
    }

    pub fn binary_not_found() -> Self {
      Self {
        response: Err(AppError::BinaryNotFound("pinentry".to_string())),
      }
    }

    pub fn timed_out() -> Self {
      Self {
        response: Err(AppError::SubprocessTimeout {
          program: "pinentry".to_string(),
          timeout_secs: 30,
        }),
      }
    }
  }

  #[async_trait]
  impl PinentryRunner for FakePinentryRunner {
    async fn collect(
      &self,
      _title: &str,
      _desc: &str,
      _prompt: &str,
    ) -> Result<Passphrase, AppError> {
      match &self.response {
        Ok(p) => Ok(Passphrase::new(p.clone())),
        Err(e) => Err(e.clone()),
      }
    }
  }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use fake::FakePinentryRunner;

  #[tokio::test]
  async fn pinentry_retourne_passphrase() {
    let runner = FakePinentryRunner::succeeds_with("correct-horse-battery");
    let pass = collect_passphrase_with("SSHive", "Test", &runner)
      .await
      .unwrap();
    assert_eq!(pass.as_str(), "correct-horse-battery");
  }

  #[tokio::test]
  async fn pinentry_passphrase_vide_rejetee() {
    let runner = FakePinentryRunner::succeeds_with("");
    assert!(matches!(
      collect_passphrase_with("T", "D", &runner).await,
      Err(AppError::Validation { .. })
    ));
  }

  #[tokio::test]
  async fn pinentry_annulation_retourne_cancelled() {
    let runner = FakePinentryRunner::cancelled();
    assert!(matches!(
      collect_passphrase_with("T", "D", &runner).await,
      Err(AppError::PinentryUserCancelled)
    ));
  }

  #[tokio::test]
  async fn pinentry_binaire_absent() {
    let runner = FakePinentryRunner::binary_not_found();
    assert!(matches!(
      collect_passphrase_with("T", "D", &runner).await,
      Err(AppError::BinaryNotFound(_))
    ));
  }

  #[tokio::test]
  async fn pinentry_timeout() {
    let runner = FakePinentryRunner::timed_out();
    assert!(matches!(
      collect_passphrase_with("T", "D", &runner).await,
      Err(AppError::SubprocessTimeout { .. })
    ));
  }

  #[test]
  fn assuan_escape_newline() {
    assert_eq!(assuan_escape("a\nb"), "a%0Ab");
  }

  #[test]
  fn assuan_escape_percent() {
    assert_eq!(assuan_escape("a%b"), "a%25b");
  }

  #[test]
  fn assuan_escape_crlf() {
    assert_eq!(assuan_escape("a\r\nb"), "a%0D%0Ab");
  }

  #[test]
  fn assuan_roundtrip() {
    let original = "my p@ss\nwörd%!";
    let escaped = assuan_escape(original);
    let unescaped = assuan_unescape(&escaped);
    assert_eq!(unescaped, original);
  }

  #[test]
  fn parse_d_line_nominal() {
    assert_eq!(
      parse_assuan_d_line("D correct-horse-battery\n"),
      Some("correct-horse-battery".to_string())
    );
  }

  #[test]
  fn parse_d_line_vide_retourne_none() {
    assert_eq!(parse_assuan_d_line("D \n"), None);
  }

  #[test]
  fn parse_d_line_encoded() {
    assert_eq!(
      parse_assuan_d_line("D pass%0Aword\n"),
      Some("pass\nword".to_string())
    );
  }

  #[test]
  fn is_cancelled_code_gpg() {
    assert!(is_assuan_cancelled("ERR 83886179 Operation cancelled"));
    assert!(!is_assuan_cancelled("ERR 67108949 Bad passphrase"));
  }
}
