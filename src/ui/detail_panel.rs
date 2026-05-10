#![allow(dead_code)]
#![allow(unused_imports)]
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};
use uuid::Uuid;

use crate::app::message::{HelpId, Message, Message::StartRevoke};
use crate::app::{AddPassphraseStatus, ReadyState, RevokeStatus};
use crate::subprocess::ssh_keygen::ProtectionStatus;
use crate::ui::help::{maybe_help, section_label};
use crate::ui::theme::{
  ACCENT_HOVER, ACCENT_PRIMARY, BACKGROUND_BASE, BACKGROUND_CARD, BACKGROUND_ELEVATED,
  BORDER_DEFAULT, BORDER_SUBTLE, DANGER_RED, DANGER_SUBTLE, FONT_MEDIUM, FONT_SEMIBOLD,
  SUCCESS_GREEN, SUCCESS_SUBTLE, TEXT_ACCENT, TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY,
  WARNING_AMBER,
};
use crate::ui::widgets::{
  age_indicator, count_services_using_key, fingerprint_text, service_type_badge, yubikey_badge,
};

/// Panneau de détail d'un service sélectionné.
pub fn view_service_detail<'a>(service_id: Uuid, state: &'a ReadyState) -> Element<'a, Message> {
  let Some(service) = state.config.services.iter().find(|s| s.id == service_id) else {
    return empty_panel();
  };

  let active_key = service.active_key.and_then(|id| {
    state
      .config
      .keys
      .iter()
      .find(|k| k.id == id)
      .or_else(|| state.local_keys.iter().find(|k| k.id == id))
  });

  let age = age_indicator(service.last_rotation);

  column![
    // En-tête service
    container(
      row![
        service_type_badge(&service.service_type),
        iced::widget::Space::new().width(10),
        text(service.name.clone())
          .font(FONT_SEMIBOLD)
          .size(18)
          .color(TEXT_PRIMARY),
      ]
      .align_y(Alignment::Center),
    )
    .padding(iced::Padding {
      top: 24.0,
      right: 24.0,
      bottom: 8.0,
      left: 24.0
    }),
    container(
      text(format!(
        "Créé le {}",
        service.created_at.format("%d %b. %Y")
      ))
      .size(12)
      .color(TEXT_SECONDARY),
    )
    .padding(iced::Padding {
      top: 0.0,
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
    // Section connexion
    if service.params.url.is_some() || service.params.user.is_some() {
      view_connection_section(service)
    } else {
      iced::widget::Space::new().height(0).into()
    },
    // Section clef active
    view_key_section(service_id, active_key, &age, service.last_rotation, state),
    // Section révocation clefs anciennes
    view_revoke_section(service_id, service.active_key, state),
    // Actions
    view_actions(service_id),
  ]
  .width(Length::Fill)
  .height(Length::Fill)
  .into()
}

fn view_connection_section(service: &crate::config::model::Service) -> Element<'static, Message> {
  let mut rows: Vec<Element<Message>> = vec![section_label("CONNEXION")];

  if let Some(url) = &service.params.url {
    rows.push(field_row("Hôte", url));
  }
  if let Some(user) = &service.params.user {
    rows.push(field_row("Utilisateur", user));
  }
  if let Some(port) = service.params.port {
    rows.push(field_row(
      "Port",
      &format!("{port}{}", if port == 22 { " (par défaut)" } else { "" }),
    ));
  }

  container(column(rows).spacing(6))
    .padding([12, 24])
    .width(Length::Fill)
    .into()
}

