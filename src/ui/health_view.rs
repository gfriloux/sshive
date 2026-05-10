use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::ReadyState;
use crate::config::model::Service;
use crate::health::{HealthLevel, HealthReason, HealthSnapshot, ServiceHealth};
use crate::ui::theme::{
  ACCENT_SUBTLE, BACKGROUND_BASE, BACKGROUND_ELEVATED, BORDER_DEFAULT, BORDER_SUBTLE, DANGER_RED,
  DANGER_SUBTLE, FONT_MEDIUM, FONT_SEMIBOLD, SUCCESS_GREEN, TEXT_DISABLED, TEXT_PRIMARY,
  TEXT_SECONDARY, WARNING_AMBER, WARNING_SUBTLE,
};
use crate::ui::widgets::service_type_badge;

pub fn view_health<'a>(state: &'a ReadyState) -> Element<'a, Message> {
  let header = view_header(&state.health);
  let table_header = view_table_header();

  let rows: Element<Message> = if state.config.services.is_empty() {
    view_empty()
  } else {
    scrollable(column(
      state
        .config
        .services
        .iter()
        .map(|svc| {
          let health = state
            .health
            .services
            .get(&svc.id)
            .cloned()
            .unwrap_or_else(|| crate::health::ServiceHealth {
              level: HealthLevel::Ok,
              reasons: vec![],
              rotation_age_days: None,
            });
          view_service_row(svc, health, state.selected_service)
        })
        .collect::<Vec<_>>(),
    ))
    .into()
  };

  column![header, table_header, rows]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn view_header(health: &HealthSnapshot) -> Element<'static, Message> {
  let (critical, warning, ok) = health.counts();

  let summary: Element<Message> = if critical == 0 && warning == 0 {
    text(format!(
      "{ok} service{} — tout va bien",
      if ok != 1 { "s" } else { "" }
    ))
    .size(12)
    .color(SUCCESS_GREEN)
    .into()
  } else {
    row(
      [
        if critical > 0 {
          Some(
            text(format!(
              "{critical} critique{}",
              if critical != 1 { "s" } else { "" }
            ))
            .size(12)
            .color(DANGER_RED)
            .into(),
          )
        } else {
          None
        },
        if warning > 0 {
          Some(
            text(format!(
              "{warning} avertissement{}",
              if warning != 1 { "s" } else { "" }
            ))
            .size(12)
            .color(WARNING_AMBER)
            .into(),
          )
        } else {
          None
        },
        if ok > 0 {
          Some(
            text(format!("{ok} OK"))
              .size(12)
              .color(TEXT_SECONDARY)
              .into(),
          )
        } else {
          None
        },
      ]
      .into_iter()
      .flatten()
      .collect::<Vec<_>>(),
    )
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
  };

  container(
    row![
      text("Health")
        .font(FONT_SEMIBOLD)
        .size(20)
        .color(TEXT_PRIMARY)
        .width(Length::Fill),
      summary,
    ]
    .align_y(Alignment::Center),
  )
  .padding(iced::Padding {
    top: 24.0,
    right: 24.0,
    bottom: 16.0,
    left: 24.0,
  })
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_BASE)),
    ..Default::default()
  })
  .into()
}

fn view_table_header() -> Element<'static, Message> {
  container(
    row![
      text("ÉTAT")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(40.0)),
      text("SERVICE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(3)),
      text("TYPE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(80.0)),
      text("ÂGE ROTATION")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(120.0)),
      text("MOTIF")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(4)),
    ]
    .align_y(Alignment::Center),
  )
  .padding([10, 24])
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_ELEVATED)),
    border: Border {
      color: BORDER_DEFAULT,
      width: 2.0,
      radius: 0.0.into(),
    },
    ..Default::default()
  })
  .into()
}

fn level_indicator(level: &HealthLevel) -> Element<'static, Message> {
  let (symbol, color) = match level {
    HealthLevel::Critical => ("✕", DANGER_RED),
    HealthLevel::Warning => ("⚠", WARNING_AMBER),
    HealthLevel::Info => ("ℹ", TEXT_SECONDARY),
    HealthLevel::Ok => ("●", SUCCESS_GREEN),
  };
  text(symbol).size(14).color(color).into()
}

fn reason_text(reason: &HealthReason) -> &'static str {
  match reason {
    HealthReason::NoKey => "Aucune clef associée",
    HealthReason::KeyUnprotected { .. } => "Clef non chiffrée",
    HealthReason::RotationOverdue { .. } => "Rotation en retard",
    HealthReason::PendingDeployment => "Déploiement en attente",
  }
}

fn view_service_row<'a>(
  service: &'a Service,
  health: ServiceHealth,
  selected_id: Option<uuid::Uuid>,
) -> Element<'a, Message> {
  let is_selected = selected_id == Some(service.id);

  let age_str = match health.rotation_age_days {
    Some(d) => format!("{d}j"),
    None => "—".to_string(),
  };

  let age_color = match health.level {
    HealthLevel::Warning => WARNING_AMBER,
    _ => TEXT_SECONDARY,
  };

  let reasons_str = if health.reasons.is_empty() {
    "—".to_string()
  } else {
    health
      .reasons
      .iter()
      .map(reason_text)
      .collect::<Vec<_>>()
      .join(", ")
  };

  let reason_color = match health.level {
    HealthLevel::Critical => DANGER_RED,
    HealthLevel::Warning => WARNING_AMBER,
    HealthLevel::Info => TEXT_SECONDARY,
    HealthLevel::Ok => TEXT_DISABLED,
  };

  let row_bg = if is_selected {
    ACCENT_SUBTLE
  } else {
    match health.level {
      HealthLevel::Critical => DANGER_SUBTLE,
      HealthLevel::Warning => WARNING_SUBTLE,
      _ => BACKGROUND_BASE,
    }
  };

  let service_id = service.id;

  iced::widget::button(
    container(
      row![
        container(level_indicator(&health.level)).width(Length::Fixed(40.0)),
        text(&service.name)
          .size(13)
          .color(TEXT_PRIMARY)
          .width(Length::FillPortion(3)),
        container(service_type_badge(&service.service_type)).width(Length::Fixed(80.0)),
        text(age_str)
          .size(12)
          .color(age_color)
          .width(Length::Fixed(120.0)),
        text(reasons_str)
          .size(11)
          .color(reason_color)
          .width(Length::FillPortion(4)),
      ]
      .align_y(Alignment::Center),
    )
    .padding([12, 24])
    .width(Length::Fill),
  )
  .on_press(Message::SelectService(Some(service_id)))
  .style(move |_, status| iced::widget::button::Style {
    background: Some(Background::Color(if is_selected {
      row_bg
    } else if matches!(status, iced::widget::button::Status::Hovered) {
      crate::ui::theme::BACKGROUND_CARD
    } else {
      row_bg
    })),
    border: Border {
      color: BORDER_SUBTLE,
      width: 1.0,
      radius: 0.0.into(),
    },
    ..Default::default()
  })
  .width(Length::Fill)
  .into()
}

fn view_empty() -> Element<'static, Message> {
  container(
    column![
      text("Aucun service déclaré.").size(14).color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Créez un service pour voir son état de santé.")
        .size(12)
        .color(TEXT_SECONDARY),
    ]
    .align_x(Alignment::Center),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(iced::alignment::Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .into()
}
