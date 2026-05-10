use std::sync::Arc;

use iced::font::{Family, Weight};
use iced::theme::Custom;
use iced::{Color, Font, Theme};

// --- Polices ---

pub const FONT_INTER_BYTES: &[u8] = include_bytes!("../../assets/fonts/InterVariable.ttf");
pub const FONT_JBMONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");

pub const FONT_MEDIUM: Font = Font {
  family: Family::Name("Inter"),
  weight: Weight::Medium,
  ..Font::DEFAULT
};

pub const FONT_SEMIBOLD: Font = Font {
  family: Family::Name("Inter"),
  weight: Weight::Semibold,
  ..Font::DEFAULT
};

pub const FONT_MONO: Font = Font {
  family: Family::Name("JetBrains Mono"),
  weight: Weight::Normal,
  ..Font::DEFAULT
};

// --- Surfaces ---

pub const BACKGROUND_BASE: Color = Color {
  r: 0.059,
  g: 0.067,
  b: 0.090,
  a: 1.0,
}; // #0F1117

pub const BACKGROUND_ELEVATED: Color = Color {
  r: 0.086,
  g: 0.106,
  b: 0.153,
  a: 1.0,
}; // #161B27

pub const BACKGROUND_CARD: Color = Color {
  r: 0.110,
  g: 0.137,
  b: 0.200,
  a: 1.0,
}; // #1C2333

#[allow(dead_code)]
pub const BACKGROUND_SUBTLE: Color = Color {
  r: 0.141,
  g: 0.176,
  b: 0.251,
  a: 1.0,
}; // #242D40

// --- Texte ---

pub const TEXT_PRIMARY: Color = Color {
  r: 0.910,
  g: 0.922,
  b: 0.941,
  a: 1.0,
}; // #E8EBF0

pub const TEXT_SECONDARY: Color = Color {
  r: 0.541,
  g: 0.592,
  b: 0.659,
  a: 1.0,
}; // #8A97A8

pub const TEXT_DISABLED: Color = Color {
  r: 0.290,
  g: 0.333,
  b: 0.408,
  a: 1.0,
}; // #4A5568

pub const TEXT_ACCENT: Color = Color {
  r: 0.494,
  g: 0.722,
  b: 0.969,
  a: 1.0,
}; // #7EB8F7

// --- Accent ---

pub const ACCENT_PRIMARY: Color = Color {
  r: 0.231,
  g: 0.510,
  b: 0.965,
  a: 1.0,
}; // #3B82F6

pub const ACCENT_HOVER: Color = Color {
  r: 0.145,
  g: 0.400,
  b: 0.922,
  a: 1.0,
}; // #2563EB

pub const ACCENT_SUBTLE: Color = Color {
  r: 0.118,
  g: 0.227,
  b: 0.373,
  a: 1.0,
}; // #1E3A5F

// --- Sémantiques ---

pub const SUCCESS_GREEN: Color = Color {
  r: 0.133,
  g: 0.773,
  b: 0.369,
  a: 1.0,
}; // #22C55E

pub const SUCCESS_SUBTLE: Color = Color {
  r: 0.078,
  g: 0.325,
  b: 0.173,
  a: 1.0,
}; // #14532D

pub const WARNING_AMBER: Color = Color {
  r: 0.961,
  g: 0.620,
  b: 0.043,
  a: 1.0,
}; // #F59E0B

pub const WARNING_SUBTLE: Color = Color {
  r: 0.271,
  g: 0.102,
  b: 0.012,
  a: 1.0,
}; // #451A03

pub const DANGER_RED: Color = Color {
  r: 0.937,
  g: 0.267,
  b: 0.267,
  a: 1.0,
}; // #EF4444

pub const DANGER_SUBTLE: Color = Color {
  r: 0.271,
  g: 0.039,
  b: 0.039,
  a: 1.0,
}; // #450A0A

// --- Borders ---

pub const BORDER_DEFAULT: Color = Color {
  r: 0.165,
  g: 0.200,
  b: 0.282,
  a: 1.0,
}; // #2A3348

pub const BORDER_SUBTLE: Color = Color {
  r: 0.110,
  g: 0.153,
  b: 0.251,
  a: 1.0,
}; // #1C2740

// --- Thème iced ---

pub fn sshive_theme() -> Theme {
  Theme::Custom(Arc::new(Custom::new(
    "SSHive".to_string(),
    iced::theme::Palette {
      background: BACKGROUND_BASE,
      text: TEXT_PRIMARY,
      primary: ACCENT_PRIMARY,
      success: SUCCESS_GREEN,
      danger: DANGER_RED,
      warning: WARNING_AMBER,
    },
  )))
}
