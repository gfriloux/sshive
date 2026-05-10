#![allow(dead_code)]
#![allow(unused_imports)]
use iced::widget::{column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::app::message::Message;
use crate::ui::theme::{
  ACCENT_PRIMARY, BACKGROUND_SUBTLE, BORDER_DEFAULT, FONT_MEDIUM, TEXT_DISABLED, TEXT_SECONDARY,
};

/// Indicateur d'étapes : ●──●──○  (step courant en 1-based)
pub fn view_stepper(current: usize, total: usize) -> Element<'static, Message> {
  let mut items: Vec<Element<Message>> = Vec::new();

  for i in 1..=total {
    let is_done = i < current;
    let is_current = i == current;

    let dot = container(iced::widget::Space::new().width(0).height(0))
      .width(10)
      .height(10)
      .style(move |_| container::Style {
        background: Some(Background::Color(if is_done || is_current {
          ACCENT_PRIMARY
        } else {
          BACKGROUND_SUBTLE
        })),
        border: Border {
          radius: 5.0.into(),
          color: if is_done || is_current {
            ACCENT_PRIMARY
          } else {
            BORDER_DEFAULT
          },
          width: 1.0,
        },
        ..Default::default()
      });

    items.push(dot.into());

    if i < total {
      items.push(
        container(iced::widget::Space::new().height(1))
          .width(20)
          .style(|_| container::Style {
            background: Some(Background::Color(BORDER_DEFAULT)),
            ..Default::default()
          })
          .into(),
      );
    }
  }

  let dots = row(items).align_y(iced::Alignment::Center).spacing(0);

  let label = text(format!("Étape {current} sur {total}"))
    .size(11)
    .font(FONT_MEDIUM)
    .color(TEXT_SECONDARY);

  column![dots, iced::widget::Space::new().height(4), label]
    .align_x(iced::Alignment::Center)
    .into()
}

/// Boutons Précédent / Suivant (ou Créer) pour les formulaires multi-étapes.
pub fn step_nav(
  current: usize,
  _total: usize,
  next_label: &'static str,
  next_enabled: bool,
) -> Element<'static, Message> {
  let prev: Element<Message> = if current > 1 {
    iced::widget::button(text("← Précédent").size(13).color(TEXT_SECONDARY))
      .on_press(Message::FormStepPrev)
      .style(|_, _| iced::widget::button::Style {
        background: None,
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      })
      .into()
  } else {
    iced::widget::Space::new().width(Length::Fill).into()
  };

  let next_color = if next_enabled {
    ACCENT_PRIMARY
  } else {
    BACKGROUND_SUBTLE
  };
  let next_text_color = if next_enabled {
    crate::ui::theme::TEXT_PRIMARY
  } else {
    TEXT_DISABLED
  };

  let mut next_btn =
    iced::widget::button(text(next_label).size(13).color(next_text_color)).style(move |_, _| {
      iced::widget::button::Style {
        background: Some(Background::Color(next_color)),
        border: Border {
          radius: 6.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }
    });

  if next_enabled {
    next_btn = next_btn.on_press(Message::FormStepNext);
  }

  row![
    prev,
    iced::widget::Space::new().width(Length::Fill),
    next_btn.padding([8, 16]),
  ]
  .align_y(iced::Alignment::Center)
  .width(Length::Fill)
  .into()
}
