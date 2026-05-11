/// Journal d'audit local pour les opérations destructives.
/// Fichier append-only : ~/.config/sshive/audit.log (chmod 0600).
use std::io::Write;
use std::path::PathBuf;

#[allow(dead_code)]
pub enum AuditEvent<'a> {
  KeyGenerated {
    service: &'a str,
    fingerprint: &'a str,
    key_type: &'a str,
  },
  KeyDeployed {
    service: &'a str,
    fingerprint: &'a str,
    result: &'a str,
  },
  KeyRevoked {
    service: &'a str,
    fingerprint: &'a str,
    result: &'a str,
  },
  KeyRotated {
    service: &'a str,
    old_fp: &'a str,
    new_fp: &'a str,
    result: &'a str,
  },
  ServiceDeleted {
    service: &'a str,
  },
}

impl<'a> AuditEvent<'a> {
  fn to_line(&self) -> String {
    let ts = chrono::Local::now().format("%Y-%m-%dT%H:%M:%SZ");
    match self {
      AuditEvent::KeyGenerated {
        service,
        fingerprint,
        key_type,
      } => format!("{ts},generate,service={service},fingerprint={fingerprint},type={key_type}"),
      AuditEvent::KeyDeployed {
        service,
        fingerprint,
        result,
      } => format!("{ts},deploy,service={service},fingerprint={fingerprint},result={result}"),
      AuditEvent::KeyRevoked {
        service,
        fingerprint,
        result,
      } => format!("{ts},revoke,service={service},fingerprint={fingerprint},result={result}"),
      AuditEvent::KeyRotated {
        service,
        old_fp,
        new_fp,
        result,
      } => format!("{ts},rotate,service={service},old_fp={old_fp},new_fp={new_fp},result={result}"),
      AuditEvent::ServiceDeleted { service } => format!("{ts},delete_service,service={service}"),
    }
  }
}

fn audit_path() -> Option<PathBuf> {
  let dir = dirs::config_dir()?.join("sshive");
  Some(dir.join("audit.log"))
}

pub fn append(event: AuditEvent<'_>) {
  let Some(path) = audit_path() else { return };

  let line = event.to_line();

  let result = (|| -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
      .create(true)
      .append(true)
      .mode(0o600)
      .open(&path)?;
    writeln!(f, "{line}")
  })();

  if let Err(e) = result {
    tracing::warn!("Écriture journal d'audit échouée ({path:?}) : {e}");
  }
}
