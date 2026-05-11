/// Flux de déploiement SSH — deux modes : automatique et guidé.
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::Message;
use crate::app::{DeployState, DeployStep};
use crate::config::model::DeployMode;
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_BASE, BACKGROUND_CARD, BACKGROUND_ELEVATED,
  BORDER_DEFAULT, DANGER_RED, DANGER_SUBTLE, FONT_MEDIUM, FONT_MONO, FONT_SEMIBOLD, SUCCESS_GREEN,
  SUCCESS_SUBTLE, TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY,
};

pub fn view(state: &DeployState) -> Element<'_, Message> {
  match &state.step {
    DeployStep::SkPreFlight => view_sk_preflight(),
    DeployStep::Preflight => view_preflight(),
    DeployStep::ChooseMode => view_choose_mode(),
    DeployStep::AutoDeploying => view_auto_progress(&[]),
    DeployStep::GuidedCommand { command } => view_guided_command(command, state.service_id),
    DeployStep::ExternalCmDeploy { public_key } => view_external_cm_deploy(public_key),
    DeployStep::Verifying => view_verifying(),
    DeployStep::Success { verified } => view_success(*verified),
    DeployStep::Error(e) => view_error(e),
  }
}

pub fn view_blocker(blocker: &crate::app::DeployBlocker) -> Element<'static, Message> {
  use crate::app::DeployBlocker;

  let (title, body, show_configure): (&'static str, String, bool) = match blocker {
    DeployBlocker::MissingToken { service_name } => (
      "Token d'accès manquant",
      format!(
        "Le service « {service_name} » n'a pas de token d'accès configuré.\n\
         Un token est requis pour déployer, vérifier ou révoquer des clefs via l'API."
      ),
      true,
    ),
    DeployBlocker::InvalidTokenFormat {
      service_name,
      reason,
    } => (
      "Format de token invalide",
      format!("Le token du service « {service_name} » semble mal formé.\n{reason}"),
      true,
    ),
    DeployBlocker::TokenProbeUnauthorized { service_name } => (
      "Token refusé par l'API",
      format!(
        "Le token du service « {service_name} » a été refusé (401/403).\n\
         Vérifiez qu'il est valide, non expiré, et possède les droits requis."
      ),
      true,
    ),
    DeployBlocker::TokenProbeError {
      service_name,
      message,
    } => (
      "Erreur lors de la vérification",
      format!("Impossible de joindre l'API pour le service « {service_name} ».\n{message}"),
      false,
    ),
  };

  let service_id_opt: Option<uuid::Uuid> = match blocker {
    DeployBlocker::MissingToken { .. }
    | DeployBlocker::InvalidTokenFormat { .. }
    | DeployBlocker::TokenProbeUnauthorized { .. }
    | DeployBlocker::TokenProbeError { .. } => None,
  };

  let configure_btn: Element<Message> = if show_configure {
    if let Some(sid) = service_id_opt {
      iced::widget::button(text("Configurer le token").size(13).color(TEXT_PRIMARY))
        .on_press(Message::OpenEditServiceAtStep {
          service_id: sid,
          step: 2,
        })
        .style(|_, _| iced::widget::button::Style {
          background: Some(Background::Color(ACCENT_PRIMARY)),
          border: Border {
            radius: 6.0.into(),
            ..Default::default()
          },
          ..Default::default()
        })
        .padding([8, 16])
        .into()
    } else {
      iced::widget::Space::new().height(0).into()
    }
  } else {
    iced::widget::Space::new().height(0).into()
  };

  container(
    column![
      text(title)
        .font(crate::ui::theme::FONT_SEMIBOLD)
        .size(15)
        .color(crate::ui::theme::DANGER_RED),
      iced::widget::Space::new().height(10),
      container(text(body).size(12).color(TEXT_PRIMARY),)
        .padding([10, 14])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(DANGER_SUBTLE)),
          border: Border {
            radius: 6.0.into(),
            ..Default::default()
          },
          ..Default::default()
        }),
      iced::widget::Space::new().height(16),
      row![
        configure_btn,
        iced::widget::Space::new().width(Length::Fill),
        iced::widget::button(text("Annuler").size(13).color(TEXT_SECONDARY))
          .on_press(Message::DismissDeployBlocker)
          .style(|_, _| iced::widget::button::Style {
            background: None,
            ..Default::default()
          }),
      ]
      .align_y(iced::Alignment::Center),
    ]
    .spacing(0),
  )
  .padding(24)
  .width(Length::Fill)
  .into()
}

