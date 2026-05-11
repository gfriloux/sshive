use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::{FormField, HelpId, Message};
use crate::app::ServiceFormState;
use crate::config::model::{DeployMode, ServiceType};
use crate::ui::stepper::view_stepper;
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_CARD, BACKGROUND_ELEVATED, BACKGROUND_SUBTLE,
  BORDER_DEFAULT, BORDER_SUBTLE, DANGER_RED, DANGER_SUBTLE, FONT_MEDIUM, FONT_SEMIBOLD,
  TEXT_ACCENT, TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY, WARNING_AMBER, WARNING_SUBTLE,
};

pub fn view(form: &ServiceFormState) -> Element<'_, Message> {
  let title = if form.editing_id.is_some() {
    "Modifier le service"
  } else {
    "Nouveau service"
  };

  let step_content: Element<Message> = match form.step {
    1 => view_step1(form),
    2 => view_step2(form),
    3 => view_step3(form),
    _ => iced::widget::Space::new().into(),
  };

  scrollable(
    column![
      // En-tête
      container(column![
        text(title).font(FONT_SEMIBOLD).size(18).color(TEXT_PRIMARY),
        iced::widget::Space::new().height(12),
        view_stepper(form.step, 3),
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
      // Contenu de l'étape courante
      container(step_content).padding([16, 24]),
      // Bloc d'erreur
      {
        let err_block: Element<Message> = if let Some(ref err) = form.error {
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
        err_block
      },
      // Confirmation d'annulation
      {
        let cancel_confirm: Element<Message> = if form.confirm_cancel {
          container(
            row![
              text("⚠").size(12).color(WARNING_AMBER),
              iced::widget::Space::new().width(8),
              text("Modifications non enregistrées.")
                .size(12)
                .color(TEXT_PRIMARY),
              iced::widget::Space::new().width(Length::Fill),
              iced::widget::button(text("Quitter sans enregistrer").size(12).color(DANGER_RED))
                .on_press(Message::CancelFormConfirmed)
                .style(|_, _| iced::widget::button::Style {
                  background: None,
                  ..Default::default()
                })
                .padding([4, 8]),
              iced::widget::Space::new().width(8),
              iced::widget::button(text("Continuer l'édition").size(12).color(ACCENT_PRIMARY))
                .on_press(Message::CancelFormAborted)
                .style(|_, _| iced::widget::button::Style {
                  background: None,
                  ..Default::default()
                })
                .padding([4, 8]),
            ]
            .align_y(Alignment::Center),
          )
          .padding([10, 24])
          .width(Length::Fill)
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(WARNING_SUBTLE)),
            ..Default::default()
          })
          .into()
        } else {
          iced::widget::Space::new().height(0).into()
        };
        cancel_confirm
      },
      // Navigation
      container(view_step_buttons(form))
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

fn view_step1(form: &ServiceFormState) -> Element<'_, Message> {
  column![
    // Nom
    field_label("Nom du service", true),
    text_input("ex. : GitHub perso", &form.name)
      .on_input(|v| Message::FormFieldChanged(FormField::Name(v)))
      .padding(10)
      .size(13)
      .style(text_input_style),
    iced::widget::Space::new().height(4),
    text("Choisissez un nom mémorable.")
      .size(11)
      .color(TEXT_SECONDARY),
    iced::widget::Space::new().height(16),
    // Type de service
    field_label("Type de service", true),
    iced::widget::Space::new().height(8),
    type_cards(form.service_type.as_ref()),
  ]
  .spacing(2)
  .width(Length::Fill)
  .into()
}

fn view_step2(form: &ServiceFormState) -> Element<'_, Message> {
  let token_section: Element<Message> = if form.needs_api_token() {
    let token_ref = crate::app::mod_helpers::sanitize_token_ref_pub(&form.name);
    let guide = token_guide(form);
    column![
      iced::widget::Space::new().height(12),
      guide,
      iced::widget::Space::new().height(10),
      field_label("Token d'accès personnel *", true),
      text_input("ghp_… ou glpat_…", &form.token_value)
        .on_input(|v| Message::FormFieldChanged(FormField::Token(v)))
        .secure(true)
        .padding(10)
        .size(13)
        .style(text_input_style),
      iced::widget::Space::new().height(4),
      text(format!("Référence dans les secrets : {token_ref}"))
        .size(11)
        .color(TEXT_SECONDARY),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
  } else {
    iced::widget::Space::new().height(0).into()
  };

  if !form.needs_connection_params() {
    // GitHub/GitLab.com : paramètres fixes
    let (host, user) = match &form.service_type {
      Some(ServiceType::GitHub) => ("github.com", "git"),
      Some(ServiceType::GitLab) => ("gitlab.com", "git"),
      _ => ("", ""),
    };
    return column![
      container(
        column![
          text("Paramètres fixes pour ce service :")
            .size(12)
            .color(TEXT_SECONDARY),
          iced::widget::Space::new().height(8),
          param_row("Hôte", host),
          param_row("Utilisateur", user),
          param_row("Port", "22"),
        ]
        .spacing(4),
      )
      .padding([12, 14])
      .width(Length::Fill)
      .style(|_| container::Style {
        background: Some(Background::Color(ACCENT_SUBTLE)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
      token_section,
    ]
    .width(Length::Fill)
    .into();
  }

  let is_external_cm = form.deploy_mode == DeployMode::ExternalCm;
  let external_cm_toggle: Element<Message> = {
    let checked = is_external_cm;
    let (check_color, border_color, border_w) = if checked {
      (ACCENT_PRIMARY, ACCENT_PRIMARY, 2.0_f32)
    } else {
      (TEXT_DISABLED, BORDER_DEFAULT, 1.0_f32)
    };
    let toggle_msg = if checked {
      Message::FormFieldChanged(FormField::DeployMode(DeployMode::Automatic))
    } else {
      Message::FormFieldChanged(FormField::DeployMode(DeployMode::ExternalCm))
    };
    iced::widget::button(
      row![
        container(
          text(if checked { "✓" } else { "" })
            .size(11)
            .color(check_color),
        )
        .width(18)
        .height(18)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_: &iced::Theme| container::Style {
          background: Some(Background::Color(BACKGROUND_ELEVATED)),
          border: Border {
            radius: 4.0.into(),
            color: border_color,
            width: border_w,
          },
          ..Default::default()
        }),
        iced::widget::Space::new().width(10),
        column![
          text("Déploiement géré par un outil externe")
            .size(13)
            .color(TEXT_PRIMARY),
          text("NixOS, Ansible, Puppet… — SSHive affichera la clef publique à copier.")
            .size(11)
            .color(TEXT_SECONDARY),
        ]
        .spacing(2),
      ]
      .align_y(Alignment::Center),
    )
    .on_press(toggle_msg)
    .style(|_, _| iced::widget::button::Style {
      background: None,
      ..Default::default()
    })
    .into()
  };

  column![
    field_label("Hôte", false),
    text_input("ex. : prod.example.com", &form.host)
      .on_input(|v| Message::FormFieldChanged(FormField::Host(v)))
      .padding(10)
      .size(13)
      .style(text_input_style),
    iced::widget::Space::new().height(4),
    text("Adresse IP ou nom de domaine de votre serveur.")
      .size(11)
      .color(TEXT_SECONDARY),
    iced::widget::Space::new().height(12),
    row![
      column![
        field_label("Utilisateur", false),
        text_input("ex. : deploy", &form.user)
          .on_input(|v| Message::FormFieldChanged(FormField::User(v)))
          .padding(10)
          .size(13)
          .style(text_input_style),
      ]
      .width(Length::FillPortion(2))
      .spacing(2),
      iced::widget::Space::new().width(12),
      column![
        field_label("Port", false),
        text_input("22", &form.port)
          .on_input(|v| Message::FormFieldChanged(FormField::Port(v)))
          .padding(10)
          .size(13)
          .style(text_input_style),
      ]
      .width(Length::Fixed(80.0))
      .spacing(2),
    ]
    .align_y(Alignment::End),
    iced::widget::Space::new().height(12),
    iced::widget::rule::horizontal(1).style(|_: &iced::Theme| iced::widget::rule::Style {
      color: BORDER_SUBTLE,
      radius: 0.0.into(),
      fill_mode: iced::widget::rule::FillMode::Full,
      snap: true,
    }),
    iced::widget::Space::new().height(8),
    external_cm_toggle,
    token_section,
  ]
  .spacing(2)
  .width(Length::Fill)
  .into()
}

fn view_step3(form: &ServiceFormState) -> Element<'_, Message> {
  let type_str = form
    .service_type
    .as_ref()
    .map(|t| t.to_string())
    .unwrap_or_else(|| "—".to_string());

  column![
    text("Récapitulatif")
      .font(FONT_SEMIBOLD)
      .size(14)
      .color(TEXT_PRIMARY),
    iced::widget::Space::new().height(12),
    container(
      column![
        recap_row("Nom", form.name.clone()),
        recap_row("Type", type_str),
        if form.needs_connection_params() && !form.host.is_empty() {
          recap_row("Hôte", form.host.clone())
        } else {
          iced::widget::Space::new().height(0).into()
        },
        if !form.user.is_empty() {
          recap_row("Utilisateur", form.user.clone())
        } else {
          iced::widget::Space::new().height(0).into()
        },
        if form.needs_api_token() && !form.token_value.is_empty() {
          recap_row("Token", "●●●●●●●●".to_string())
        } else {
          iced::widget::Space::new().height(0).into()
        },
      ]
      .spacing(6),
    )
    .padding([12, 14])
    .width(Length::Fill)
    .style(|_| container::Style {
      background: Some(Background::Color(BACKGROUND_CARD)),
      border: Border {
        radius: 6.0.into(),
        ..Default::default()
      },
      ..Default::default()
    }),
    iced::widget::Space::new().height(12),
    text("La clef SSH pourra être générée depuis le panneau de détail du service.")
      .size(11)
      .color(TEXT_SECONDARY),
  ]
  .spacing(2)
  .width(Length::Fill)
  .into()
}

fn view_step_buttons(form: &ServiceFormState) -> Element<'_, Message> {
  let step = form.step;

  let next_label: &'static str = if step == 3 {
    if form.editing_id.is_some() {
      "Enregistrer"
    } else {
      "Créer le service"
    }
  } else {
    "Suivant →"
  };

  let next_enabled = match step {
    1 => form.step1_valid(),
    2 | 3 => true,
    _ => false,
  };

  let cancel_btn = iced::widget::button(text("Annuler").size(13).color(TEXT_SECONDARY))
    .on_press(Message::CancelFormRequested)
    .style(|_, status| iced::widget::button::Style {
      background: Some(Background::Color(
        if matches!(status, iced::widget::button::Status::Hovered) {
          BACKGROUND_ELEVATED
        } else {
          iced::Color::TRANSPARENT
        },
      )),
      ..Default::default()
    })
    .padding([8, 12]);

  let left: Element<Message> = if step > 1 {
    row![
      cancel_btn,
      iced::widget::Space::new().width(4),
      iced::widget::button(text("← Précédent").size(13).color(TEXT_SECONDARY))
        .on_press(Message::FormStepPrev)
        .style(|_, _| iced::widget::button::Style {
          background: None,
          ..Default::default()
        }),
    ]
    .align_y(Alignment::Center)
    .into()
  } else {
    cancel_btn.into()
  };

  let next_bg = if next_enabled {
    ACCENT_PRIMARY
  } else {
    BACKGROUND_SUBTLE
  };
  let next_fg = if next_enabled {
    TEXT_PRIMARY
  } else {
    TEXT_DISABLED
  };
  let next_msg = if step == 3 {
    Message::SubmitServiceForm
  } else {
    Message::FormStepNext
  };

  let mut next = iced::widget::button(text(next_label).size(13).color(next_fg))
    .style(move |_, _| iced::widget::button::Style {
      background: Some(Background::Color(next_bg)),
      border: Border {
        radius: 6.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .padding([8, 16]);

  if next_enabled {
    next = next.on_press(next_msg);
  }

  row![left, iced::widget::Space::new().width(Length::Fill), next]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn type_cards(selected: Option<&ServiceType>) -> Element<'static, Message> {
  let types = [
    (ServiceType::GitHub, "GH", "GitHub"),
    (ServiceType::GitLab, "GL", "GitLab.com"),
    (ServiceType::GitLabSelfHosted, "GL*", "GitLab self-hosted"),
    (ServiceType::SshGeneric, "SSH", "SSH générique"),
  ];

  let cards: Vec<Element<Message>> = types
    .iter()
    .map(|(t, badge, label)| {
      let is_sel = selected == Some(t);
      let bg = if is_sel {
        ACCENT_SUBTLE
      } else {
        BACKGROUND_ELEVATED
      };
      let border_c = if is_sel {
        ACCENT_PRIMARY
      } else {
        BORDER_DEFAULT
      };
      let border_w = if is_sel { 2.0_f32 } else { 1.0 };
      let t_clone = t.clone();

      iced::widget::button(
        container(
          row![
            container(
              text(*badge)
                .size(10)
                .font(FONT_MEDIUM)
                .color(TEXT_SECONDARY),
            )
            .padding([2, 6])
            .style(|_| container::Style {
              background: Some(Background::Color(BACKGROUND_CARD)),
              border: Border {
                radius: 3.0.into(),
                ..Default::default()
              },
              ..Default::default()
            }),
            iced::widget::Space::new().width(8),
            text(*label).size(13).color(TEXT_PRIMARY),
          ]
          .align_y(Alignment::Center),
        )
        .padding([10, 12])
        .width(Length::Fill),
      )
      .on_press(Message::FormFieldChanged(FormField::ServiceType(t_clone)))
      .style(move |_, _| iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        border: Border {
          radius: 6.0.into(),
          color: border_c,
          width: border_w,
        },
        ..Default::default()
      })
      .width(Length::Fill)
      .into()
    })
    .collect();

  column(cards).spacing(6).width(Length::Fill).into()
}

fn field_label(label: &str, required: bool) -> Element<'static, Message> {
  let suffix = if required { " *" } else { "" };
  text(format!("{label}{suffix}"))
    .size(12)
    .font(FONT_MEDIUM)
    .color(TEXT_PRIMARY)
    .into()
}

fn param_row(label: &str, value: &str) -> Element<'static, Message> {
  row![
    text(label.to_string())
      .size(12)
      .color(TEXT_SECONDARY)
      .width(Length::Fixed(90.0)),
    text(value.to_string()).size(12).color(TEXT_PRIMARY),
  ]
  .align_y(Alignment::Center)
  .into()
}

