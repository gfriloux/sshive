use crate::config::model::{Config, SshKey};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub enum Message {
  ConfigLoaded(Result<Config, AppError>),
  KeysScanned(Result<Vec<SshKey>, AppError>),
  Navigate(NavItem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavItem {
  #[default]
  Services,
  SshKeys,
  Settings,
}
