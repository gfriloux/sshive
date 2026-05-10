#![deny(unsafe_code)]

mod app;
mod config;
mod error;
mod regression_v010;
mod secrets;
mod security;
mod subprocess;
mod ui;

use app::App;
use ui::theme::{FONT_INTER_BYTES, FONT_JBMONO_BYTES};

fn main() -> iced::Result {
  security::harden_process();

  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("sshive=debug".parse().expect("directive valide")),
    )
    .init();

  iced::application(App::new, App::update, App::view)
    .title("SSHive")
    .subscription(App::subscription)
    .theme(|_: &App| ui::theme::sshive_theme())
    .window_size((1024.0, 700.0))
    .font(FONT_INTER_BYTES)
    .font(FONT_JBMONO_BYTES)
    .run()
}