fn recap_row(label: &'static str, value: impl Into<String>) -> Element<'static, Message> {
  let v = value.into();
  row![
    text(label)
      .size(12)
      .color(TEXT_SECONDARY)
      .width(Length::Fixed(100.0)),
    text(v).size(13).color(TEXT_PRIMARY),
  ]
  .align_y(Alignment::Center)
  .into()
}

fn text_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
  text_input::Style {
    background: Background::Color(BACKGROUND_ELEVATED),
    border: Border {
      radius: 6.0.into(),
      color: match status {
        text_input::Status::Focused { .. } => ACCENT_PRIMARY,
        _ => BORDER_DEFAULT,
      },
      width: if matches!(status, text_input::Status::Focused { .. }) {
        2.0
      } else {
        1.0
      },
    },
    icon: TEXT_DISABLED,
    placeholder: TEXT_DISABLED,
    value: TEXT_PRIMARY,
    selection: ACCENT_SUBTLE,
  }
}

fn token_guide(form: &ServiceFormState) -> Element<'_, Message> {
  let _ = HelpId::TokenGuideGitHub; // évite le warning unused

  let (title, full_url, scope_line) = match &form.service_type {
    Some(ServiceType::GitHub) => (
      "Créer un token GitHub (Personal Access Token classique)",
      "https://github.com/settings/tokens/new".to_string(),
      "Scope requis : admin:public_key",
    ),
    Some(ServiceType::GitLab) => (
      "Créer un token GitLab.com",
      "https://gitlab.com/-/user_settings/personal_access_tokens".to_string(),
      "Scope requis : api",
    ),
    Some(ServiceType::GitLabSelfHosted) => {
      let base = form.host.trim_end_matches('/');
      let url = if base.is_empty() {
        "<votre-instance>/-/user_settings/personal_access_tokens".to_string()
      } else if base.starts_with("http") {
        format!("{base}/-/user_settings/personal_access_tokens")
      } else {
        format!("https://{base}/-/user_settings/personal_access_tokens")
      };
      (
        "Créer un token GitLab (self-hosted)",
        url,
        "Scope requis : api",
      )
    }
    _ => return iced::widget::Space::new().height(0).into(),
  };

  let url_clone = full_url.clone();

  container(
    column![
      text(title).size(11).font(FONT_MEDIUM).color(TEXT_SECONDARY),
      iced::widget::Space::new().height(6),
      // URL cliquable + bouton copier
      row![
        iced::widget::button(
          text("Ouvrir dans le navigateur")
            .size(11)
            .color(TEXT_ACCENT)
        )
        .on_press(Message::OpenUrl(full_url.clone()))
        .style(|_, _| iced::widget::button::Style {
          background: None,
          ..Default::default()
        })
        .padding([0, 0]),
        iced::widget::Space::new().width(12),
        iced::widget::button(text("Copier l'URL").size(11).color(TEXT_SECONDARY))
          .on_press(Message::CopyText(url_clone))
          .style(|_, status| iced::widget::button::Style {
            background: Some(Background::Color(
              if matches!(status, iced::widget::button::Status::Hovered) {
                BACKGROUND_ELEVATED
              } else {
                iced::Color::TRANSPARENT
              },
            )),
            ..Default::default()
          })
          .padding([0, 4]),
      ]
      .align_y(Alignment::Center),
      iced::widget::Space::new().height(4),
      text("Durée de vie recommandée : 90 jours ou moins.")
        .size(11)
        .color(TEXT_SECONDARY),
      text(scope_line).size(11).color(TEXT_SECONDARY),
      text("Générez et copiez le token. Il ne sera plus affiché après.")
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
  })
  .into()
}
