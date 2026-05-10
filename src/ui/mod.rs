pub mod deploy_flow;
pub mod detail_panel;
pub mod gpg_setup;
pub mod help;
pub mod key_gen_flow;
pub mod key_list;
pub mod service_form;
pub mod service_list;
pub mod sidebar;
pub mod stepper;
pub mod theme;
pub mod widgets;

use iced::widget::{center, column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::{GpgSetupState, ReadyState};
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

pub fn view_gpg_setup(state: &GpgSetupState) -> Element<'_, Message> {
  gpg_setup::view(state)
}

pub fn view_ready(state: &ReadyState) -> Element<'_, Message> {
  use crate::app::message::NavItem;

  // Colonne 2 : liste
  let list_content: Element<Message> = match state.active_nav {
    NavItem::Services => service_list::view_service_list(
      &state.config.services,
      &state.local_keys,
      state.selected_service,
    ),
    NavItem::SshKeys => key_list::view_key_list(
      &state.local_keys,
      &state.config.services,
      &state.key_protection,
      state.selected_key,
    ),
    NavItem::Settings => container(text("Paramètres — à venir").size(13).color(TEXT_SECONDARY))
      .width(Length::Fill)
      .height(Length::Fill)
      .padding(24)
      .into(),
  };

  // Colonne 3 : deploy > keygen > formulaire > détail
  let detail: Element<Message> = if let Some(ref d) = state.deploy {
    deploy_flow::view(d)
  } else if let Some(ref kg) = state.key_gen {
    key_gen_flow::view(kg)
  } else if let Some(ref form) = state.form {
    service_form::view(form)
  } else if let Some(sid) = state.selected_service {
    detail_panel::view_service_detail(sid, state)
  } else if let Some(kid) = state.selected_key {
    detail_panel::view_key_detail(kid, state)
  } else {
    detail_panel::view_empty_panel()
  };

  // Séparateur vertical simulé
  let vsep = container(iced::widget::Space::new().width(1).height(Length::Fill)).style(|_| {
    container::Style {
      background: Some(Background::Color(crate::ui::theme::BORDER_DEFAULT)),
      ..Default::default()
    }
  });

  row![
    sidebar::view_sidebar(state.active_nav),
    container(list_content)
      .width(Length::FillPortion(2))
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(Background::Color(BACKGROUND_BASE)),
        ..Default::default()
      }),
    vsep,
    container(detail)
      .width(Length::FillPortion(3))
      .height(Length::Fill)
      .style(|_| container::Style {
        background: Some(Background::Color(BACKGROUND_BASE)),
        ..Default::default()
      }),
  ]
  .into()
}