fn view_external_cm_deploy(public_key: &str) -> Element<'_, Message> {
  let key = public_key.to_string();

  column![
    container(
      text("Déploiement — clef publique")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
    )
    .padding(iced::Padding {
      top: 24.0,
      right: 24.0,
      bottom: 8.0,
      left: 24.0
    }),
    container(
      column![
        text("Copiez cette clef dans votre configuration :")
          .size(13)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().height(8),
        container(text(key).size(11).font(FONT_MONO).color(TEXT_PRIMARY),)
          .padding([12, 14])
          .width(Length::Fill)
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(BACKGROUND_CARD)),
            border: Border {
              radius: 6.0.into(),
              ..Default::default()
            },
            ..Default::default()
          }),
        iced::widget::Space::new().height(12),
        text("Étapes :")
          .size(12)
          .font(FONT_MEDIUM)
          .color(TEXT_PRIMARY),
        text("1. Ajoutez cette clef dans votre configuration.")
          .size(12)
          .color(TEXT_SECONDARY),
        text("2. Appliquez la configuration sur le serveur cible.")
          .size(12)
          .color(TEXT_SECONDARY),
        text("3. Vérifiez que la connexion SSH fonctionne.")
          .size(12)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().height(12),
        container(
          column![
            text("ℹ  SSHive ne vérifiera pas ce déploiement automatiquement.")
              .size(11)
              .color(TEXT_SECONDARY),
            text("Testez la connexion avant de cliquer « Déploiement effectué ».")
              .size(11)
              .color(TEXT_SECONDARY),
          ]
          .spacing(2),
        )
        .padding([10, 12])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(ACCENT_SUBTLE)),
          border: Border {
            radius: 6.0.into(),
            ..Default::default()
          },
          ..Default::default()
        }),
        iced::widget::Space::new().height(16),
        iced::widget::button(text("Déploiement effectué →").size(13).color(TEXT_PRIMARY),)
          .on_press(Message::GuidedDeployConfirmed)
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
      .spacing(4),
    )
    .padding(iced::Padding {
      top: 0.0,
      right: 24.0,
      bottom: 24.0,
      left: 24.0
    }),
  ]
  .width(Length::Fill)
  .into()
}

fn view_sk_preflight() -> Element<'static, Message> {
  container(
    column![
      text("Rotation d'une clef YubiKey")
        .font(crate::ui::theme::FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(12),
      container(
        column![
          text("⚠  Votre YubiKey doit rester branchée pendant toute l'opération.")
            .size(12)
            .color(TEXT_PRIMARY),
          iced::widget::Space::new().height(6),
          text("Vous serez invité à la toucher lors de la génération et de la vérification.")
            .size(11)
            .color(TEXT_SECONDARY),
          text("Ne retirez pas la YubiKey avant que l'opération soit terminée.")
            .size(11)
            .color(TEXT_SECONDARY),
        ]
        .spacing(2),
      )
      .padding([12, 14])
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(crate::ui::theme::WARNING_SUBTLE)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
      iced::widget::Space::new().height(20),
      row![
        iced::widget::button(text("Annuler").size(13).color(TEXT_SECONDARY))
          .on_press(Message::CloseDeployFlow)
          .style(|_, _| iced::widget::button::Style {
            background: None,
            ..Default::default()
          }),
        iced::widget::Space::new().width(Length::Fill),
        iced::widget::button(text("Commencer →").size(13).color(TEXT_PRIMARY))
          .on_press(Message::SkPreFlightConfirmed)
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
      .align_y(iced::Alignment::Center),
    ]
    .spacing(0),
  )
  .padding(24)
  .width(Length::Fill)
  .height(Length::Fill)
  .into()
}

fn view_preflight() -> Element<'static, Message> {
  container(
    column![
      text("Vérification du token en cours…")
        .font(crate::ui::theme::FONT_SEMIBOLD)
        .size(14)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Connexion à l'API pour valider les droits d'accès…")
        .size(12)
        .color(TEXT_SECONDARY),
    ]
    .align_x(iced::Alignment::Center),
  )
  .width(Length::Fill)
  .height(Length::Fill)
  .align_x(iced::alignment::Horizontal::Center)
  .align_y(iced::alignment::Vertical::Center)
  .padding(24)
  .into()
}