fn view_key_section<'a>(
  service_id: Uuid,
  active_key: Option<&'a crate::config::model::SshKey>,
  age: &crate::ui::widgets::AgeIndicator,
  _last_rotation: Option<chrono::NaiveDate>,
  state: &'a ReadyState,
) -> Element<'a, Message> {
  let key_info: Element<Message> = match active_key {
    Some(key) => column![
      row![
        fingerprint_text(&key.fingerprint),
        iced::widget::Space::new().width(8),
        if key.yubikey {
          yubikey_badge()
        } else {
          iced::widget::Space::new().width(0).into()
        },
      ]
      .align_y(Alignment::Center),
      iced::widget::Space::new().height(4),
      row![
        text(format!("Générée le {}", key.created_at.format("%d %b. %Y")))
          .size(11)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().width(12),
        text(age.label.clone()).size(11).color(if age.is_warning {
          WARNING_AMBER
        } else {
          TEXT_SECONDARY
        }),
      ]
      .align_y(Alignment::Center),
    ]
    .into(),
    None => text("Aucune clef associée")
      .size(12)
      .color(TEXT_DISABLED)
      .into(),
  };

  let primary_action: Element<Message> = if let Some(key) = active_key {
    iced::widget::button(text("Faire pivoter la clef").size(13).color(TEXT_PRIMARY))
      .on_press(Message::OpenDeployFlow {
        service_id,
        key_id: key.id,
      })
      .style(|_, status| iced::widget::button::Style {
        background: Some(Background::Color(
          if matches!(status, iced::widget::button::Status::Hovered) {
            ACCENT_HOVER
          } else {
            ACCENT_PRIMARY
          },
        )),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      })
      .padding([8, 16])
      .into()
  } else {
    iced::widget::button(
      text("Générer une nouvelle clef")
        .size(13)
        .color(TEXT_PRIMARY),
    )
    .on_press(Message::OpenGenerateKey { service_id })
    .style(|_, status| iced::widget::button::Style {
      background: Some(Background::Color(
        if matches!(status, iced::widget::button::Status::Hovered) {
          ACCENT_HOVER
        } else {
          ACCENT_PRIMARY
        },
      )),
      border: Border {
        radius: 6.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .padding([8, 16])
    .into()
  };

  // Le picker n'est affiché que quand aucune clef n'est encore assignée (B2/B3)
  let assign_picker: Element<Message> = if active_key.is_none() {
    view_assign_picker(service_id, &state.local_keys)
  } else {
    iced::widget::Space::new().height(0).into()
  };

  column![
    section_label("CLEF SSH ACTIVE"),
    key_info,
    iced::widget::Space::new().height(12),
    primary_action,
    iced::widget::Space::new().height(8),
    assign_picker,
    maybe_help(
      HelpId::Rotation,
      "Qu'est-ce que la rotation ?",
      "Remplacer périodiquement votre clef SSH réduit le risque si elle est compromise. \
       SSHive génère une nouvelle clef, la déploie, vérifie qu'elle fonctionne, \
       puis révoque l'ancienne.",
      &state.open_helps,
    ),
  ]
  .padding([12, 24])
  .spacing(4)
  .width(Length::Fill)
  .into()
}

fn view_assign_picker<'a>(
  service_id: Uuid,
  local_keys: &'a [crate::config::model::SshKey],
) -> Element<'a, Message> {
  use crate::config::model::KeyType;
  use crate::ui::widgets::service_type_badge;

  if local_keys.is_empty() {
    return iced::widget::Space::new().height(0).into();
  }

  let cards: Vec<Element<Message>> = local_keys
    .iter()
    .map(|k| {
      let key_id = k.id;

      // Badge de type de clef
      let type_label = match k.key_type {
        KeyType::Ed25519 => "ed25519",
        KeyType::SkEd25519 => "sk-ed25519",
      };
      let type_badge = container(
        text(type_label)
          .size(10)
          .font(FONT_MEDIUM)
          .color(TEXT_SECONDARY),
      )
      .padding([2, 6])
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(BACKGROUND_BASE)),
        border: Border {
          radius: 3.0.into(),
          color: BORDER_SUBTLE,
          width: 1.0,
        },
        ..Default::default()
      });

      let yubikey_indicator: Element<Message> = if k.yubikey {
        row![iced::widget::Space::new().width(6), yubikey_badge(),].into()
      } else {
        iced::widget::Space::new().width(0).into()
      };

      // Commentaire (prioritaire) ou fingerprint tronqué
      let display_name = if k.comment.is_empty() {
        let fp = &k.fingerprint;
        // Afficher la partie après "SHA256:" tronquée
        fp.trim_start_matches("SHA256:")
          .chars()
          .take(24)
          .collect::<String>()
          + "…"
      } else {
        k.comment.clone()
      };

      // Fingerprint court (12 derniers chars)
      let fp_short = k
        .fingerprint
        .trim_start_matches("SHA256:")
        .chars()
        .rev()
        .take(12)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

      iced::widget::button(
        container(
          column![
            row![
              type_badge,
              yubikey_indicator,
              iced::widget::Space::new().width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            iced::widget::Space::new().height(4),
            text(display_name).size(12).color(TEXT_PRIMARY),
            text(format!("…{fp_short}"))
              .size(10)
              .font(FONT_MEDIUM)
              .color(TEXT_DISABLED),
          ]
          .spacing(0),
        )
        .padding([10, 12])
        .width(Length::Fill),
      )
      .on_press(Message::AssignKeyToService { service_id, key_id })
      .style(|_, status| iced::widget::button::Style {
        background: Some(Background::Color(
          if matches!(status, iced::widget::button::Status::Hovered) {
            BACKGROUND_CARD
          } else {
            BACKGROUND_ELEVATED
          },
        )),
        border: Border {
          radius: 6.0.into(),
          color: BORDER_SUBTLE,
          width: 1.0,
        },
        ..Default::default()
      })
      .width(Length::Fill)
      .into()
    })
    .collect();

  column![
    section_label("ATTACHER UNE CLEF EXISTANTE"),
    column(cards).spacing(6),
  ]
  .spacing(6)
  .width(Length::Fill)
  .into()
}

