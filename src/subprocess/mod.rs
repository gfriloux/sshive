pub mod gpg;
pub mod pinentry;
pub mod ssh_copy_id;
pub mod ssh_key_reencrypt;
pub mod ssh_keygen;

use async_trait::async_trait;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ProcessOutput {
  pub exit_code: i32,
  pub stdout: String,
  pub stderr: String,
}

#[async_trait]
pub trait CommandRunner: Send + Sync {
  async fn run(
    &self,
    program: &str,
    args: &[&str],
    timeout_secs: u64,
  ) -> Result<ProcessOutput, AppError>;
}

pub struct RealRunner;

#[async_trait]
impl CommandRunner for RealRunner {
  async fn run(
    &self,
    program: &str,
    args: &[&str],
    timeout_secs: u64,
  ) -> Result<ProcessOutput, AppError> {
    let child = tokio::process::Command::new(program)
      .args(args)
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .kill_on_drop(true) // évite les zombies si le Task iced est annulé
      .spawn()
      .map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
          AppError::BinaryNotFound(program.to_string())
        } else {
          AppError::SubprocessSpawn {
            program: program.to_string(),
            message: e.to_string(),
          }
        }
      })?;

    let output = tokio::time::timeout(
      std::time::Duration::from_secs(timeout_secs),
      child.wait_with_output(),
    )
    .await
    .map_err(|_| AppError::SubprocessTimeout {
      program: program.to_string(),
      timeout_secs,
    })?
    .map_err(|e| AppError::SubprocessIo {
      program: program.to_string(),
      message: e.to_string(),
    })?;

    Ok(ProcessOutput {
      exit_code: output.status.code().unwrap_or(-1),
      stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
      stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
  }
}

/// Implémentation de test — file de réponses préprogrammées.
#[cfg(test)]
pub mod fake {
  use std::collections::VecDeque;
  use std::sync::Mutex;

  use async_trait::async_trait;

  use super::{CommandRunner, ProcessOutput};
  use crate::error::AppError;

  pub struct FakeRunner {
    responses: Mutex<VecDeque<Result<ProcessOutput, AppError>>>,
  }

  impl FakeRunner {
    pub fn new(responses: Vec<Result<ProcessOutput, AppError>>) -> Self {
      Self {
        responses: Mutex::new(responses.into()),
      }
    }

    pub fn succeeds_with(stdout: impl Into<String>) -> Self {
      Self::new(vec![Ok(ProcessOutput {
        stdout: stdout.into(),
        stderr: String::new(),
        exit_code: 0,
      })])
    }

    pub fn fails_with(exit_code: i32, stderr: impl Into<String>) -> Self {
      Self::new(vec![Ok(ProcessOutput {
        stdout: String::new(),
        stderr: stderr.into(),
        exit_code,
      })])
    }

    pub fn binary_not_found(program: &str) -> Self {
      Self::new(vec![Err(AppError::BinaryNotFound(program.to_string()))])
    }

    /// Plusieurs réponses consommées dans l'ordre des appels
    pub fn sequence(responses: Vec<Result<ProcessOutput, AppError>>) -> Self {
      Self::new(responses)
    }
  }

  #[async_trait]
  impl CommandRunner for FakeRunner {
    async fn run(
      &self,
      _program: &str,
      _args: &[&str],
      _timeout_secs: u64,
    ) -> Result<ProcessOutput, AppError> {
      self
        .responses
        .lock()
        .unwrap()
        .pop_front()
        .expect("FakeRunner: plus de réponses préprogrammées")
    }
  }
}
