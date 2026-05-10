#![allow(dead_code)]
#![allow(unused_imports)]
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Element, Length, Padding};

use crate::app::message::Message;
use crate::app::GpgSetupState;
use crate::subprocess::gpg::GpgKeyInfo;
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_BASE, BACKGROUND_CARD, BACKGROUND_ELEVATED,
  BACKGROUND_SUBTLE, BORDER_DEFAULT, FONT_MEDIUM, FONT_MONO, FONT_SEMIBOLD, TEXT_DISABLED,
  TEXT_PRIMARY, TEXT_SECONDARY,
};

pub fn view(state: &GpgSetupState) -> Element<'_, Message> {
  let body: Element<Message> = if state.loading {
    view_loading()
  } else if state.available_keys.is_empty() {
    view_no_keys()
  } else {
    view_select_existing(&state.available_keys, state.selected_fingerprint.as_deref())
  };

  container(
    column![
      text("SSHive")
        .font(FONT_SEMIBOLD)
        .size(24)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Configuration GPG requise")
        .font(FONT_SEMIBOLD)
        .size(18)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text(
        "SSHive chiffre vos données sensibles avec une clef GPG. \
         Sélectionnez une clef existante pour continuer."
      )
      .size(13)
      .color(TEXT_SECONDARY),
      iced::widget::Space::new().height(32),
      body,
    ]
    .align_x(Alignment::Center)
    .max_width(600),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(iced::alignment::Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_BASE)),
    ..Default::default()
  })
  .into()
}

fn view_loading() -> Element<'static, Message> {
  container(
    text("Recherche des clefs GPG sur cet ordinateur…")
      .size(13)
      .color(TEXT_SECONDARY),
  )
  .padding(24)
  .into()
}

fn view_no_keys() -> Element<'static, Message> {
  container(
    column![
      text("Aucune clef GPG trouvée")
        .font(FONT_SEMIBOLD)
        .size(15)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Créez une clef GPG dans un terminal, puis relancez SSHive :")
        .size(12)
        .color(TEXT_SECONDARY),
      iced::widget::Space::new().height(8),
      container(
        text("gpg --full-generate-key")
          .size(12)
          .font(FONT_MONO)
          .color(TEXT_PRIMARY),
      )
      .padding([8, 12])
      .style(|_| container::Style {
        background: Some(Background::Color(BACKGROUND_CARD)),
        border: Border {
          radius: 4.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
    ]
    .spacing(0),
  )
  .padding(24)
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_ELEVATED)),
    border: Border {
      radius: 8.0.into(),
      color: BORDER_DEFAULT,
      width: 1.0,
    },
    ..Default::default()
  })
  .into()
}

fn view_select_existing<'a>(
  keys: &'a [GpgKeyInfo],
  selected: Option<&'a str>,
) -> Element<'a, Message> {
  let key_cards = scrollable(
    column(
      keys
        .iter()
        .map(|k| key_card(k, selected))
        .collect::<Vec<_>>(),
    )
    .spacing(8),
  );

  let confirm_enabled = selected.is_some();
  let confirm_bg = if confirm_enabled {
    ACCENT_PRIMARY
  } else {
    BACKGROUND_SUBTLE
  };
  let confirm_fg = if confirm_enabled {
    TEXT_PRIMARY
  } else {
    TEXT_DISABLED
  };

  let mut confirm_btn =
    iced::widget::button(text("Utiliser cette clef").size(13).color(confirm_fg))
      .style(move |_, _| iced::widget::button::Style {
        background: Some(Background::Color(confirm_bg)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      })
      .padding([10, 24]);

  if selected.is_some() {
    confirm_btn = confirm_btn.on_press(Message::GpgSetupConfirmed);
  }

  column![
    text(format!(
      "{} clef{} GPG disponible{} :",
      keys.len(),
      if keys.len() != 1 { "s" } else { "" },
      if keys.len() != 1 { "s" } else { "" }
    ))
    .size(13)
    .color(TEXT_SECONDARY),
    iced::widget::Space::new().height(12),
    key_cards,
    iced::widget::Space::new().height(24),
    container(confirm_btn)
      .width(Length::Fill)
      .align_x(iced::alignment::Horizontal::Center),
  ]
  .spacing(0)
  .into()
}

fn key_card<'a>(key: &'a GpgKeyInfo, selected: Option<&'a str>) -> Element<'a, Message> {
  let is_selected = selected == Some(key.fingerprint.as_str());

  let fp_short = if key.fingerprint.len() > 16 {
    format!(
      "{}…{}",
      &key.fingerprint[..8],
      &key.fingerprint[key.fingerprint.len() - 8..]
    )
  } else {
    key.fingerprint.clone()
  };

  let content = column![
    text(key.uid.clone())
      .size(13)
      .font(FONT_SEMIBOLD)
      .color(TEXT_PRIMARY),
    iced::widget::Space::new().height(4),
    text(format!("Clef : {fp_short}"))
      .size(11)
      .font(FONT_MONO)
      .color(TEXT_SECONDARY),
  ];

  let bg = if is_selected {
    ACCENT_SUBTLE
  } else {
    BACKGROUND_ELEVATED
  };
  let border_color = if is_selected {
    ACCENT_PRIMARY
  } else {
    BORDER_DEFAULT
  };
  let border_width = if is_selected { 2.0 } else { 1.0 };

  iced::widget::button(
    container(content)
      .padding(Padding {
        top: 12.0,
        right: 16.0,
        bottom: 12.0,
        left: 16.0,
      })
      .width(Length::Fill),
  )
  .on_press(Message::GpgKeySelected(key.fingerprint.clone()))
  .style(move |_, _| iced::widget::button::Style {
    background: Some(Background::Color(bg)),
    border: Border {
      radius: 8.0.into(),
      color: border_color,
      width: border_width,
    },
    ..Default::default()
  })
  .width(Length::Fill)
  .into()
}