fn view_revoke_section<'a>(
  service_id: Uuid,
  active_key_id: Option<Uuid>,
  state: &'a ReadyState,
) -> Element<'a, Message> {
  // Anciennes clefs : dans config.keys, liées à ce service, mais pas la clef active
  let old_keys: Vec<&crate::config::model::SshKey> = state
    .config
    .keys
    .iter()
    .filter(|k| k.service_id == Some(service_id) && Some(k.id) != active_key_id)
    .collect();

  if old_keys.is_empty() {
    return iced::widget::Space::new().height(0).into();
  }

  let rows: Vec<Element<Message>> = old_keys
    .iter()
    .map(|key| {
      let key_id = key.id;

      // Vérifier si une révocation est en cours pour cette clef
      let revoke_status = state.revoke.as_ref().filter(|r| r.key_id == key_id);

      let action: Element<Message> = match revoke_status {
        Some(r) => match &r.status {
          RevokeStatus::Revoking => text("Révocation en cours…")
            .size(11)
            .color(TEXT_SECONDARY)
            .into(),
          RevokeStatus::Success => text("✓ Révoquée").size(11).color(SUCCESS_GREEN).into(),
          RevokeStatus::Error(e) => column![
            text(format!("✕ {e}")).size(11).color(DANGER_RED),
            iced::widget::button(text("Réessayer").size(11).color(TEXT_SECONDARY))
              .on_press(Message::StartRevoke { service_id, key_id })
              .style(|_, _| iced::widget::button::Style {
                background: None,
                ..Default::default()
              }),
          ]
          .spacing(2)
          .into(),
        },
        None => iced::widget::button(text("Révoquer").size(11).color(DANGER_RED))
          .on_press(Message::StartRevoke { service_id, key_id })
          .style(|_, status| iced::widget::button::Style {
            background: Some(iced::Background::Color(
              if matches!(status, iced::widget::button::Status::Hovered) {
                DANGER_SUBTLE
              } else {
                iced::Color::TRANSPARENT
              },
            )),
            border: iced::Border {
              radius: 4.0.into(),
              ..Default::default()
            },
            ..Default::default()
          })
          .padding([2, 8])
          .into(),
      };

      container(
        row![
          column![
            fingerprint_text(&key.fingerprint),
            text(format!("Générée le {}", key.created_at.format("%d %b. %Y")))
              .size(10)
              .color(TEXT_DISABLED),
          ]
          .spacing(2)
          .width(Length::Fill),
          action,
        ]
        .align_y(Alignment::Center),
      )
      .padding([8, 0])
      .width(Length::Fill)
      .into()
    })
    .collect();

  container(column![section_label("ANCIENNES CLEFS"), column(rows).spacing(4),].spacing(6))
    .padding([12, 24])
    .width(Length::Fill)
    .into()
}

