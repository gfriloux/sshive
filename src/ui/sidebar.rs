use iced::widget::{column, container, row, rule, text};
use iced::{Alignment, Background, Border, Element, Length, Padding};

use crate::app::message::{Message, NavItem};
use crate::ui::theme::{
  ACCENT_PRIMARY, ACCENT_SUBTLE, BACKGROUND_CARD, BACKGROUND_ELEVATED, BORDER_DEFAULT, FONT_MEDIUM,
  FONT_SEMIBOLD, TEXT_ACCENT, TEXT_PRIMARY, TEXT_SECONDARY,
};

pub fn view_sidebar(active: NavItem) -> Element<'static, Message> {
  let logo = container(
    text("SSHive")
      .font(FONT_SEMIBOLD)
      .size(16)
      .color(TEXT_PRIMARY),
  )
  .padding(Padding {
    top: 20.0,
    right: 16.0,
    bottom: 16.0,
    left: 16.0,
  });

  let divider = rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
    color: BORDER_DEFAULT,
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: true,
  });

  let nav = column![
    nav_item("Services", NavItem::Services, active),
    nav_item("SSH Keys", NavItem::SshKeys, active),
  ]
  .spacing(2)
  .padding(Padding {
    top: 8.0,
    right: 8.0,
    bottom: 0.0,
    left: 8.0,
  });

  let settings = container(nav_item("Paramètres", NavItem::Settings, active)).padding(Padding {
    top: 0.0,
    right: 8.0,
    bottom: 16.0,
    left: 8.0,
  });

  container(
    column![
      logo,
      divider,
      nav,
      iced::widget::Space::new().height(Length::Fill),
      settings,
    ]
    .width(Length::Fill),
  )
  .width(220)
  .height(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(BACKGROUND_ELEVATED)),
    border: Border {
      color: BORDER_DEFAULT,
      width: 1.0,
      radius: 0.0.into(),
    },
    ..Default::default()
  })
  .into()
}

fn nav_item(label: &'static str, item: NavItem, active: NavItem) -> Element<'static, Message> {
  let is_active = item == active;

  let indicator = container(iced::widget::Space::new().height(20))
    .width(3)
    .style(move |_| container::Style {
      background: if is_active {
        Some(Background::Color(ACCENT_PRIMARY))
      } else {
        None
      },
      border: Border {
        radius: 2.0.into(),
        ..Default::default()
      },
      ..Default::default()
    });

  let label_widget = text(label)
    .size(14)
    .font(if is_active {
      FONT_SEMIBOLD
    } else {
      FONT_MEDIUM
    })
    .color(if is_active {
      TEXT_ACCENT
    } else {
      TEXT_SECONDARY
    });

  let content =
    row![indicator, iced::widget::Space::new().width(8), label_widget,].align_y(Alignment::Center);

  let msg = Message::Navigate(item);

  iced::widget::button(container(content).padding([8, 12]).width(Length::Fill))
    .on_press(msg)
    .style(move |_, status| iced::widget::button::Style {
      background: if is_active {
        Some(Background::Color(ACCENT_SUBTLE))
      } else if matches!(status, iced::widget::button::Status::Hovered) {
        Some(Background::Color(BACKGROUND_CARD))
      } else {
        None
      },
      border: Border {
        radius: 6.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .width(Length::Fill)
    .into()
}
