/// Wizard de génération de clef SSH.
use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Element, Length};
use uuid::Uuid;

use crate::app::message::Message;
use crate::app::{KeyGenState, KeyGenStep};
use crate::config::model::KeyType;
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_BASE, BACKGROUND_ELEVATED, BORDER_DEFAULT, DANGER_RED,
  DANGER_SUBTLE, FONT_MEDIUM, FONT_MONO, FONT_SEMIBOLD, SUCCESS_GREEN, SUCCESS_SUBTLE,
  TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY, WARNING_AMBER, WARNING_SUBTLE,
};

pub fn view(state: &KeyGenState) -> Element<'_, Message> {
  match &state.step {
    KeyGenStep::ChooseType => view_choose_type(state.service_id),
    KeyGenStep::EnterPassphrase => view_enter_passphrase(state),
    KeyGenStep::Generating => view_generating(),
    KeyGenStep::Success { fingerprint } => view_success(fingerprint),
  }
}

pub fn view_choose_type(_service_id: Uuid) -> Element<'static, Message> {
  let ed25519_card = iced::widget::button(
    container(
      column![
        row![
          text("ed25519")
            .font(FONT_SEMIBOLD)
            .size(14)
            .color(TEXT_PRIMARY),
          iced::widget::Space::new().width(8),
          container(
            text("RECOMMANDÉ")
              .size(10)
              .font(FONT_MEDIUM)
              .color(SUCCESS_GREEN),
          )
          .padding([2, 6])
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(SUCCESS_SUBTLE)),
            border: Border {
              radius: 4.0.into(),
              ..Default::default()
            },
            ..Default::default()
          }),
        ]
        .align_y(Alignment::Center),
        iced::widget::Space::new().height(6),
        text("La clef est stockée dans un fichier sur cet ordinateur.")
          .size(12)
          .color(TEXT_SECONDARY),
        text("Protégez-la avec une phrase secrète.")
          .size(12)
          .color(TEXT_SECONDARY),
      ]
      .spacing(0),
    )
    .padding([16, 16])
    .width(Length::Fill),
  )
  .on_press(Message::KeyTypeSelected(KeyType::Ed25519))
  .style(|_, status| iced::widget::button::Style {
    background: Some(Background::Color(
      if matches!(status, iced::widget::button::Status::Hovered) {
        ACCENT_SUBTLE
      } else {
        BACKGROUND_ELEVATED
      },
    )),
    border: Border {
      radius: 8.0.into(),
      color: BORDER_DEFAULT,
      width: 1.0,
    },
    ..Default::default()
  })
  .width(Length::Fill);

  let sk_card = iced::widget::button(
    container(
      column![
        row![
          text("sk-ed25519")
            .font(FONT_SEMIBOLD)
            .size(14)
            .color(TEXT_PRIMARY),
          iced::widget::Space::new().width(8),
          container(
            text("YUBIKEY")
              .size(10)
              .font(FONT_MEDIUM)
              .color(WARNING_AMBER),
          )
          .padding([2, 6])
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(WARNING_SUBTLE)),
            border: Border {
              radius: 4.0.into(),
              ..Default::default()
            },
            ..Default::default()
          }),
        ]
        .align_y(Alignment::Center),
        iced::widget::Space::new().height(6),
        text("La clef privée ne quitte jamais votre clef physique (YubiKey, Nitrokey…).")
          .size(12)
          .color(TEXT_SECONDARY),
        text("Assurez-vous que votre clef physique est branchée.")
          .size(11)
          .color(TEXT_DISABLED),
      ]
      .spacing(0),
    )
    .padding([16, 16])
    .width(Length::Fill),
  )
  .on_press(Message::KeyTypeSelected(KeyType::SkEd25519))
  .style(|_, status| iced::widget::button::Style {
    background: Some(Background::Color(
      if matches!(status, iced::widget::button::Status::Hovered) {
        ACCENT_SUBTLE
      } else {
        BACKGROUND_ELEVATED
      },
    )),
    border: Border {
      radius: 8.0.into(),
      color: BORDER_DEFAULT,
      width: 1.0,
    },
    ..Default::default()
  })
  .width(Length::Fill);

  container(
    column![
      container(column![
        text("Générer une nouvelle clef SSH")
          .font(FONT_SEMIBOLD)
          .size(16)
          .color(TEXT_PRIMARY),
        iced::widget::Space::new().height(4),
        text("Quel type de clef souhaitez-vous créer ?")
          .size(13)
          .color(TEXT_SECONDARY),
      ],)
      .padding(iced::Padding {
        top: 24.0,
        right: 24.0,
        bottom: 16.0,
        left: 24.0
      }),
      container(ed25519_card).padding([0, 24]),
      iced::widget::Space::new().height(8),
      container(sk_card).padding([0, 24]),
      iced::widget::Space::new().height(16),
      container(
        iced::widget::button(text("Annuler").size(13).color(TEXT_SECONDARY))
          .on_press(Message::CloseKeyGen)
          .style(|_, _| iced::widget::button::Style {
            background: None,
            ..Default::default()
          }),
      )
      .padding([0, 24]),
      iced::widget::Space::new().height(24),
    ]
    .width(Length::Fill),
  )
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(BACKGROUND_BASE)),
    ..Default::default()
  })
  .into()
}