fn view_key_security_section<'a>(key_id: Uuid, state: &'a ReadyState) -> Element<'a, Message> {
  use crate::app::AddPassphraseStatus;

  let protection = state.key_protection.get(&key_id);

  // Clef protégée ou hardware → pas de section
  match protection {
    Some(ProtectionStatus::Protected) | Some(ProtectionStatus::HardwareKey) | None => {
      return iced::widget::Space::new().height(0).into();
    }
    Some(ProtectionStatus::Unprotected) => {}
  }

  // Regarder si un wizard est en cours pour cette clef
  let wizard = state.add_passphrase.as_ref().filter(|a| a.key_id == key_id);

  let body: Element<Message> = match wizard {
    Some(aps) => match &aps.status {
      AddPassphraseStatus::CollectingPassphrase => container(
        text("Saisie de la phrase secrète en cours…")
          .size(12)
          .color(TEXT_SECONDARY),
      )
      .into(),

      AddPassphraseStatus::ApplyingPassphrase => container(
        text("Chiffrement de la clef en cours…")
          .size(12)
          .color(TEXT_SECONDARY),
      )
      .into(),

      AddPassphraseStatus::Error(e) => column![
        container(
          row![
            text("✕").size(12).color(DANGER_RED),
            iced::widget::Space::new().width(8),
            text(e.clone()).size(12).color(TEXT_PRIMARY),
          ]
          .align_y(Alignment::Center),
        )
        .padding([8, 10])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(DANGER_SUBTLE)),
          border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
          },
          ..Default::default()
        }),
        iced::widget::Space::new().height(8),
        iced::widget::button(text("Réessayer").size(12).color(TEXT_PRIMARY),)
          .on_press(Message::OpenAddPassphrase { key_id })
          .style(|_, _| iced::widget::button::Style {
            background: Some(Background::Color(ACCENT_PRIMARY)),
            border: iced::Border {
              radius: 6.0.into(),
              ..Default::default()
            },
            ..Default::default()
          })
          .padding([6, 12]),
      ]
      .spacing(0)
      .into(),
    },
    None => column![
      container(
        row![
          text("⚠").size(13).color(DANGER_RED),
          iced::widget::Space::new().width(8),
          text("Cette clef n'est pas protégée par une phrase secrète.")
            .size(12)
            .color(TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center),
      )
      .padding([8, 10])
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(DANGER_SUBTLE)),
        border: iced::Border {
          radius: 4.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
      iced::widget::Space::new().height(8),
      iced::widget::button(
        text("Ajouter une phrase secrète")
          .size(12)
          .color(TEXT_PRIMARY),
      )
      .on_press(Message::OpenAddPassphrase { key_id })
      .style(|_, status| iced::widget::button::Style {
        background: Some(Background::Color(
          if matches!(status, iced::widget::button::Status::Hovered) {
            ACCENT_HOVER
          } else {
            ACCENT_PRIMARY
          },
        )),
        border: iced::Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      })
      .padding([6, 12]),
    ]
    .spacing(0)
    .into(),
  };

  container(column![section_label("SÉCURITÉ"), body].spacing(6))
    .padding([12, 24])
    .width(Length::Fill)
    .into()
}

fn view_actions(service_id: Uuid) -> Element<'static, Message> {
  column![
    iced::widget::rule::horizontal(1).style(|_: &iced::Theme| iced::widget::rule::Style {
      color: BORDER_DEFAULT,
      radius: 0.0.into(),
      fill_mode: iced::widget::rule::FillMode::Full,
      snap: true,
    }),
    container(
      column![
        iced::widget::button(text("Modifier ce service").size(13).color(TEXT_ACCENT))
          .on_press(Message::OpenEditService(service_id))
          .style(|_, _| iced::widget::button::Style {
            background: None,
            ..Default::default()
          }),
        iced::widget::button(text("Supprimer").size(13).color(DANGER_RED),)
          .on_press(Message::DeleteService(service_id))
          .style(|_, _| iced::widget::button::Style {
            background: None,
            ..Default::default()
          }),
      ]
      .spacing(4),
    )
    .padding([12, 24]),
  ]
  .width(Length::Fill)
  .into()
}

