use chrono::NaiveDate;
use iced::widget::{container, text};
use iced::{Background, Border, Element};

use uuid::Uuid;

use crate::app::message::Message;
use crate::config::model::{Service, ServiceType};
use crate::ui::theme::{
  FONT_MEDIUM, FONT_MONO, SUCCESS_GREEN, SUCCESS_SUBTLE, TEXT_SECONDARY, WARNING_AMBER,
  WARNING_SUBTLE,
};

// --- Fonctions pures (testables sans iced) ---

pub fn truncate_fp(fp: &str) -> String {
  if fp.len() > 23 {
    format!("{}…{}", &fp[..15], &fp[fp.len() - 8..])
  } else {
    fp.to_string()
  }
}

pub struct AgeIndicator {
  pub label: String,
  pub is_warning: bool,
}

pub fn age_indicator(last_rotation: Option<NaiveDate>) -> AgeIndicator {
  match last_rotation {
    None => AgeIndicator {
      label: "⚠ jamais".to_string(),
      is_warning: true,
    },
    Some(date) => {
      let today = chrono::Local::now().date_naive();
      let days = (today - date).num_days();
      if days >= 365 {
        let years = days / 365;
        AgeIndicator {
          label: format!("⚠ {} an{}", years, if years > 1 { "s" } else { "" }),
          is_warning: true,
        }
      } else {
        let months = days / 30;
        AgeIndicator {
          label: if months > 0 {
            format!("{} mois", months)
          } else {
            "< 1 mois".to_string()
          },
          is_warning: false,
        }
      }
    }
  }
}

pub fn count_services_using_key(key_id: Uuid, services: &[Service]) -> usize {
  services
    .iter()
    .filter(|s| s.active_key == Some(key_id))
    .count()
}

// --- Composants visuels ---

pub fn service_type_badge(service_type: &ServiceType) -> Element<'static, Message> {
  let (bg_r, bg_g, bg_b, fg_r, fg_g, fg_b, label) = match service_type {
    ServiceType::GitHub => (0.129, 0.149, 0.176, 0.788, 0.820, 0.851, "GH"),
    ServiceType::GitLab => (0.176, 0.106, 0.208, 0.988, 0.427, 0.149, "GL"),
    ServiceType::GitLabSelfHosted => (0.176, 0.106, 0.208, 0.988, 0.427, 0.149, "GL*"),
    ServiceType::SshGeneric => (0.051, 0.129, 0.216, 0.494, 0.722, 0.969, "SSH"),
    ServiceType::Manual => (0.102, 0.165, 0.102, 0.529, 0.937, 0.671, "M"),
  };
  let bg = iced::Color::from_rgb(bg_r, bg_g, bg_b);
  let fg = iced::Color::from_rgb(fg_r, fg_g, fg_b);

  container(text(label).size(11).font(FONT_MEDIUM).color(fg))
    .padding([3, 8])
    .style(move |_| container::Style {
      background: Some(Background::Color(bg)),
      border: Border {
        radius: 4.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
}

pub fn yubikey_badge() -> Element<'static, Message> {
  container(text("YK").size(11).font(FONT_MEDIUM).color(SUCCESS_GREEN))
    .padding([3, 6])
    .style(|_| container::Style {
      background: Some(Background::Color(SUCCESS_SUBTLE)),
      border: Border {
        radius: 4.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
}

pub fn shared_key_badge(count: usize) -> Element<'static, Message> {
  container(
    text(format!("×{count}"))
      .size(11)
      .font(FONT_MEDIUM)
      .color(WARNING_AMBER),
  )
  .padding([3, 6])
  .style(|_| container::Style {
    background: Some(Background::Color(WARNING_SUBTLE)),
    border: Border {
      radius: 4.0.into(),
      ..Default::default()
    },
    ..Default::default()
  })
  .into()
}

pub fn fingerprint_text(fp: &str) -> Element<'static, Message> {
  text(truncate_fp(fp))
    .size(11)
    .font(FONT_MONO)
    .color(TEXT_SECONDARY)
    .into()
}

// --- Tests ---

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::model::{DeployMode, KeyType, ServiceParams, ServiceType};

  fn make_service(name: &str, key_id: Option<Uuid>) -> Service {
    Service {
      id: Uuid::new_v4(),
      name: name.to_string(),
      service_type: ServiceType::Manual,
      params: ServiceParams::default(),
      active_key: key_id,
      pending_key: None,
      created_at: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
      last_rotation: None,
      deploy_mode: DeployMode::Automatic,
      deployments: vec![],
    }
  }

  #[test]
  fn truncate_fp_court_inchange() {
    assert_eq!(truncate_fp("SHA256:abc"), "SHA256:abc");
  }

  #[test]
  fn truncate_fp_long_ellipse_et_debut_preserve() {
    let fp = "SHA256:abcdefghijklmnopqrstuvwxyz123456";
    let result = truncate_fp(fp);
    assert!(result.len() < fp.len());
    assert!(result.contains('…'));
    assert!(result.starts_with(&fp[..15]));
  }

  #[test]
  fn age_indicator_jamais() {
    let ind = age_indicator(None);
    assert_eq!(ind.label, "⚠ jamais");
    assert!(ind.is_warning);
  }

  #[test]
  fn age_indicator_recent_neutre() {
    let recent = chrono::Local::now()
      .date_naive()
      .checked_sub_months(chrono::Months::new(6))
      .unwrap();
    let ind = age_indicator(Some(recent));
    assert!(!ind.is_warning);
    assert!(!ind.label.contains('⚠'));
  }

  #[test]
  fn age_indicator_vieux_warning() {
    let old = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let ind = age_indicator(Some(old));
    assert!(ind.is_warning);
    assert!(ind.label.starts_with('⚠'));
  }

  #[test]
  fn shared_key_count_exact() {
    let shared_id = Uuid::new_v4();
    let other_id = Uuid::new_v4();
    let services = vec![
      make_service("A", Some(shared_id)),
      make_service("B", Some(shared_id)),
      make_service("C", Some(other_id)),
    ];
    assert_eq!(count_services_using_key(shared_id, &services), 2);
    assert_eq!(count_services_using_key(other_id, &services), 1);
    assert_eq!(count_services_using_key(Uuid::new_v4(), &services), 0);
  }
}
