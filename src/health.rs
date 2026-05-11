#![allow(dead_code)]

use std::collections::HashMap;

use uuid::Uuid;

use crate::config::model::{Config, ServiceType, SshKey};
use crate::secrets::model::Secrets;
use crate::subprocess::ssh_keygen::ProtectionStatus;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthLevel {
  Ok,
  Info,
  Warning,
  Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthReason {
  NoKey,
  KeyUnprotected { key_id: Uuid },
  RotationOverdue { days_overdue: i64 },
  PendingDeployment,
  NoApiToken,
  HardwareKeyHandleNotBackedUp { key_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct ServiceHealth {
  pub level: HealthLevel,
  pub reasons: Vec<HealthReason>,
  pub rotation_age_days: Option<i64>,
}

impl ServiceHealth {
  fn ok() -> Self {
    Self {
      level: HealthLevel::Ok,
      reasons: vec![],
      rotation_age_days: None,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct HealthSnapshot {
  pub services: HashMap<Uuid, ServiceHealth>,
  pub computed_at: Option<chrono::DateTime<chrono::Local>>,
}

// ── Calcul ────────────────────────────────────────────────────────────────

pub fn compute_service_health(
  service: &crate::config::model::Service,
  all_keys: &[SshKey],
  protection: &HashMap<Uuid, ProtectionStatus>,
  rotation_warning_days: u32,
  has_api_token: Option<bool>,
) -> ServiceHealth {
  let mut reasons = Vec::new();

  // Clef active
  match service.active_key {
    None => {
      reasons.push(HealthReason::NoKey);
    }
    Some(key_id) => {
      if let Some(ProtectionStatus::Unprotected) = protection.get(&key_id) {
        reasons.push(HealthReason::KeyUnprotected { key_id });
      }
      // Clef SK sans backup de handle
      let active_key = all_keys.iter().find(|k| k.id == key_id);
      if let Some(key) = active_key {
        if key.yubikey && !key.backup_prompted {
          reasons.push(HealthReason::HardwareKeyHandleNotBackedUp { key_id });
        }
      }
    }
  }

  // Rotation
  let rotation_age_days = service
    .last_rotation
    .map(|d| (chrono::Local::now().date_naive() - d).num_days());

  if let Some(age) = rotation_age_days {
    let threshold = rotation_warning_days as i64;
    if age > threshold {
      reasons.push(HealthReason::RotationOverdue {
        days_overdue: age - threshold,
      });
    }
  }

  // Déploiement en attente
  if service.pending_key.is_some() {
    reasons.push(HealthReason::PendingDeployment);
  }

  // Token API manquant (Some(false) = service API sans token ; None = non-API)
  if has_api_token == Some(false) {
    reasons.push(HealthReason::NoApiToken);
  }

  let level = derive_level(&reasons);
  ServiceHealth {
    level,
    reasons,
    rotation_age_days,
  }
}

fn derive_level(reasons: &[HealthReason]) -> HealthLevel {
  let has_critical = reasons
    .iter()
    .any(|r| matches!(r, HealthReason::NoKey | HealthReason::KeyUnprotected { .. }));
  let has_rotation_overdue = reasons
    .iter()
    .any(|r| matches!(r, HealthReason::RotationOverdue { .. }));
  let has_no_api_token = reasons
    .iter()
    .any(|r| matches!(r, HealthReason::NoApiToken));
  let has_info = reasons.iter().any(|r| {
    matches!(
      r,
      HealthReason::PendingDeployment | HealthReason::HardwareKeyHandleNotBackedUp { .. }
    )
  });

  // NoApiToken + RotationOverdue → Critical : l'utilisateur ne peut pas corriger la rotation
  // sans d'abord configurer le token.
  if has_critical || (has_no_api_token && has_rotation_overdue) {
    HealthLevel::Critical
  } else if has_rotation_overdue || has_no_api_token {
    HealthLevel::Warning
  } else if has_info {
    HealthLevel::Info
  } else {
    HealthLevel::Ok
  }
}

fn service_needs_api_token(service_type: &ServiceType) -> bool {
  matches!(
    service_type,
    ServiceType::GitHub | ServiceType::GitLab | ServiceType::GitLabSelfHosted
  )
}

impl HealthSnapshot {
  pub fn compute(
    config: &Config,
    _all_keys: &[SshKey],
    protection: &HashMap<Uuid, ProtectionStatus>,
    secrets: Option<&Secrets>,
  ) -> Self {
    let mut services = HashMap::new();
    for svc in &config.services {
      let has_api_token = if service_needs_api_token(&svc.service_type) {
        secrets.map(|s| s.token_for(svc).is_some())
      } else {
        None
      };

      let health = compute_service_health(
        svc,
        _all_keys,
        protection,
        config.health.rotation_warning_days,
        has_api_token,
      );
      services.insert(svc.id, health);
    }
    Self {
      services,
      computed_at: Some(chrono::Local::now()),
    }
  }

  /// Nombre de services par niveau.
  pub fn counts(&self) -> (usize, usize, usize) {
    let critical = self
      .services
      .values()
      .filter(|h| h.level == HealthLevel::Critical)
      .count();
    let warning = self
      .services
      .values()
      .filter(|h| h.level == HealthLevel::Warning)
      .count();
    let ok = self
      .services
      .values()
      .filter(|h| matches!(h.level, HealthLevel::Ok | HealthLevel::Info))
      .count();
    (critical, warning, ok)
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use chrono::NaiveDate;

  use super::*;
  use crate::config::model::{DeployMode, Service, ServiceParams, ServiceType};

  fn make_service(active_key: Option<Uuid>, last_rotation: Option<NaiveDate>) -> Service {
    Service {
      id: Uuid::new_v4(),
      name: "Test".into(),
      service_type: ServiceType::SshGeneric,
      params: ServiceParams::default(),
      active_key,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    }
  }

  fn make_github_service(active_key: Option<Uuid>) -> Service {
    Service {
      id: Uuid::new_v4(),
      name: "GitHub perso".into(),
      service_type: ServiceType::GitHub,
      params: ServiceParams {
        token_ref: Some("github_perso".into()),
        ..ServiceParams::default()
      },
      active_key,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    }
  }

  #[test]
  fn health_sans_clef_est_critical() {
    let svc = make_service(None, None);
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::NoKey));
  }

  #[test]
  fn health_clef_non_protegee_est_critical() {
    let key_id = Uuid::new_v4();
    let svc = make_service(Some(key_id), None);
    let mut prot = HashMap::new();
    prot.insert(key_id, ProtectionStatus::Unprotected);
    let h = compute_service_health(&svc, &[], &prot, 90, None);
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::KeyUnprotected { key_id }));
  }

  #[test]
  fn health_rotation_overdue_est_warning() {
    let key_id = Uuid::new_v4();
    let old_date = chrono::Local::now().date_naive() - chrono::Duration::days(91);
    let svc = make_service(Some(key_id), Some(old_date));
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Warning);
    assert!(matches!(
      h.reasons[0],
      HealthReason::RotationOverdue { days_overdue: 1 }
    ));
  }

  #[test]
  fn health_rotation_ok() {
    let key_id = Uuid::new_v4();
    let recent = chrono::Local::now().date_naive() - chrono::Duration::days(45);
    let svc = make_service(Some(key_id), Some(recent));
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Ok);
    assert!(h.reasons.is_empty());
  }

  #[test]
  fn health_plusieurs_raisons_accumulees() {
    let key_id = Uuid::new_v4();
    let old_date = chrono::Local::now().date_naive() - chrono::Duration::days(100);
    let mut svc = make_service(Some(key_id), Some(old_date));
    svc.pending_key = Some(Uuid::new_v4());
    let mut prot = HashMap::new();
    prot.insert(key_id, ProtectionStatus::Unprotected);
    let h = compute_service_health(&svc, &[], &prot, 90, None);
    // KeyUnprotected (critical) + RotationOverdue (warning) + PendingDeployment (info)
    assert_eq!(h.level, HealthLevel::Critical); // critical domine
    assert_eq!(h.reasons.len(), 3);
  }

  #[test]
  fn health_seuil_rotation_configurable() {
    let key_id = Uuid::new_v4();
    let date_31j = chrono::Local::now().date_naive() - chrono::Duration::days(31);
    let svc = make_service(Some(key_id), Some(date_31j));
    let h90 = compute_service_health(&svc, &[], &HashMap::new(), 90, None);
    assert_eq!(h90.level, HealthLevel::Ok);
    let h30 = compute_service_health(&svc, &[], &HashMap::new(), 30, None);
    assert_eq!(h30.level, HealthLevel::Warning);
  }

  #[test]
  fn snapshot_compute_peuple_tous_services() {
    let mut config = Config::default();
    config.services.push(make_service(None, None));
    config.services.push(make_service(None, None));
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), None);
    assert_eq!(snap.services.len(), 2);
    assert!(snap.computed_at.is_some());
  }

  #[test]
  fn snapshot_counts() {
    let mut config = Config::default();
    let key_id = Uuid::new_v4();
    config.services.push(make_service(None, None)); // critical
    config.services.push(make_service(Some(key_id), None)); // ok (no protection data)
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), None);
    let (critical, warning, ok) = snap.counts();
    assert_eq!(critical, 1);
    assert_eq!(warning, 0);
    assert_eq!(ok, 1);
  }

  // ── Tests NoApiToken ──────────────────────────────────────────────────────

  #[test]
  fn health_no_api_token_est_warning() {
    let key_id = Uuid::new_v4();
    let svc = make_github_service(Some(key_id));
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, Some(false));
    assert_eq!(h.level, HealthLevel::Warning);
    assert!(h.reasons.contains(&HealthReason::NoApiToken));
  }

  #[test]
  fn health_no_api_token_no_key_est_critical() {
    // NoKey est déjà Critical — NoApiToken s'accumule mais ne change pas le niveau
    let svc = make_github_service(None);
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, Some(false));
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::NoKey));
    assert!(h.reasons.contains(&HealthReason::NoApiToken));
  }

  #[test]
  fn health_no_api_token_plus_rotation_overdue_est_critical() {
    let key_id = Uuid::new_v4();
    let old_date = chrono::Local::now().date_naive() - chrono::Duration::days(91);
    let mut svc = make_github_service(Some(key_id));
    svc.last_rotation = Some(old_date);
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, Some(false));
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::NoApiToken));
    assert!(h
      .reasons
      .iter()
      .any(|r| matches!(r, HealthReason::RotationOverdue { .. })));
  }

  #[test]
  fn health_token_present_pas_de_no_api_token() {
    let key_id = Uuid::new_v4();
    let svc = make_github_service(Some(key_id));
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, Some(true));
    assert_eq!(h.level, HealthLevel::Ok);
    assert!(!h.reasons.contains(&HealthReason::NoApiToken));
  }

  #[test]
  fn health_service_non_api_ignore_token_check() {
    // SshGeneric avec has_api_token=None → pas de NoApiToken
    let svc = make_service(Some(Uuid::new_v4()), None);
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90, None);
    assert_eq!(h.level, HealthLevel::Ok);
    assert!(!h.reasons.contains(&HealthReason::NoApiToken));
  }

  #[test]
  fn health_vault_locked_supprime_no_api_token() {
    // secrets = None → vault verrouillé → has_api_token = None → pas de NoApiToken
    let key_id = Uuid::new_v4();
    let mut config = Config::default();
    let mut svc = make_github_service(Some(key_id));
    svc.params.token_ref = Some("github_perso".into());
    config.services.push(svc);
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new(), None);
    let health = snap.services.values().next().unwrap();
    assert!(!health.reasons.contains(&HealthReason::NoApiToken));
  }
}
