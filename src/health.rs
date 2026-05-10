#![allow(dead_code)]

use std::collections::HashMap;

use uuid::Uuid;

use crate::config::model::{Config, SshKey};
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
  _all_keys: &[SshKey],
  protection: &HashMap<Uuid, ProtectionStatus>,
  rotation_warning_days: u32,
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
  let has_warning = reasons
    .iter()
    .any(|r| matches!(r, HealthReason::RotationOverdue { .. }));
  let has_info = reasons
    .iter()
    .any(|r| matches!(r, HealthReason::PendingDeployment));

  if has_critical {
    HealthLevel::Critical
  } else if has_warning {
    HealthLevel::Warning
  } else if has_info {
    HealthLevel::Info
  } else {
    HealthLevel::Ok
  }
}

impl HealthSnapshot {
  pub fn compute(
    config: &Config,
    _all_keys: &[SshKey],
    protection: &HashMap<Uuid, ProtectionStatus>,
  ) -> Self {
    let mut services = HashMap::new();
    for svc in &config.services {
      let health = compute_service_health(
        svc,
        _all_keys,
        protection,
        config.health.rotation_warning_days,
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

  #[test]
  fn health_sans_clef_est_critical() {
    let svc = make_service(None, None);
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90);
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::NoKey));
  }

  #[test]
  fn health_clef_non_protegee_est_critical() {
    let key_id = Uuid::new_v4();
    let svc = make_service(Some(key_id), None);
    let mut prot = HashMap::new();
    prot.insert(key_id, ProtectionStatus::Unprotected);
    let h = compute_service_health(&svc, &[], &prot, 90);
    assert_eq!(h.level, HealthLevel::Critical);
    assert!(h.reasons.contains(&HealthReason::KeyUnprotected { key_id }));
  }

  #[test]
  fn health_rotation_overdue_est_warning() {
    let key_id = Uuid::new_v4();
    let old_date = chrono::Local::now().date_naive() - chrono::Duration::days(91);
    let svc = make_service(Some(key_id), Some(old_date));
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90);
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
    let h = compute_service_health(&svc, &[], &HashMap::new(), 90);
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
    let h = compute_service_health(&svc, &[], &prot, 90);
    // KeyUnprotected (critical) + RotationOverdue (warning) + PendingDeployment (info)
    assert_eq!(h.level, HealthLevel::Critical); // critical domine
    assert_eq!(h.reasons.len(), 3);
  }

  #[test]
  fn health_seuil_rotation_configurable() {
    let key_id = Uuid::new_v4();
    let date_31j = chrono::Local::now().date_naive() - chrono::Duration::days(31);
    let svc = make_service(Some(key_id), Some(date_31j));
    // Avec seuil 90j : OK
    let h90 = compute_service_health(&svc, &[], &HashMap::new(), 90);
    assert_eq!(h90.level, HealthLevel::Ok);
    // Avec seuil 30j : Warning
    let h30 = compute_service_health(&svc, &[], &HashMap::new(), 30);
    assert_eq!(h30.level, HealthLevel::Warning);
  }

  #[test]
  fn snapshot_compute_peuple_tous_services() {
    let mut config = Config::default();
    config.services.push(make_service(None, None));
    config.services.push(make_service(None, None));
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new());
    assert_eq!(snap.services.len(), 2);
    assert!(snap.computed_at.is_some());
  }

  #[test]
  fn snapshot_counts() {
    let mut config = Config::default();
    let key_id = Uuid::new_v4();
    config.services.push(make_service(None, None)); // critical
    config.services.push(make_service(Some(key_id), None)); // ok (no protection data)
    let snap = HealthSnapshot::compute(&config, &[], &HashMap::new());
    let (critical, warning, ok) = snap.counts();
    assert_eq!(critical, 1);
    assert_eq!(warning, 0);
    assert_eq!(ok, 1);
  }
}
