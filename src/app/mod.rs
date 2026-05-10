pub mod message;

use iced::{Element, Subscription, Task};

use crate::app::message::{Message, NavItem};
use crate::config::model::{Config, SshKey};
use crate::error::AppError;

#[derive(Debug, Default)]
pub struct App {
  pub state: AppState,
}

#[derive(Debug, Default)]
pub enum AppState {
  #[default]
  Loading,
  Ready(ReadyState),
  Error(AppError),
}

#[derive(Debug, Default)]
pub struct ReadyState {
  pub config: Config,
  pub local_keys: Vec<SshKey>,
  pub active_nav: NavItem,
}

impl App {
  pub fn new() -> (Self, Task<Message>) {
    let task = Task::perform(
      crate::config::loader::load_or_create(),
      Message::ConfigLoaded,
    );
    (Self::default(), task)
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::ConfigLoaded(Ok(config)) => {
        self.state = AppState::Ready(ReadyState {
          config,
          ..ReadyState::default()
        });
        Task::perform(
          crate::config::ssh_scanner::scan_pub_keys(),
          Message::KeysScanned,
        )
      }

      Message::ConfigLoaded(Err(e)) => {
        self.state = AppState::Error(e);
        Task::none()
      }

      Message::KeysScanned(Ok(keys)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.local_keys = keys;
        }
        Task::none()
      }

      Message::KeysScanned(Err(e)) => {
        tracing::warn!("Scan des clefs SSH échoué : {e}");
        Task::none()
      }

      Message::Navigate(nav) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.active_nav = nav;
        }
        Task::none()
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    match &self.state {
      AppState::Loading => crate::ui::view_loading(),
      AppState::Ready(s) => crate::ui::view_ready(s),
      AppState::Error(e) => crate::ui::view_error(e),
    }
  }

  pub fn subscription(&self) -> Subscription<Message> {
    Subscription::none()
  }
}
