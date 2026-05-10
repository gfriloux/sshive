use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Element, Length};

use crate::app::message::{FormField, Message};
use crate::app::ServiceFormState;
use crate::config::model::ServiceType;
use crate::ui::stepper::view_stepper;
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_CARD, BACKGROUND_ELEVATED, BACKGROUND_SUBTLE,
  BORDER_DEFAULT, DANGER_RED, DANGER_SUBTLE, FONT_MEDIUM, FONT_SEMIBOLD, TEXT_DISABLED,
  TEXT_PRIMARY, TEXT_SECONDARY,
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

  let next_label = if form.step == 3 {
    if form.editing_id.is_some() {
      "Enregistrer"
    } else {
      "Créer le service"
    }
  } else {
    "Suivant →"
  };

  let next_enabled = match form.step {
    1 => form.step1_valid(),
    2 => true,
    3 => true,
    _ => false,
  };

  // Sur l'étape 3, Suivant soumet le formulaire
  // Sur les autres, il avance d'une étape

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
      // Navigation
      container(view_step_buttons(form.step, next_label, next_enabled))
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
    column![
      iced::widget::Space::new().height(12),
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
          text("ℹ  Paramètres fixes pour ce service :")
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

fn view_step_buttons(
  step: usize,
  next_label: &'static str,
  next_enabled: bool,
) -> Element<'static, Message> {
  let prev: Element<Message> = if step > 1 {
    iced::widget::button(text("← Précédent").size(13).color(TEXT_SECONDARY))
      .on_press(Message::FormStepPrev)
      .style(|_, _| iced::widget::button::Style {
        background: None,
        ..Default::default()
      })
      .into()
  } else {
    iced::widget::Space::new().width(Length::Fill).into()
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

  row![prev, iced::widget::Space::new().width(Length::Fill), next,]
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
    (ServiceType::Manual, "M", "Manuel"),
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
