use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::config::model::{DeployMode, Service, SshKey};
use crate::ui::theme::{
  BACKGROUND_BASE, BACKGROUND_ELEVATED, BACKGROUND_SUBTLE, BORDER_DEFAULT, BORDER_SUBTLE,
  FONT_MEDIUM, FONT_SEMIBOLD, TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY, WARNING_AMBER,
};
use crate::ui::widgets::{
  age_indicator, count_services_using_key, fingerprint_text, service_type_badge, shared_key_badge,
  yubikey_badge,
};

pub fn view_service_list<'a>(
  services: &'a [Service],
  all_keys: &'a [SshKey],
  selected_id: Option<uuid::Uuid>,
) -> Element<'a, Message> {
  let header = view_header(services.len());
  let table_header = view_table_header();

  let rows: Element<Message> = if services.is_empty() {
    view_empty_services()
  } else {
    scrollable(column(
      services
        .iter()
        .map(|s| view_service_row(s, services, all_keys, selected_id))
        .collect::<Vec<_>>(),
    ))
    .into()
  };

  column![header, table_header, rows]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn view_header(count: usize) -> Element<'static, Message> {
  container(
    row![
      text("Services")
        .font(FONT_SEMIBOLD)
        .size(20)
        .color(TEXT_PRIMARY)
        .width(Length::Fill),
      text(format!(
        "{count} service{}",
        if count != 1 { "s" } else { "" }
      ))
      .size(13)
      .color(TEXT_SECONDARY),
      iced::widget::Space::new().width(12),
      iced::widget::button(
        text("+ Ajouter")
          .size(12)
          .color(crate::ui::theme::TEXT_PRIMARY),
      )
      .on_press(Message::OpenCreateService)
      .style(|_, _| iced::widget::button::Style {
        background: Some(Background::Color(crate::ui::theme::ACCENT_PRIMARY)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      })
      .padding([6, 12]),
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
    border: Border {
      color: BORDER_DEFAULT,
      width: 0.0,
      radius: 0.0.into(),
    },
    ..Default::default()
  })
  .into()
}

fn view_table_header() -> Element<'static, Message> {
  container(
    row![
      text("SERVICE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(3)),
      text("TYPE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(90.0)),
      text("FINGERPRINT")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(4)),
      text("ROTATION")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(120.0)),
      text("BADGES")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(80.0)),
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

fn view_service_row<'a>(
  service: &'a Service,
  all_services: &'a [Service],
  all_keys: &'a [SshKey],
  selected_id: Option<uuid::Uuid>,
) -> Element<'a, Message> {
  let is_selected = selected_id == Some(service.id);
  let age = age_indicator(service.last_rotation);
  let age_color = if age.is_warning {
    WARNING_AMBER
  } else {
    TEXT_SECONDARY
  };

  // Résoudre la clef active par UUID
  let active_key = service
    .active_key
    .and_then(|id| all_keys.iter().find(|k| k.id == id));

  let fp_display: Element<Message> = match active_key {
    Some(key) => fingerprint_text(&key.fingerprint),
    None => text("—").size(11).color(TEXT_DISABLED).into(),
  };

  let shared_count = service
    .active_key
    .map(|id| count_services_using_key(id, all_services))
    .unwrap_or(0);

  let has_yubikey = active_key.is_some_and(|k| k.yubikey);

  let yk: Element<Message> = if has_yubikey {
    yubikey_badge()
  } else {
    iced::widget::Space::new().width(0).into()
  };
  let shared: Element<Message> = if shared_count > 1 {
    row![
      iced::widget::Space::new().width(4),
      shared_key_badge(shared_count),
    ]
    .into()
  } else {
    iced::widget::Space::new().width(0).into()
  };
  let ext: Element<Message> = if service.deploy_mode == DeployMode::ExternalCm {
    row![
      iced::widget::Space::new().width(4),
      container(text("EXT").size(10).font(FONT_MEDIUM).color(TEXT_DISABLED),)
        .padding([2, 5])
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(BACKGROUND_SUBTLE)),
          border: Border {
            radius: 3.0.into(),
            ..Default::default()
          },
          ..Default::default()
        }),
    ]
    .into()
  } else {
    iced::widget::Space::new().width(0).into()
  };
  let badges = row![yk, shared, ext].align_y(Alignment::Center);

  let service_id = service.id;

  iced::widget::button(
    container(
      row![
        text(&service.name)
          .size(13)
          .color(TEXT_PRIMARY)
          .width(Length::FillPortion(3)),
        container(service_type_badge(&service.service_type)).width(Length::Fixed(90.0)),
        container(fp_display).width(Length::FillPortion(4)),
        text(age.label)
          .size(12)
          .color(age_color)
          .width(Length::Fixed(120.0)),
        container(badges).width(Length::Fixed(80.0)),
      ]
      .align_y(Alignment::Center),
    )
    .padding([12, 24])
    .width(Length::Fill),
  )
  .on_press(Message::SelectService(Some(service_id)))
  .style(move |_, status| iced::widget::button::Style {
    background: Some(Background::Color(if is_selected {
      crate::ui::theme::ACCENT_SUBTLE
    } else if matches!(status, iced::widget::button::Status::Hovered) {
      crate::ui::theme::BACKGROUND_CARD
    } else {
      crate::ui::theme::BACKGROUND_BASE
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

fn view_empty_services() -> Element<'static, Message> {
  container(
    column![
      text("Aucun service déclaré.").size(14).color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Éditez ~/.config/sshive/config.yaml pour commencer.")
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
