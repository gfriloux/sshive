pub mod message;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::{Element, Subscription, Task};
use uuid::Uuid;

use crate::app::message::{FormField, HelpId, Message, NavItem};
use crate::config::model::{
  Config, DeployMode, KeyType, Service, ServiceParams, ServiceType, SshKey,
};
use crate::error::AppError;
use crate::secrets::model::Secrets;
use crate::subprocess::gpg::GpgKeyInfo;
use crate::subprocess::ssh_keygen::ProtectionStatus;

#[derive(Debug, Default)]
pub struct App {
  pub state: AppState,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default)]
pub enum AppState {
  #[default]
  Loading,
  /// Premier lancement : aucune clef GPG configurée
  GpgSetup(GpgSetupState),
  Ready(ReadyState),
  Error(AppError),
}

#[derive(Debug, Default)]
pub struct GpgSetupState {
  pub config: Config, // Config chargée au démarrage
  pub available_keys: Vec<GpgKeyInfo>,
  pub selected_fingerprint: Option<String>,
  pub loading: bool,
}

#[derive(Debug, Default)]
pub struct ReadyState {
  pub config: Config,
  pub local_keys: Vec<SshKey>,
  pub secrets: Secrets,
  pub active_nav: NavItem,
  pub selected_service: Option<Uuid>,
  pub selected_key: Option<Uuid>,
  pub open_helps: HashSet<HelpId>,
  pub form: Option<ServiceFormState>,
  pub key_gen: Option<KeyGenState>,
  pub deploy: Option<DeployState>,
  pub key_protection: HashMap<Uuid, ProtectionStatus>,
  pub add_passphrase: Option<AddPassphraseState>,
}

/// Étapes du flux de déploiement.
#[derive(Debug, Clone)]
pub enum DeployStep {
  ChooseMode,
  AutoDeploying,
  GuidedCommand { command: String },
  Success,
  Error(String),
}

/// État du flux de déploiement SSH.
#[derive(Debug, Clone)]
pub struct DeployState {
  pub service_id: Uuid,
  pub key_id: Uuid,
  pub step: DeployStep,
}

/// Étapes du wizard de génération de clef.
#[derive(Debug, Clone, Default)]
pub enum KeyGenStep {
  #[default]
  ChooseType,
  EnterPassphrase,
  Generating,
  Success {
    fingerprint: String,
  },
}

/// État du wizard de génération de clef SSH.
#[derive(Debug, Clone)]
pub struct KeyGenState {
  pub service_id: Uuid,
  pub step: KeyGenStep,
  pub key_type: Option<KeyType>,
  pub passphrase: String,
  pub error: Option<String>,
}

impl KeyGenState {
  pub fn new(service_id: Uuid) -> Self {
    Self {
      service_id,
      step: KeyGenStep::ChooseType,
      key_type: None,
      passphrase: String::new(),
      error: None,
    }
  }
}

/// Statut du wizard d'ajout de passphrase sur une clef existante.
#[derive(Debug, Clone)]
pub enum AddPassphraseStatus {
  CollectingPassphrase,
  ApplyingPassphrase,
  Error(String),
}

#[derive(Debug, Clone)]
pub struct AddPassphraseState {
  pub key_id: Uuid,
  pub status: AddPassphraseStatus,
}

/// État du formulaire de création/édition de service.
#[derive(Debug, Clone, Default)]
pub struct ServiceFormState {
  pub step: usize,
  pub name: String,
  pub service_type: Option<ServiceType>,
  pub host: String,
  pub user: String,
  pub port: String,
  pub deploy_mode: DeployMode,
  pub editing_id: Option<Uuid>,
  pub error: Option<String>,
}

impl ServiceFormState {
  pub fn new() -> Self {
    Self {
      step: 1,
      port: "22".to_string(),
      deploy_mode: DeployMode::Automatic,
      ..Default::default()
    }
  }

  pub fn from_service(service: &Service) -> Self {
    Self {
      step: 1,
      name: service.name.clone(),
      service_type: Some(service.service_type.clone()),
      host: service.params.url.clone().unwrap_or_default(),
      user: service.params.user.clone().unwrap_or_default(),
      port: service
        .params
        .port
        .map(|p| p.to_string())
        .unwrap_or_else(|| "22".to_string()),
      deploy_mode: service.deploy_mode.clone(),
      editing_id: Some(service.id),
      error: None,
    }
  }