fn view_choose_mode() -> Element<'static, Message> {
  let body = column![
    container(
      text("Déployer la clef SSH")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
    )
    .padding(iced::Padding {
      top: 24.0,
      right: 24.0,
      bottom: 16.0,
      left: 24.0
    }),
    container(
      column![
        text("Comment voulez-vous déployer votre clef ?")
          .size(13)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().height(12),
        // Mode automatique
        iced::widget::button(
          container(
            column![
              row![
                text("Automatique")
                  .font(FONT_SEMIBOLD)
                  .size(14)
                  .color(TEXT_PRIMARY),
                iced::widget::Space::new().width(8),
                container(
                  text("RECOMMANDÉ")
                    .size(10)
                    .font(FONT_MEDIUM)
                    .color(SUCCESS_GREEN)
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
              text("SSHive copie votre clef sur le serveur à votre place.")
                .size(12)
                .color(TEXT_SECONDARY),
              text("Utilise : ssh-copy-id").size(11).color(TEXT_DISABLED),
            ]
            .spacing(0),
          )
          .padding([16, 16])
          .width(Length::Fill),
        )
        .on_press(Message::DeployModeSelected(DeployMode::Automatic))
        .style(|_, status| iced::widget::button::Style {
          background: Some(Background::Color(
            if matches!(status, iced::widget::button::Status::Hovered) {
              ACCENT_SUBTLE
            } else {
              BACKGROUND_ELEVATED
            }
          )),
          border: Border {
            radius: 8.0.into(),
            color: BORDER_DEFAULT,
            width: 1.0
          },
          ..Default::default()
        })
        .width(Length::Fill),
        iced::widget::Space::new().height(8),
        // Mode guidé
        iced::widget::button(
          container(
            column![
              text("Guidé")
                .font(FONT_SEMIBOLD)
                .size(14)
                .color(TEXT_PRIMARY),
              iced::widget::Space::new().height(6),
              text(
                "SSHive affiche la commande exacte à copier dans votre terminal. \
                 Vous gardez le contrôle."
              )
              .size(12)
              .color(TEXT_SECONDARY),
              text("Utile si : bastion, accès restreint, ou préférence.")
                .size(11)
                .color(TEXT_DISABLED),
            ]
            .spacing(0),
          )
          .padding([16, 16])
          .width(Length::Fill),
        )
        .on_press(Message::DeployModeSelected(DeployMode::Guided))
        .style(|_, status| iced::widget::button::Style {
          background: Some(Background::Color(
            if matches!(status, iced::widget::button::Status::Hovered) {
              ACCENT_SUBTLE
            } else {
              BACKGROUND_ELEVATED
            }
          )),
          border: Border {
            radius: 8.0.into(),
            color: BORDER_DEFAULT,
            width: 1.0
          },
          ..Default::default()
        })
        .width(Length::Fill),
      ]
      .spacing(0),
    )
    .padding([0, 24]),
  ]
  .width(Length::Fill);

  container(body)
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(BACKGROUND_BASE)),
      ..Default::default()
    })
    .into()
}

