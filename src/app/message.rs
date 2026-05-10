use uuid::Uuid;

use crate::config::model::{Config, DeployMode, KeyType, ServiceType, SshKey};
use crate::error::AppError;
use crate::secrets::model::Secrets;
use crate::subprocess::gpg::GpgKeyInfo;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
  // ── Chargement initial ─────────────────────────────────────────
  ConfigLoaded(Result<Config, AppError>),
  KeysScanned(Result<Vec<SshKey>, AppError>),

  // ── GPG setup (premier lancement) ─────────────────────────────
  GpgKeysListed(Result<Vec<GpgKeyInfo>, AppError>),
  GpgKeySelected(String), // sélection UI (highlight)
  GpgSetupConfirmed,      // confirmation — déclenche la sauvegarde + transition
  GpgKeyCreated(Result<String, AppError>),
  SecretsLoaded(Result<Secrets, AppError>),
  SecretsSaved(Result<(), AppError>),

  // ── Navigation ─────────────────────────────────────────────────
  Navigate(NavItem),
  SelectService(Option<Uuid>),
  SelectKey(Option<Uuid>),

  // ── CRUD services ──────────────────────────────────────────────
  OpenCreateService,
  OpenEditService(Uuid),
  #[allow(dead_code)]
  FormFieldChanged(FormField),
  SubmitServiceForm,
  ServiceSaved(Result<Config, AppError>),
  TokenSaved(Result<(), AppError>),
  DeleteService(Uuid),
  ServiceDeleted(Result<Config, AppError>),

  // ── Association clef existante ─────────────────────────────────
  AssignKeyToService {
    service_id: Uuid,
    key_id: Uuid,
  },
  KeyAssigned(Result<Config, AppError>),

  // ── Génération de clef ─────────────────────────────────────────
  OpenGenerateKey {
    service_id: Uuid,
  },
  KeyTypeSelected(KeyType),
  PassphraseChanged(String),
  GenerateKeyConfirmed,
  KeyGenerated(Result<SshKey, AppError>),
  KeySaved(Result<Config, AppError>),
  CloseKeyGen,

  // ── Déploiement ────────────────────────────────────────────────
  OpenDeployFlow {
    service_id: Uuid,
    key_id: Uuid,
  },
  DeployModeSelected(DeployMode),
  DeployCompleted(Result<(), AppError>),
  VerifyCompleted(Result<bool, AppError>),
  DeployAndSaved(Result<Config, AppError>),
  GuidedDeployConfirmed,
  CloseDeployFlow,

  // ── Chiffrement clef existante ─────────────────────────────────
  OpenAddPassphrase {
    key_id: Uuid,
  },
  PassphraseCollectedForKey {
    key_id: Uuid,
    passphrase: String,
  },
  AddPassphraseCompleted(Result<(), AppError>),
  KeysProtectionChecked(Vec<(Uuid, crate::subprocess::ssh_keygen::ProtectionStatus)>),

  // ── Révocation de clef ─────────────────────────────────────────
  StartRevoke {
    service_id: Uuid,
    key_id: Uuid,
  },
  RevokeCompleted(Result<(), AppError>),
  RevokeSaved(Result<Config, AppError>),

  // ── UI ─────────────────────────────────────────────────────────
  #[allow(dead_code)]
  ToggleHelp(HelpId),
  FormStepNext,
  FormStepPrev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavItem {
  #[default]
  Services,
  SshKeys,
  Health,
  Settings,
}

/// Champs du formulaire service.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FormField {
  Name(String),
  ServiceType(ServiceType),
  Host(String),
  User(String),
  Port(String),
  DeployMode(DeployMode),
  Token(String),
}

/// Blocs d'aide collapse dans l'UI.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpId {
  ServiceType,
  Ed25519,
  SshCopyId,
  Gpg,
  Fingerprint,
  Rotation,
}
