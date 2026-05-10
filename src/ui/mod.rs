pub mod key_list;
pub mod service_list;
pub mod sidebar;
pub mod theme;
pub mod widgets;

use iced::widget::{center, column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::ReadyState;
use crate::error::AppError;
use crate::ui::theme::{
  BACKGROUND_BASE, DANGER_RED, DANGER_SUBTLE, FONT_MEDIUM, FONT_SEMIBOLD, TEXT_PRIMARY,
  TEXT_SECONDARY,
};

pub fn view_loading() -> Element<'static, Message> {
  center(
    text("Chargement…")
      .font(FONT_SEMIBOLD)
      .size(16)
      .color(TEXT_SECONDARY),
  )
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_BASE)),
    ..Default::default()
  })
  .into()
}

pub fn view_error(error: &AppError) -> Element<'static, Message> {
  let msg = error.to_string();

  let bar = container(iced::widget::Space::new().height(Length::Fill))
    .width(3)
    .style(|_| container::Style {
      background: Some(Background::Color(DANGER_RED)),
      border: Border {
        radius: 2.0.into(),
        ..Default::default()
      },
      ..Default::default()
    });

  let content = column![
    row![
      text("✕").size(14).color(DANGER_RED).font(FONT_MEDIUM),
      iced::widget::Space::new().width(8),
      text("Erreur de configuration")
        .size(13)
        .font(FONT_SEMIBOLD)
        .color(TEXT_PRIMARY),
    ]
    .align_y(iced::Alignment::Center),
    iced::widget::Space::new().height(8),
    text(msg).size(12).color(TEXT_SECONDARY),
  ];

  let block = container(row![bar, iced::widget::Space::new().width(12), content])
    .padding([12, 16])
    .style(|_| container::Style {
      background: Some(Background::Color(DANGER_SUBTLE)),
      border: Border {
        radius: 6.0.into(),
        ..Default::default()
      },
      ..Default::default()
    });

  center(block)
    .style(|_| container::Style {
      background: Some(Background::Color(BACKGROUND_BASE)),
      ..Default::default()
    })
    .into()
}

pub fn view_ready(state: &ReadyState) -> Element<'_, Message> {
  use crate::app::message::NavItem;

  let content: Element<Message> = match state.active_nav {
    NavItem::Services => service_list::view_service_list(&state.config.services, &state.local_keys),
    NavItem::SshKeys => key_list::view_key_list(&state.local_keys, &state.config.services),
    NavItem::Settings => container(text("Paramètres — à venir").size(13).color(TEXT_SECONDARY))
      .width(Length::Fill)
      .height(Length::Fill)
      .padding(24)
      .into(),
  };

  row![
    sidebar::view_sidebar(state.active_nav),
    container(content)
      .width(Length::Fill)
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(Background::Color(BACKGROUND_BASE)),
        ..Default::default()
      }),
  ]
  .into()
}
