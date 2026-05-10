#![allow(dead_code)]
use crate::error::AppError;

pub fn validate_hostname(host: &str) -> Result<(), AppError> {
  let host = host.trim();
  if host.is_empty() {
    return err("host", "ne peut pas être vide");
  }
  if host.len() > 253 {
    return err("host", "trop long (max 253 caractères)");
  }
  if let Some(c) = host
    .chars()
    .find(|c| matches!(c, '$' | '`' | ';' | '|' | '&' | ' ' | '\n' | '\r' | '\0'))
  {
    return err("host", &format!("caractère interdit : {c:?}"));
  }
  if host.starts_with('-') {
    return err("host", "ne peut pas commencer par '-'");
  }
  Ok(())
}

pub fn validate_username(user: &str) -> Result<(), AppError> {
  let user = user.trim();
  if user.is_empty() {
    return err("user", "ne peut pas être vide");
  }
  if user.len() > 32 {
    return err("user", "trop long (max 32 caractères)");
  }
  if let Some(c) = user
    .chars()
    .find(|c| matches!(c, '$' | '`' | ';' | '|' | '&' | ' ' | '\n' | '\r' | '\0'))
  {
    return err("user", &format!("caractère interdit : {c:?}"));
  }
  Ok(())
}

pub fn validate_port(port_str: &str) -> Result<u16, AppError> {
  port_str
    .trim()
    .parse::<u16>()
    .ok()
    .filter(|&p| p >= 1)
    .ok_or_else(|| AppError::Validation {
      field: "port".to_string(),
      reason: "doit être un entier entre 1 et 65535".to_string(),
    })
}

pub fn validate_service_name(name: &str) -> Result<(), AppError> {
  let name = name.trim();
  if name.is_empty() {
    return err("name", "ne peut pas être vide");
  }
  if name.len() > 128 {
    return err("name", "trop long (max 128 caractères)");
  }
  Ok(())
}

fn err(field: &str, reason: &str) -> Result<(), AppError> {
  Err(AppError::Validation {
    field: field.to_string(),
    reason: reason.to_string(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  // ── hostname ──────────────────────────────────────────────────

  #[test]
  fn hostname_valide_fqdn() {
    assert!(validate_hostname("prod.example.com").is_ok());
  }

  #[test]
  fn hostname_valide_ip() {
    assert!(validate_hostname("192.168.1.1").is_ok());
  }

  #[test]
  fn hostname_vide() {
    assert!(validate_hostname("").is_err());
    assert!(validate_hostname("   ").is_err());
  }

  #[test]
  fn hostname_injection_dollar() {
    assert!(validate_hostname("host$(rm -rf ~)").is_err());
  }

  #[test]
  fn hostname_injection_pipe() {
    assert!(validate_hostname("host | cat /etc/passwd").is_err());
  }

  #[test]
  fn hostname_injection_newline() {
    assert!(validate_hostname("host\nmalicious").is_err());
  }

  #[test]
  fn hostname_injection_null() {
    assert!(validate_hostname("host\x00evil").is_err());
  }

  #[test]
  fn hostname_injection_backtick() {
    assert!(validate_hostname("host`id`").is_err());
  }

  #[test]
  fn hostname_commence_par_tiret() {
    assert!(validate_hostname("-evil").is_err());
  }

  #[test]
  fn hostname_trop_long() {
    assert!(validate_hostname(&"a".repeat(254)).is_err());
  }

  // ── port ──────────────────────────────────────────────────────

  #[test]
  fn port_22() {
    assert!(validate_port("22").is_ok());
    assert_eq!(validate_port("22").unwrap(), 22);
  }

  #[test]
  fn port_65535() {
    assert_eq!(validate_port("65535").unwrap(), 65535);
  }

  #[test]
  fn port_0() {
    assert!(validate_port("0").is_err());
  }

  #[test]
  fn port_65536() {
    assert!(validate_port("65536").is_err());
  }

  #[test]
  fn port_negatif() {
    assert!(validate_port("-1").is_err());
  }

  #[test]
  fn port_alpha() {
    assert!(validate_port("ssh").is_err());
  }

  #[test]
  fn port_vide() {
    assert!(validate_port("").is_err());
  }

  // ── username ──────────────────────────────────────────────────

  #[test]
  fn username_valide() {
    assert!(validate_username("deploy").is_ok());
  }

  #[test]
  fn username_injection_space() {
    assert!(validate_username("deploy user").is_err());
  }

  #[test]
  fn username_injection_semicolon() {
    assert!(validate_username("user;id").is_err());
  }

  #[test]
  fn username_vide() {
    assert!(validate_username("").is_err());
  }
}