  pub fn apply(&mut self, field: FormField) {
    self.error = None;
    match field {
      FormField::Name(v) => self.name = v,
      FormField::ServiceType(v) => self.service_type = Some(v),
      FormField::Host(v) => self.host = v,
      FormField::User(v) => self.user = v,
      FormField::Port(v) => self.port = v,
      FormField::DeployMode(v) => self.deploy_mode = v,
    }
  }

  pub fn step1_valid(&self) -> bool {
    !self.name.trim().is_empty() && self.service_type.is_some()
  }

  pub fn needs_connection_params(&self) -> bool {
    matches!(
      self.service_type,
      Some(ServiceType::SshGeneric)
        | Some(ServiceType::GitLabSelfHosted)
        | Some(ServiceType::Manual)
    )
  }

  pub fn build(&self) -> Result<Service, String> {
    let name = self.name.trim().to_string();
    if name.is_empty() {
      return Err("Le nom est requis.".to_string());
    }
    let service_type = self
      .service_type
      .clone()
      .ok_or_else(|| "Le type de service est requis.".to_string())?;

    if self.needs_connection_params() && !self.host.is_empty() {
      crate::config::validation::validate_hostname(&self.host).map_err(|e| e.to_string())?;
    }
    if !self.user.is_empty() {
      crate::config::validation::validate_username(&self.user).map_err(|e| e.to_string())?;
    }

    let port: Option<u16> = if self.port.trim().is_empty() || self.port == "22" {
      None
    } else {
      Some(crate::config::validation::validate_port(&self.port).map_err(|e| e.to_string())?)
    };

    Ok(Service {
      id: self.editing_id.unwrap_or_else(Uuid::new_v4),
      name,
      service_type,
      params: ServiceParams {
        url: if self.host.is_empty() {
          None
        } else {
          Some(self.host.clone())
        },
        user: if self.user.is_empty() {
          None
        } else {
          Some(self.user.clone())
        },
        port,
        token_ref: None,
      },
      active_key: None,
      pending_key: None,
      created_at: chrono::Local::now().date_naive(),
      last_rotation: None,
      deploy_mode: self.deploy_mode.clone(),
    })
  }
}

/// Infère le chemin de la clef privée depuis une SshKey.
/// Pour les clefs générées par SSHive, private_path est déjà défini.
/// Pour les clefs scannées, on retire l'extension .pub du chemin public.
fn infer_private_path(key: &SshKey) -> Option<PathBuf> {
  if let Some(ref p) = key.private_path {
    return Some(p.clone());
  }
  if let Some(ref pub_path) = key.public_path {
    let private = pub_path.with_extension("");
    if private.exists() {
      return Some(private);
    }
  }
  None
}