fn view_enter_passphrase(state: &KeyGenState) -> Element<'_, Message> {
  let key_label = match state.key_type {
    Some(KeyType::SkEd25519) => "sk-ed25519 (YubiKey)",
    _ => "ed25519",
  };
  let passphrase_ok = state.passphrase.len() >= 12;

  let error_block: Element<Message> = if let Some(ref err) = state.error {
    container(
      row![
        text("✕").size(12).color(DANGER_RED),
        iced::widget::Space::new().width(8),
        text(err.clone()).size(12).color(TEXT_PRIMARY),
      ]
      .align_y(Alignment::Center),
    )
    .padding([10, 24])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(DANGER_SUBTLE)),
      ..Default::default()
    })
    .into()
  } else {
    iced::widget::Space::new().height(0).into()
  };

  scrollable(
    column![
      container(column![
        text("Phrase secrète")
          .font(FONT_SEMIBOLD)
          .size(16)
          .color(TEXT_PRIMARY),
        iced::widget::Space::new().height(4),
        text(format!("Type : {key_label}"))
          .size(12)
          .color(TEXT_SECONDARY),
      ],)
      .padding(iced::Padding {
        top: 24.0,
        right: 24.0,
        bottom: 16.0,
        left: 24.0
      }),
      iced::widget::rule::horizontal(1).style(|_: &iced::Theme| iced::widget::rule::Style {
        color: BORDER_DEFAULT,
        radius: 0.0.into(),
        fill_mode: iced::widget::rule::FillMode::Full,
        snap: true,
      }),
      container(
        column![
          text("Phrase secrète *")
            .size(12)
            .font(FONT_MEDIUM)
            .color(TEXT_PRIMARY),
          iced::widget::Space::new().height(4),
          text_input("12 caractères minimum", &state.passphrase)
            .on_input(Message::PassphraseChanged)
            .secure(true)
            .padding(10)
            .size(13)
            .style(|_theme: &iced::Theme, status| {
              let focused = matches!(status, text_input::Status::Focused { .. });
              text_input::Style {
                background: Background::Color(BACKGROUND_ELEVATED),
                border: Border {
                  radius: 6.0.into(),
                  color: if focused {
                    ACCENT_PRIMARY
                  } else {
                    BORDER_DEFAULT
                  },
                  width: if focused { 2.0 } else { 1.0 },
                },
                icon: TEXT_DISABLED,
                placeholder: TEXT_DISABLED,
                value: TEXT_PRIMARY,
                selection: ACCENT_SUBTLE,
              }
            }),
          iced::widget::Space::new().height(4),
          text("Obligatoire — protège la clef si le disque ou le device est compromis.")
            .size(11)
            .color(TEXT_SECONDARY),
        ]
        .spacing(2),
      )
      .padding([16, 24]),
      error_block,
      container(
        row![
          iced::widget::button(text("← Retour").size(13).color(TEXT_SECONDARY))
            .on_press(Message::CloseKeyGen)
            .style(|_, _| iced::widget::button::Style {
              background: None,
              ..Default::default()
            }),
          iced::widget::Space::new().width(Length::Fill),
          {
            let fg = if passphrase_ok {
              TEXT_PRIMARY
            } else {
              TEXT_DISABLED
            };
            let bg = if passphrase_ok {
              ACCENT_PRIMARY
            } else {
              BACKGROUND_ELEVATED
            };
            let mut btn = iced::widget::button(text("Générer").size(13).color(fg))
              .style(move |_, _| iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                  radius: 6.0.into(),
                  ..Default::default()
                },
                ..Default::default()
              })
              .padding([8, 16]);
            if passphrase_ok {
              btn = btn.on_press(Message::GenerateKeyConfirmed);
            }
            btn
          },
        ]
        .align_y(Alignment::Center),
      )
      .padding(iced::Padding {
        top: 16.0,
        right: 24.0,
        bottom: 24.0,
        left: 24.0
      })
      .width(Length::Fill),
    ]
    .width(Length::Fill),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .into()
}

pub fn view_generating() -> Element<'static, Message> {
  container(
    column![
      text("Génération en cours…")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(16),
      text("Création de la paire de clefs ed25519…")
        .size(12)
        .color(TEXT_SECONDARY),
      iced::widget::Space::new().height(8),
      text("Cette opération prend généralement moins de 2 secondes.")
        .size(12)
        .color(TEXT_SECONDARY),
    ]
    .align_x(Alignment::Center),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(iced::alignment::Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .padding(24)
  .into()
}

pub fn view_success(fingerprint: &str) -> Element<'_, Message> {
  container(
    column![
      text("Clef générée avec succès")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(SUCCESS_GREEN),
      iced::widget::Space::new().height(16),
      container(
        text(fingerprint.to_string())
          .size(11)
          .font(FONT_MONO)
          .color(TEXT_PRIMARY),
      )
      .padding([10, 12])
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(SUCCESS_SUBTLE)),
        border: Border {
          radius: 6.0.into(),
          color: SUCCESS_GREEN,
          width: 1.0,
        },
        ..Default::default()
      }),
      iced::widget::Space::new().height(24),
      iced::widget::button(text("Fermer").size(13).color(TEXT_PRIMARY))
        .on_press(Message::CloseKeyGen)
        .style(|_, _| iced::widget::button::Style {
          background: Some(Background::Color(ACCENT_PRIMARY)),
          border: Border {
            radius: 6.0.into(),
            ..Default::default()
          },
          ..Default::default()
        })
        .padding([8, 16]),
    ]
    .align_x(Alignment::Center),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(iced::alignment::Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .padding(24)
  .into()
}