/// Affiche la commande ssh-copy-id à copier (mode guidé).
fn view_guided_command(command: &str, _service_id: uuid::Uuid) -> Element<'static, Message> {
  let cmd = command.to_string();

  column![
    container(
      text("Déploiement guidé")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
    )
    .padding(iced::Padding {
      top: 24.0,
      right: 24.0,
      bottom: 8.0,
      left: 24.0
    }),
    container(
      column![
        text("Copiez cette commande dans votre terminal :")
          .size(13)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().height(8),
        container(
          text(cmd.clone())
            .size(11)
            .font(FONT_MONO)
            .color(TEXT_PRIMARY),
        )
        .padding([12, 14])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(BACKGROUND_CARD)),
          border: Border {
            radius: 6.0.into(),
            ..Default::default()
          },
          ..Default::default()
        }),
        iced::widget::Space::new().height(12),
        text("Étapes :")
          .size(12)
          .font(FONT_MEDIUM)
          .color(TEXT_PRIMARY),
        text("1. Ouvrez un terminal").size(12).color(TEXT_SECONDARY),
        text("2. Collez la commande ci-dessus")
          .size(12)
          .color(TEXT_SECONDARY),
        text("3. Saisissez votre mot de passe si demandé")
          .size(12)
          .color(TEXT_SECONDARY),
        iced::widget::Space::new().height(16),
        iced::widget::button(text("C'est fait →").size(13).color(TEXT_PRIMARY),)
          .on_press(Message::GuidedDeployConfirmed)
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
      .spacing(4),
    )
    .padding(iced::Padding {
      top: 0.0,
      right: 24.0,
      bottom: 24.0,
      left: 24.0
    }),
  ]
  .width(Length::Fill)
  .into()
}

/// Progression du déploiement automatique.
fn view_auto_progress(log: &[String]) -> Element<'_, Message> {
  let log_items = column(
    log
      .iter()
      .map(|line| {
        text(line.clone())
          .size(11)
          .font(FONT_MONO)
          .color(TEXT_SECONDARY)
          .into()
      })
      .collect::<Vec<_>>(),
  )
  .spacing(2);

  column![
    container(
      text("Déploiement en cours…")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
    )
    .padding(iced::Padding {
      top: 24.0,
      right: 24.0,
      bottom: 16.0,
      left: 24.0
    }),
    container(scrollable(log_items).height(Length::Fixed(200.0)),)
      .padding(iced::Padding {
        top: 0.0,
        right: 24.0,
        bottom: 24.0,
        left: 24.0
      })
      .width(Length::Fill),
  ]
  .width(Length::Fill)
  .into()
}

fn view_verifying() -> Element<'static, Message> {
  container(
    column![
      text("Vérification de la connexion…")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(TEXT_PRIMARY),
      iced::widget::Space::new().height(8),
      text("Test de connexion SSH avec la nouvelle clef…")
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

fn view_success(verified: bool) -> Element<'static, Message> {
  let status: Element<Message> = if verified {
    text("✓ Connexion vérifiée avec succès.")
      .size(12)
      .color(SUCCESS_GREEN)
      .into()
  } else {
    text("⚠ Vérification non effectuée — connectez-vous manuellement pour confirmer.")
      .size(12)
      .color(crate::ui::theme::WARNING_AMBER)
      .into()
  };

  container(
    column![
      text("Clef déployée avec succès")
        .font(FONT_SEMIBOLD)
        .size(16)
        .color(SUCCESS_GREEN),
      iced::widget::Space::new().height(8),
      status,
      iced::widget::Space::new().height(24),
      iced::widget::button(text("Fermer").size(13).color(TEXT_PRIMARY))
        .on_press(Message::CloseDeployFlow)
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

fn view_error(error: &str) -> Element<'static, Message> {
  container(
    column![
      container(column![
        text("Échec du déploiement")
          .font(FONT_SEMIBOLD)
          .size(14)
          .color(DANGER_RED),
        iced::widget::Space::new().height(8),
        text(error.to_string()).size(12).color(TEXT_SECONDARY),
      ],)
      .padding([12, 16])
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(DANGER_SUBTLE)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
      iced::widget::Space::new().height(16),
      iced::widget::button(text("Fermer").size(13).color(TEXT_SECONDARY))
        .on_press(Message::CloseDeployFlow)
        .style(|_, _| iced::widget::button::Style {
          background: None,
          ..Default::default()
        }),
    ]
    .spacing(0),
  )
  .padding(24)
  .into()
}
