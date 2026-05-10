use std::collections::HashMap;

use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Element, Length};
use uuid::Uuid;

use crate::app::message::Message;
use crate::config::model::{Service, SshKey};
use crate::subprocess::ssh_keygen::ProtectionStatus;
use crate::ui::theme::{
  BACKGROUND_BASE, BACKGROUND_ELEVATED, BORDER_DEFAULT, BORDER_SUBTLE, DANGER_RED, DANGER_SUBTLE,
  FONT_MEDIUM, FONT_SEMIBOLD, TEXT_PRIMARY, TEXT_SECONDARY, WARNING_AMBER,
};
use crate::ui::widgets::{count_services_using_key, fingerprint_text, yubikey_badge};

pub fn view_key_list<'a>(
  keys: &'a [SshKey],
  all_services: &'a [Service],
  key_protection: &'a HashMap<Uuid, ProtectionStatus>,
  selected_id: Option<Uuid>,
) -> Element<'a, Message> {
  let header = view_header(keys.len());
  let table_header = view_table_header();

  let rows: Element<Message> = if keys.is_empty() {
    view_empty_keys()
  } else {
    scrollable(column(
      keys
        .iter()
        .map(|k| view_key_row(k, all_services, key_protection, selected_id))
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
      text("SSH Keys")
        .font(FONT_SEMIBOLD)
        .size(20)
        .color(TEXT_PRIMARY)
        .width(Length::Fill),
      text(format!("{count} clef{}", if count != 1 { "s" } else { "" }))
        .size(13)
        .color(TEXT_SECONDARY),
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
      text("FINGERPRINT")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(4)),
      text("TYPE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(100.0)),
      text("SOURCE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::Fixed(70.0)),
      text("COMMENTAIRE")
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY)
        .width(Length::FillPortion(3)),
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

fn view_key_row<'a>(
  key: &'a SshKey,
  all_services: &'a [Service],
  key_protection: &'a HashMap<Uuid, ProtectionStatus>,
  selected_id: Option<Uuid>,
) -> Element<'a, Message> {
  let is_selected = selected_id == Some(key.id);
  let usage_count = count_services_using_key(key.id, all_services);
  let (usage_label, usage_color) = match usage_count {
    0 => ("non utilisée".to_string(), TEXT_SECONDARY),
    1 => ("utilisée par 1 service".to_string(), TEXT_SECONDARY),
    n => (format!("utilisée par {n} services"), WARNING_AMBER),
  };

  let unprotected = key_protection.get(&key.id) == Some(&ProtectionStatus::Unprotected);

  let source: Element<Message> = if key.yubikey {
    yubikey_badge()
  } else if unprotected {
    container(
      text("⚠ non chiffrée")
        .size(10)
        .font(FONT_MEDIUM)
        .color(DANGER_RED),
    )
    .padding([2, 6])
    .style(|_: &iced::Theme| container::Style {
      background: Some(iced::Background::Color(DANGER_SUBTLE)),
      border: iced::Border {
        radius: 3.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
  } else {
    iced::widget::Space::new().width(0).into()
  };

  let key_id = key.id;

  iced::widget::button(
    container(
      row![
        column![
          fingerprint_text(&key.fingerprint),
          iced::widget::Space::new().height(2),
          text(usage_label).size(11).color(usage_color),
        ]
        .width(Length::FillPortion(4)),
        text(key.key_type.to_string())
          .size(11)
          .color(TEXT_SECONDARY)
          .width(Length::Fixed(100.0)),
        container(source).width(Length::Fixed(70.0)),
        text(&key.comment)
          .size(12)
          .color(TEXT_SECONDARY)
          .width(Length::FillPortion(3)),
      ]
      .align_y(Alignment::Center),
    )
    .padding([12, 24])
    .width(Length::Fill),
  )
  .on_press(Message::SelectKey(Some(key_id)))
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

fn view_empty_keys() -> Element<'static, Message> {
  container(
    column![
      text("Aucune clef SSH trouvée.")
        .size(14)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Les clefs publiques dans ~/.ssh/*.pub apparaîtront ici.")
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