/// Détecte la protection pour une liste de clefs (par lot, séquentiel).
async fn check_keys_protection(keys: Vec<(Uuid, PathBuf)>) -> Vec<(Uuid, ProtectionStatus)> {
  let mut results = Vec::new();
  for (id, path) in keys {
    match crate::subprocess::ssh_keygen::detect_protection(&path).await {
      Ok(status) => results.push((id, status)),
      Err(e) => tracing::warn!("Détection protection {} : {e}", path.display()),
    }
  }
  results
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
      // ── Chargement initial ──────────────────────────────────────
      Message::ConfigLoaded(Ok(config)) => {
        // Si aucune clef GPG n'est configurée → GpgSetup
        if config.gpg.key_fingerprint.is_none() {
          self.state = AppState::GpgSetup(GpgSetupState {
            config, // on conserve la config pour la sauvegarder après
            loading: true,
            ..Default::default()
          });
          return Task::perform(
            crate::subprocess::gpg::list_secret_keys(),
            Message::GpgKeysListed,
          );
        }
        // Sinon → charger les secrets puis scanner les clefs
        // key_fingerprint est Some : vérifié par le is_none() ci-dessus
        let fp = config
          .gpg
          .key_fingerprint
          .clone()
          .expect("key_fingerprint présent — vérifié par is_none()");
        let secrets_path = config.gpg.secrets_path.clone().unwrap_or_else(|| {
          crate::secrets::secrets_path_default().expect("répertoire config accessible")
        });

        self.state = AppState::Ready(ReadyState {
          config,
          ..ReadyState::default()
        });
        Task::perform(
          crate::secrets::load_or_create(fp, secrets_path),
          Message::SecretsLoaded,
        )
      }

      Message::ConfigLoaded(Err(e)) => {
        self.state = AppState::Error(e);
        Task::none()
      }

      Message::KeysScanned(Ok(keys)) => {
        if let AppState::Ready(ref mut s) = self.state {
          let fp_to_uuid: HashMap<&str, Uuid> = s
            .config
            .keys
            .iter()
            .map(|k| (k.fingerprint.as_str(), k.id))
            .collect();
          s.local_keys = keys
            .into_iter()
            .map(|mut k| {
              if let Some(&stable_id) = fp_to_uuid.get(k.fingerprint.as_str()) {
                k.id = stable_id;
              }
              k
            })
            .collect();

          // Détecter la protection pour les clefs non-hardware dont on peut inférer la privée
          let to_check: Vec<(Uuid, PathBuf)> = s
            .local_keys
            .iter()
            .filter_map(|k| infer_private_path(k).map(|p| (k.id, p)))
            .collect();

          if !to_check.is_empty() {
            return Task::perform(
              check_keys_protection(to_check),
              Message::KeysProtectionChecked,
            );
          }
        }
        Task::none()
      }

      Message::KeysScanned(Err(e)) => {
        tracing::warn!("Scan des clefs SSH échoué : {e}");
        Task::none()
      }

      // ── GPG setup ───────────────────────────────────────────────
      Message::GpgKeysListed(Ok(keys)) => {
        if let AppState::GpgSetup(ref mut s) = self.state {
          s.available_keys = keys;
          s.loading = false;
        }
        Task::none()
      }

      Message::GpgKeysListed(Err(e)) => {
        tracing::warn!("Liste des clefs GPG échouée : {e}");
        if let AppState::GpgSetup(ref mut s) = self.state {
          s.available_keys.clear();
          s.loading = false;
        }
        Task::none()
      }

      // Clic sur une carte dans le setup GPG — met à jour le highlight uniquement
      Message::GpgKeySelected(fp) => {
        if let AppState::GpgSetup(ref mut s) = self.state {
          s.selected_fingerprint = Some(fp);
        }
        Task::none()
      }

      // Clic sur "Utiliser cette clef" — applique le choix et démarre le flux
      Message::GpgSetupConfirmed => {
        let (mut config, fp) = match &self.state {
          AppState::GpgSetup(s) => {
            let fp = match s.selected_fingerprint.clone() {
              Some(fp) => fp,
              None => return Task::none(), // bouton désactivé — ne devrait pas arriver
            };
            (s.config.clone(), fp)
          }
          _ => return Task::none(),
        };

        // Écrire le fingerprint dans la config
        config.gpg.key_fingerprint = Some(fp.clone());

        // Basculer en Ready immédiatement (optimiste) puis sauvegarder
        let secrets_path = config
          .gpg
          .secrets_path
          .clone()
          .unwrap_or_else(|| crate::secrets::secrets_path_default().unwrap());

        self.state = AppState::Ready(ReadyState {
          config: config.clone(),
          ..ReadyState::default()
        });

        // Sauvegarder la config et charger les secrets en parallèle
        Task::batch([
          Task::perform(
            crate::config::writer::save_config_owned(config),
            |r| match r {
              Ok(_) => Message::SecretsSaved(Ok(())),
              Err(e) => Message::SecretsSaved(Err(e)),
            },
          ),
          Task::perform(
            crate::secrets::load_or_create(fp, secrets_path),
            Message::SecretsLoaded,
          ),
          Task::perform(
            crate::config::ssh_scanner::scan_pub_keys(),
            Message::KeysScanned,
          ),
        ])
      }

      Message::GpgKeyCreated(Ok(fp)) => self.update(Message::GpgKeySelected(fp)),

      Message::GpgKeyCreated(Err(e)) => {
        tracing::warn!("Création clef GPG échouée : {e}");
        Task::none()
      }

      Message::SecretsLoaded(Ok(secrets)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.secrets = secrets;
        }
        Task::perform(
          crate::config::ssh_scanner::scan_pub_keys(),
          Message::KeysScanned,
        )
      }

      Message::SecretsLoaded(Err(e)) => {
        tracing::warn!("Chargement des secrets échoué : {e}");
        // Mode dégradé : on continue sans les secrets
        Task::perform(
          crate::config::ssh_scanner::scan_pub_keys(),
          Message::KeysScanned,
        )
      }

      Message::SecretsSaved(Err(e)) => {
        tracing::warn!("Sauvegarde des secrets échouée : {e}");
        Task::none()
      }

      Message::SecretsSaved(Ok(())) => Task::none(),

      // ── Navigation ──────────────────────────────────────────────
      Message::Navigate(nav) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.active_nav = nav;
          s.selected_service = None;
          s.selected_key = None;
        }
        Task::none()
      }

      Message::SelectService(id) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.selected_service = id;
          s.selected_key = None;
        }
        Task::none()
      }

      Message::SelectKey(id) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.selected_key = id;
          s.selected_service = None;
        }
        Task::none()
      }

      // ── CRUD services ───────────────────────────────────────────
      Message::OpenCreateService => {
        if let AppState::Ready(ref mut s) = self.state {
          s.form = Some(ServiceFormState::new());
          s.selected_service = None;
          s.selected_key = None;
        }
        Task::none()
      }

      Message::OpenEditService(id) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(svc) = s.config.services.iter().find(|svc| svc.id == id) {
            s.form = Some(ServiceFormState::from_service(svc));
          }
        }
        Task::none()
      }

      Message::FormFieldChanged(field) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut form) = s.form {
            form.apply(field);
          }
        }
        Task::none()
      }

      Message::FormStepNext => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut form) = s.form {
            if form.step == 1 && !form.step1_valid() {
              form.error = Some("Remplissez le nom et choisissez un type.".to_string());
            } else if form.step < 3 {
              form.error = None;
              form.step += 1;
            }
          }
        }
        Task::none()
      }

      Message::FormStepPrev => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut form) = s.form {
            if form.step > 1 {
              form.error = None;
              form.step -= 1;
            }
          }
        }
        Task::none()
      }

      Message::SubmitServiceForm => {
        if let AppState::Ready(ref s) = self.state {
          if let Some(ref form) = s.form {
            match form.build() {
              Ok(service) => {
                let mut config = s.config.clone();
                let editing_id = form.editing_id;
                if let Some(id) = editing_id {
                  if let Some(pos) = config.services.iter().position(|svc| svc.id == id) {
                    config.services[pos] = service;
                  }
                } else {
                  config.services.push(service);
                }
                return Task::perform(
                  crate::config::writer::save_config_owned(config),
                  Message::ServiceSaved,
                );
              }
              Err(msg) => {
                if let AppState::Ready(ref mut s) = self.state {
                  if let Some(ref mut form) = s.form {
                    form.error = Some(msg);
                  }
                }
              }
            }
          }
        }
        Task::none()
      }

      Message::ServiceSaved(Ok(config)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.config = config;
          s.form = None;
        }
        Task::none()
      }

      Message::ServiceSaved(Err(e)) => {
        tracing::warn!("Sauvegarde service échouée : {e}");
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut form) = s.form {
            form.error = Some(format!("Erreur de sauvegarde : {e}"));
          }
        }
        Task::none()
      }

      Message::DeleteService(id) => {
        if let AppState::Ready(ref s) = self.state {
          let mut config = s.config.clone();
          config.services.retain(|svc| svc.id != id);
          return Task::perform(
            crate::config::writer::save_config_owned(config),
            Message::ServiceDeleted,
          );
        }
        Task::none()
      }

      Message::ServiceDeleted(Ok(config)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.config = config;
          s.selected_service = None;
          s.form = None;
        }
        Task::none()
      }

      Message::ServiceDeleted(Err(e)) => {
        tracing::warn!("Suppression service échouée : {e}");
        Task::none()
      }

      // ── Association clef existante ─────────────────────────────
      Message::AssignKeyToService { service_id, key_id } => {
        if let AppState::Ready(ref s) = self.state {
          let mut config = s.config.clone();
          // Promouvoir la clef de local_keys → config.keys pour que son UUID soit stable
          if let Some(key) = s.local_keys.iter().find(|k| k.id == key_id) {
            if !config.keys.iter().any(|k| k.fingerprint == key.fingerprint) {
              config.keys.push(key.clone());
            }
          }
          if let Some(svc) = config.services.iter_mut().find(|s| s.id == service_id) {
            svc.active_key = Some(key_id);
          }
          return Task::perform(
            crate::config::writer::save_config_owned(config),
            Message::KeyAssigned,
          );
        }
        Task::none()
      }

      Message::KeyAssigned(Ok(config)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.config = config;
        }
        Task::none()
      }

      Message::KeyAssigned(Err(e)) => {
        tracing::warn!("Association clef échouée : {e}");
        Task::none()
      }

      // ── Génération clef ─────────────────────────────────────────
      Message::OpenGenerateKey { service_id } => {
        if let AppState::Ready(ref mut s) = self.state {
          s.key_gen = Some(KeyGenState::new(service_id));
          s.form = None;
        }
        Task::none()
      }

      Message::CloseKeyGen => {
        if let AppState::Ready(ref mut s) = self.state {
          s.key_gen = None;
        }
        Task::none()
      }

      Message::KeyTypeSelected(kt) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut kg) = s.key_gen {
            kg.key_type = Some(kt);
            kg.error = None;
            kg.step = KeyGenStep::EnterPassphrase;
          }
        }
        Task::none()
      }

      Message::PassphraseChanged(v) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut kg) = s.key_gen {
            kg.passphrase = v;
          }
        }
        Task::none()
      }

      Message::GenerateKeyConfirmed => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut kg) = s.key_gen {
            if kg.passphrase.len() < 12 {
              kg.error = Some("La phrase secrète doit faire au moins 12 caractères.".to_string());
              return Task::none();
            }
            let service_id = kg.service_id;
            let passphrase = kg.passphrase.clone();
            let use_yubikey = matches!(kg.key_type, Some(KeyType::SkEd25519));
            let service_name = s
              .config
              .services
              .iter()
              .find(|sv| sv.id == service_id)
              .map(|sv| sv.name.clone())
              .unwrap_or_default();
            let ssh_dir = dirs::home_dir().map(|h| h.join(".ssh")).unwrap_or_default();
            kg.step = KeyGenStep::Generating;
            kg.error = None;
            return Task::perform(
              crate::subprocess::ssh_keygen::generate_key(
                crate::subprocess::ssh_keygen::KeyGenRequest {
                  service_name,
                  service_id,
                  ssh_dir,
                  use_yubikey,
                  passphrase: Some(passphrase),
                },
              ),
              Message::KeyGenerated,
            );
          }
        }
        Task::none()
      }

      Message::KeyGenerated(Ok(key)) => {
        if let AppState::Ready(ref mut s) = self.state {
          let fingerprint = key.fingerprint.clone();
          let key_id = key.id;
          // Lier la clef au service
          let service_id = s.key_gen.as_ref().map(|kg| kg.service_id);
          s.local_keys.push(key.clone());
          s.config.keys.push(key);
          if let Some(sid) = service_id {
            if let Some(svc) = s.config.services.iter_mut().find(|sv| sv.id == sid) {
              svc.active_key = Some(key_id);
            }
          }
          if let Some(ref mut kg) = s.key_gen {
            kg.step = KeyGenStep::Success { fingerprint };
          }
          let config = s.config.clone();
          return Task::perform(
            crate::config::writer::save_config_owned(config),
            Message::KeySaved,
          );
        }
        Task::none()
      }

      Message::KeyGenerated(Err(e)) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut kg) = s.key_gen {
            kg.step = KeyGenStep::EnterPassphrase;
            kg.error = Some(e.to_string());
          }
        }
        Task::none()
      }

      Message::KeySaved(Ok(config)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.config = config;
        }
        Task::none()
      }

      Message::KeySaved(Err(e)) => {
        tracing::warn!("Sauvegarde après génération clef échouée : {e}");
        Task::none()
      }

      // ── Déploiement ─────────────────────────────────────────────
      Message::OpenDeployFlow { service_id, key_id } => {
        if let AppState::Ready(ref mut s) = self.state {
          s.deploy = Some(DeployState {
            service_id,
            key_id,
            step: DeployStep::ChooseMode,
          });
          s.key_gen = None;
          s.form = None;
        }
        Task::none()
      }

      Message::CloseDeployFlow => {
        if let AppState::Ready(ref mut s) = self.state {
          s.deploy = None;
        }
        Task::none()
      }

      Message::DeployModeSelected(mode) => {
        if let AppState::Ready(ref mut s) = self.state {
          let Some(ref deploy) = s.deploy else {
            return Task::none();
          };
          let service_id = deploy.service_id;
          let key_id = deploy.key_id;

          let service = s
            .config
            .services
            .iter()
            .find(|sv| sv.id == service_id)
            .cloned();
          let key = s
            .local_keys
            .iter()
            .chain(s.config.keys.iter())
            .find(|k| k.id == key_id)
            .cloned();

          let (Some(service), Some(key)) = (service, key) else {
            if let Some(ref mut d) = s.deploy {
              d.step = DeployStep::Error("Service ou clef introuvable.".to_string());
            }
            return Task::none();
          };

          let host = service.params.url.clone().unwrap_or_default();
          let user = service.params.user.clone().unwrap_or_default();
          let port = service.params.port.unwrap_or(22);

          if host.is_empty() || user.is_empty() {
            if let Some(ref mut d) = s.deploy {
              d.step = DeployStep::Error(
                "Ce service n'a pas de paramètres de connexion configurés.".to_string(),
              );
            }
            return Task::none();
          }

          match mode {
            DeployMode::Automatic => {
              let Some(pub_path) = key.public_path.clone() else {
                if let Some(ref mut d) = s.deploy {
                  d.step = DeployStep::Error("Chemin de clef publique introuvable.".to_string());
                }
                return Task::none();
              };
              if let Some(ref mut d) = s.deploy {
                d.step = DeployStep::AutoDeploying;
              }
              return Task::perform(
                async move {
                  crate::subprocess::ssh_copy_id::deploy_key(&pub_path, &user, &host, port).await
                },
                Message::DeployCompleted,
              );
            }
            DeployMode::Guided => {
              let pub_path = key.public_path.clone().unwrap_or_default();
              let command = crate::subprocess::ssh_copy_id::guided_deploy_command(
                &pub_path, &user, &host, port,
              );
              if let Some(ref mut d) = s.deploy {
                d.step = DeployStep::GuidedCommand { command };
              }
            }
          }
        }
        Task::none()
      }

      Message::DeployCompleted(Ok(())) => {
        if let AppState::Ready(ref mut s) = self.state {
          // Mise à jour last_rotation
          let service_id = s.deploy.as_ref().map(|d| d.service_id);
          if let Some(sid) = service_id {
            if let Some(svc) = s.config.services.iter_mut().find(|sv| sv.id == sid) {
              svc.last_rotation = Some(chrono::Local::now().date_naive());
            }
          }
          if let Some(ref mut d) = s.deploy {
            d.step = DeployStep::Success;
          }
          let config = s.config.clone();
          return Task::perform(
            crate::config::writer::save_config_owned(config),
            Message::DeployAndSaved,
          );
        }
        Task::none()
      }

      Message::DeployCompleted(Err(e)) => {
        tracing::warn!("Déploiement SSH échoué : {e}");
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut d) = s.deploy {
            d.step = DeployStep::Error(e.to_string());
          }
        }
        Task::none()
      }

      Message::GuidedDeployConfirmed => {
        if let AppState::Ready(ref mut s) = self.state {
          let service_id = s.deploy.as_ref().map(|d| d.service_id);
          if let Some(sid) = service_id {
            if let Some(svc) = s.config.services.iter_mut().find(|sv| sv.id == sid) {
              svc.last_rotation = Some(chrono::Local::now().date_naive());
            }
          }
          if let Some(ref mut d) = s.deploy {
            d.step = DeployStep::Success;
          }
          let config = s.config.clone();
          return Task::perform(
            crate::config::writer::save_config_owned(config),
            Message::DeployAndSaved,
          );
        }
        Task::none()
      }

      Message::DeployAndSaved(Ok(config)) => {
        if let AppState::Ready(ref mut s) = self.state {
          s.config = config;
        }
        Task::none()
      }

      Message::DeployAndSaved(Err(e)) => {
        tracing::warn!("Sauvegarde post-déploiement échouée : {e}");
        Task::none()
      }

      // ── Chiffrement clef existante ─────────────────────────────
      Message::KeysProtectionChecked(results) => {
        if let AppState::Ready(ref mut s) = self.state {
          for (id, status) in results {
            s.key_protection.insert(id, status);
          }
        }
        Task::none()
      }

      Message::OpenAddPassphrase { key_id } => {
        if let AppState::Ready(ref mut s) = self.state {
          s.add_passphrase = Some(AddPassphraseState {
            key_id,
            status: AddPassphraseStatus::CollectingPassphrase,
          });
          return Task::perform(
            async move {
              let pass = crate::subprocess::pinentry::collect_passphrase(
                "SSHive",
                "Chiffrez votre clef SSH\n(12 caractères minimum)",
              )
              .await?;
              if pass.len() < 12 {
                return Err(crate::error::AppError::Validation {
                  field: "passphrase".into(),
                  reason: "La phrase secrète doit faire au moins 12 caractères.".into(),
                });
              }
              Ok(pass.as_str().to_string())
            },
            move |result| match result {
              Ok(passphrase) => Message::PassphraseCollectedForKey { key_id, passphrase },
              Err(e) => Message::AddPassphraseCompleted(Err(e)),
            },
          );
        }
        Task::none()
      }

      Message::PassphraseCollectedForKey { key_id, passphrase } => {
        if let AppState::Ready(ref mut s) = self.state {
          let private_path = s
            .local_keys
            .iter()
            .find(|k| k.id == key_id)
            .and_then(infer_private_path);

          if let Some(path) = private_path {
            if let Some(ref mut aps) = s.add_passphrase {
              aps.status = AddPassphraseStatus::ApplyingPassphrase;
            }
            return Task::perform(
              async move {
                let pass = crate::subprocess::pinentry::Passphrase::new(passphrase);
                crate::subprocess::ssh_keygen::add_passphrase(&path, &pass).await
              },
              Message::AddPassphraseCompleted,
            );
          } else {
            tracing::warn!("add_passphrase : clef privée introuvable pour id={key_id}");
            if let Some(ref mut aps) = s.add_passphrase {
              aps.status =
                AddPassphraseStatus::Error("Clef privée introuvable sur le disque.".into());
            }
          }
        }
        Task::none()
      }

      Message::AddPassphraseCompleted(Ok(())) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref aps) = s.add_passphrase {
            s.key_protection.remove(&aps.key_id);
          }
          s.add_passphrase = None;
        }
        Task::perform(
          crate::config::ssh_scanner::scan_pub_keys(),
          Message::KeysScanned,
        )
      }

      Message::AddPassphraseCompleted(Err(e)) => {
        if let AppState::Ready(ref mut s) = self.state {
          if let Some(ref mut aps) = s.add_passphrase {
            aps.status = AddPassphraseStatus::Error(e.to_string());
          }
        }
        Task::none()
      }

      // ── UI ──────────────────────────────────────────────────────
      Message::ToggleHelp(id) => {
        if let AppState::Ready(ref mut s) = self.state {
          if s.open_helps.contains(&id) {
            s.open_helps.remove(&id);
          } else {
            s.open_helps.insert(id);
          }
        }
        Task::none()
      }
    }
  }

  pub fn view(&self) -> Element<'_, Message> {
    match &self.state {
      AppState::Loading => crate::ui::view_loading(),
      AppState::GpgSetup(s) => crate::ui::view_gpg_setup(s),
      AppState::Ready(s) => crate::ui::view_ready(s),
      AppState::Error(e) => crate::ui::view_error(e),
    }
  }

  pub fn subscription(&self) -> Subscription<Message> {
    Subscription::none()
  }
}