/// Panneau de détail d'une clef SSH sélectionnée.
pub fn view_key_detail<'a>(key_id: Uuid, state: &'a ReadyState) -> Element<'a, Message> {
  let mut all_keys = state.config.keys.iter().chain(state.local_keys.iter());
  let Some(key) = all_keys.find(|k| k.id == key_id) else {
    return empty_panel();
  };

  let usage_count = count_services_using_key(key.id, &state.config.services);

  column![
    container(column![
      text("Clef SSH")
        .font(FONT_SEMIBOLD)
        .size(18)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      row![
        fingerprint_text(&key.fingerprint),
        iced::widget::Space::new().width(8),
        if key.yubikey {
          yubikey_badge()
        } else {
          iced::widget::Space::new().width(0).into()
        },
      ]
      .align_y(Alignment::Center),
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
    // Propriétés
    container(
      column![
        section_label("PROPRIÉTÉS"),
        field_row("Type", &key.key_type.to_string()),
        field_row("YubiKey", if key.yubikey { "Oui" } else { "Non" }),
        field_row("Commentaire", &key.comment),
        field_row("Créée le", &key.created_at.format("%d %b. %Y").to_string(),),
        iced::widget::Space::new().height(8),
        maybe_help(
          HelpId::Ed25519,
          "À propos de ed25519",
          "ed25519 est un algorithme de signature moderne, rapide et compact. \
           sk-ed25519 est la variante pour clef physique (YubiKey) : \
           la clef privée ne quitte jamais le périphérique matériel.",
          &state.open_helps,
        ),
      ]
      .spacing(4),
    )
    .padding([12, 24]),
    // Section sécurité — clef non protégée
    view_key_security_section(key_id, state),
    // Utilisée par
    container(
      column![
        section_label("UTILISÉE PAR"),
        if usage_count == 0 {
          let t: Element<Message> = text("Non utilisée par aucun service")
            .size(12)
            .color(TEXT_DISABLED)
            .into();
          t
        } else {
          column(
            state
              .config
              .services
              .iter()
              .filter(|s| s.active_key == Some(key.id))
              .map(|s| {
                iced::widget::button(
                  row![
                    service_type_badge(&s.service_type),
                    iced::widget::Space::new().width(8),
                    text(s.name.clone()).size(13).color(TEXT_ACCENT),
                  ]
                  .align_y(Alignment::Center),
                )
                .on_press(Message::SelectService(Some(s.id)))
                .style(|_, _| iced::widget::button::Style {
                  background: None,
                  ..Default::default()
                })
                .into()
              })
              .collect::<Vec<_>>(),
          )
          .spacing(4)
          .into()
        },
        {
          let warning: Element<Message> = if usage_count > 1 {
            container(
              text(format!(
                "⚠  Cette clef est partagée entre {usage_count} services. \
                 Envisagez une clef dédiée par service."
              ))
              .size(11)
              .color(WARNING_AMBER),
            )
            .padding([8, 0])
            .into()
          } else {
            iced::widget::Space::new().height(0).into()
          };
          warning
        },
      ]
      .spacing(4),
    )
    .padding([12, 24]),
  ]
  .width(Length::Fill)
  .height(Length::Fill)
  .into()
}

/// Panneau vide (rien de sélectionné).
pub fn view_empty_panel() -> Element<'static, Message> {
  container(
    text("Sélectionnez un service ou une clef pour voir ses détails.")
      .size(13)
      .color(TEXT_SECONDARY),
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

fn empty_panel() -> Element<'static, Message> {
  container(text("Service introuvable").size(13).color(TEXT_DISABLED))
    .padding(24)
    .into()
}

fn field_row(label: &str, value: &str) -> Element<'static, Message> {
  row![
    text(label.to_string())
      .size(11)
      .font(FONT_MEDIUM)
      .color(TEXT_SECONDARY)
      .width(Length::Fixed(100.0)),
    text(value.to_string()).size(13).color(TEXT_PRIMARY),
  ]
  .align_y(Alignment::Center)
  .into()
}
