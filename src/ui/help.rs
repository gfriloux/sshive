#![allow(dead_code)]
#![allow(unused_imports)]
use std::collections::HashSet;

use iced::widget::{column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::app::message::{HelpId, Message};
use crate::ui::theme::{
  ACCENT_PRIMARY, BACKGROUND_CARD, BORDER_DEFAULT, BORDER_SUBTLE, FONT_MEDIUM, TEXT_ACCENT,
  TEXT_SECONDARY,
};

/// Lien "? Libellé" qui envoie ToggleHelp(id) au clic.
pub fn help_toggle<'a>(id: HelpId, label: &'a str, open: &HashSet<HelpId>) -> Element<'a, Message> {
  let is_open = open.contains(&id);
  let icon = if is_open { "▲" } else { "▼" };

  iced::widget::button(
    row![
      text(format!("? {label}"))
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_ACCENT),
      iced::widget::Space::new().width(4),
      text(icon).size(9).color(TEXT_ACCENT),
    ]
    .align_y(iced::Alignment::Center),
  )
  .on_press(Message::ToggleHelp(id))
  .style(|_, _| iced::widget::button::Style {
    background: None,
    border: Border::default(),
    ..Default::default()
  })
  .into()
}

/// Bloc d'aide inline (affiché quand le collapse est ouvert).
pub fn help_block(content: &str) -> Element<'static, Message> {
  let bar = container(iced::widget::Space::new().height(Length::Fill))
    .width(2)
    .style(|_| container::Style {
      background: Some(Background::Color(ACCENT_PRIMARY)),
      ..Default::default()
    });

  container(row![
    bar,
    iced::widget::Space::new().width(10),
    text(content.to_string())
      .size(12)
      .color(TEXT_SECONDARY)
      .width(Length::Fill),
  ])
  .padding([10, 12])
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_CARD)),
    border: Border {
      radius: 6.0.into(),
      ..Default::default()
    },
    ..Default::default()
  })
  .into()
}

/// Bloc info (fond accent, toujours visible — niveau 1).
pub fn info_block(content: &str) -> Element<'static, Message> {
  let bar = container(iced::widget::Space::new().height(Length::Fill))
    .width(2)
    .style(|_| container::Style {
      background: Some(Background::Color(ACCENT_PRIMARY)),
      ..Default::default()
    });

  container(row![
    bar,
    iced::widget::Space::new().width(10),
    text(content.to_string()).size(12).color(TEXT_SECONDARY),
  ])
  .padding([10, 12])
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(crate::ui::theme::ACCENT_SUBTLE)),
    border: Border {
      radius: 6.0.into(),
      ..Default::default()
    },
    ..Default::default()
  })
  .into()
}

/// Séparateur de section avec label.
pub fn section_label(label: &str) -> Element<'static, Message> {
  container(
    row![
      text(label.to_string())
        .size(11)
        .font(FONT_MEDIUM)
        .color(TEXT_SECONDARY),
      iced::widget::Space::new().width(8),
      container(iced::widget::Space::new().height(1))
        .width(Length::Fill)
        .style(|_| container::Style {
          background: Some(Background::Color(BORDER_SUBTLE)),
          ..Default::default()
        }),
    ]
    .align_y(iced::Alignment::Center),
  )
  .padding([8, 0])
  .width(Length::Fill)
  .into()
}

/// Affiche conditionnellement un bloc d'aide selon l'état du collapse.
pub fn maybe_help<'a>(
  id: HelpId,
  label: &'a str,
  content: &'static str,
  open: &HashSet<HelpId>,
) -> Element<'a, Message> {
  let is_open = open.contains(&id);

  if is_open {
    column![help_toggle(id, label, open), help_block(content),]
      .spacing(6)
      .into()
  } else {
    help_toggle(id, label, open)
  }
}
