#![deny(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sshive::config::model::{
  Config, Deployment, DeployMode, Service, ServiceParams, ServiceType,
};
use sshive::health::{HealthLevel, HealthSnapshot};
use sshive::subprocess::ssh_keygen::{KeyGenRequest, ProtectionStatus};
use uuid::Uuid;

// ── Vue JSON des types de domaine ─────────────────────────────────────────

/// Représentation JSON d'un Service enrichie des données de santé et de la
/// clef publique en clair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceView {
  pub id: String,
  pub name: String,
  pub service_type: String,
  pub params: ServiceParams,
  pub active_key: Option<String>,
  pub pending_key: Option<String>,
  pub created_at: String,
  pub last_rotation: Option<String>,
  pub deploy_mode: String,
  pub deployments: Vec<Deployment>,
  // Santé calculée
  pub health_level: String,
  pub health_reasons: Vec<String>,
  pub rotation_age_days: Option<i64>,
  pub public_key: Option<String>,
}

/// Représentation JSON d'une clef SSH avec protection et service liés.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyView {
  pub id: String,
  pub fingerprint: String,
  pub key_type: String,
  pub yubikey: bool,
  pub created_at: String,
  pub comment: String,
  pub private_path: Option<String>,
  pub public_path: Option<String>,
  pub service_id: Option<String>,
  pub backup_prompted: bool,
  // Données calculées
  pub protection: String,
  pub linked_service_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
  pub services: Vec<ServiceView>,
  pub keys: Vec<KeyView>,
  pub health_counts: HealthCounts,
  pub gpg_configured: bool,
  pub settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCounts {
  pub ok: usize,
  pub warning: usize,
  pub critical: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
  pub rotation_warning_days: u32,
  pub gpg_fingerprint: Option<String>,
  pub min_passphrase_len: u8,
}

/// Vue JSON d'une clef GPG (GpgKeyInfo n'implémente pas Serialize).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpgKeyView {
  pub fingerprint: String,
  pub uid: String,
  pub expires: Option<String>,
}

// ── Helpers de conversion ─────────────────────────────────────────────────

fn service_type_to_str(st: &ServiceType) -> &'static str {
  match st {
    ServiceType::GitHub => "github",
    ServiceType::GitLab => "gitlab",
    ServiceType::GitLabSelfHosted => "gitlab-self-hosted",
    ServiceType::SshGeneric => "ssh-generic",
  }
}

fn deploy_mode_to_str(dm: &DeployMode) -> &'static str {
  match dm {
    DeployMode::Automatic => "automatic",
    DeployMode::Guided => "guided",
    DeployMode::ExternalCm => "external-cm",
  }
}

fn parse_service_type(s: &str) -> Result<ServiceType, String> {
  match s {
    "github" => Ok(ServiceType::GitHub),
    "gitlab" => Ok(ServiceType::GitLab),
    "gitlab-self-hosted" => Ok(ServiceType::GitLabSelfHosted),
    "ssh-generic" | "manual" => Ok(ServiceType::SshGeneric),
    other => Err(format!("type de service inconnu : {other}")),
  }
}

fn parse_deploy_mode(s: &str) -> Result<DeployMode, String> {
  match s {
    "automatic" => Ok(DeployMode::Automatic),
    "guided" => Ok(DeployMode::Guided),
    "external-cm" => Ok(DeployMode::ExternalCm),
    other => Err(format!("mode de déploiement inconnu : {other}")),
  }
}

fn health_level_to_str(level: &HealthLevel) -> &'static str {
  match level {
    HealthLevel::Ok => "ok",
    HealthLevel::Info => "info",
    HealthLevel::Warning => "warning",
    HealthLevel::Critical => "critical",
  }
}

fn health_reason_to_str(reason: &sshive::health::HealthReason) -> String {
  use sshive::health::HealthReason;
  match reason {
    HealthReason::NoKey => "Aucune clef SSH active".to_string(),
    HealthReason::KeyUnprotected { key_id } => {
      format!("Clef {key_id} non protégée par passphrase")
    }
    HealthReason::RotationOverdue { days_overdue } => {
      format!("Rotation en retard de {days_overdue} jour(s)")
    }
    HealthReason::PendingDeployment => "Déploiement en attente".to_string(),
    HealthReason::NoApiToken => "Token API absent ou non configuré".to_string(),
    HealthReason::HardwareKeyHandleNotBackedUp { key_id } => {
      format!("Handle de la clef hardware {key_id} non sauvegardé")
    }
  }
}

fn protection_to_str(ps: &ProtectionStatus) -> &'static str {
  match ps {
    ProtectionStatus::Protected => "protected",
    ProtectionStatus::Unprotected => "unprotected",
    ProtectionStatus::HardwareKey => "hardware-key",
  }
}

// ── Validateurs IPC ───────────────────────────────────────────────────────

const MAX_NAME_LEN: usize = 256;
const MAX_TOKEN_REF_LEN: usize = 128;
const MAX_URL_LEN: usize = 512;
const MAX_USER_LEN: usize = 64;
const MAX_TOKEN_VALUE_LEN: usize = 4096;
const MAX_PUBKEY_BYTES: u64 = 16_384;

fn validate_token_ref(token_ref: &str) -> Result<(), String> {
  if token_ref.is_empty() || token_ref.len() > MAX_TOKEN_REF_LEN {
    return Err(format!("token_ref doit avoir entre 1 et {MAX_TOKEN_REF_LEN} caractères"));
  }
  if !token_ref
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-')
  {
    return Err("token_ref ne peut contenir que [A-Za-z0-9/_-]".into());
  }
  if token_ref.starts_with('/') || token_ref.contains("//") || token_ref.contains("..") {
    return Err("token_ref invalide (chemin interdit)".into());
  }
  Ok(())
}

fn validate_name(name: &str) -> Result<(), String> {
  if name.is_empty() || name.len() > MAX_NAME_LEN {
    return Err(format!("nom doit avoir entre 1 et {MAX_NAME_LEN} caractères"));
  }
  if name.chars().any(|c| c.is_control()) {
    return Err("nom contient des caractères de contrôle".into());
  }
  Ok(())
}

fn validate_url(url: &Option<String>) -> Result<(), String> {
  if let Some(u) = url {
    if u.len() > MAX_URL_LEN {
      return Err(format!("url doit faire ≤ {MAX_URL_LEN} caractères"));
    }
    if u.chars().any(|c| c.is_control()) {
      return Err("url contient des caractères de contrôle".into());
    }
  }
  Ok(())
}

fn validate_user(user: &Option<String>) -> Result<(), String> {
  if let Some(u) = user {
    if u.is_empty() || u.len() > MAX_USER_LEN {
      return Err(format!("user doit avoir entre 1 et {MAX_USER_LEN} caractères"));
    }
    if !u
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
      return Err("user ne peut contenir que [A-Za-z0-9._-]".into());
    }
  }
  Ok(())
}

fn validate_token_value(token: &str) -> Result<(), String> {
  if token.is_empty() || token.len() > MAX_TOKEN_VALUE_LEN {
    return Err(format!("token doit avoir entre 1 et {MAX_TOKEN_VALUE_LEN} caractères"));
  }
  if token.chars().any(|c| c.is_control() && c != '\n') {
    return Err("token contient des caractères de contrôle".into());
  }
  Ok(())
}

fn validate_gpg_fingerprint(fp: &str) -> Result<(), String> {
  let stripped: String = fp.chars().filter(|c| !c.is_whitespace()).collect();
  if stripped.len() != 40 || !stripped.chars().all(|c| c.is_ascii_hexdigit()) {
    return Err("Fingerprint GPG invalide : 40 caractères hexadécimaux requis".into());
  }
  Ok(())
}

fn read_file_bounded(path: &std::path::Path) -> Option<String> {
  let meta = std::fs::metadata(path).ok()?;
  if meta.len() > MAX_PUBKEY_BYTES {
    return None;
  }
  std::fs::read_to_string(path).ok()
}

// ── Construction des vues ─────────────────────────────────────────────────

fn build_service_view(
  service: &Service,
  snapshot: &HealthSnapshot,
  all_keys: &[sshive::config::model::SshKey],
) -> ServiceView {
  let health = snapshot.services.get(&service.id);
  let health_level = health
    .map(|h| health_level_to_str(&h.level))
    .unwrap_or("ok")
    .to_string();
  let health_reasons: Vec<String> = health
    .map(|h| h.reasons.iter().map(health_reason_to_str).collect())
    .unwrap_or_default();
  let rotation_age_days = health.and_then(|h| h.rotation_age_days);

  // Lire le fichier .pub de la clef active si disponible
  let public_key = service
    .active_key
    .and_then(|kid| all_keys.iter().find(|k| k.id == kid))
    .and_then(|k| k.public_path.as_ref())
    .and_then(|p| read_file_bounded(p));

  ServiceView {
    id: service.id.to_string(),
    name: service.name.clone(),
    service_type: service_type_to_str(&service.service_type).to_string(),
    params: service.params.clone(),
    active_key: service.active_key.map(|u| u.to_string()),
    pending_key: service.pending_key.map(|u| u.to_string()),
    created_at: service.created_at.to_string(),
    last_rotation: service.last_rotation.map(|d| d.to_string()),
    deploy_mode: deploy_mode_to_str(&service.deploy_mode).to_string(),
    deployments: service.deployments.clone(),
    health_level,
    health_reasons,
    rotation_age_days,
    public_key,
  }
}

fn build_key_view(
  key: &sshive::config::model::SshKey,
  protection: &HashMap<Uuid, ProtectionStatus>,
  services: &[Service],
) -> KeyView {
  let prot = protection
    .get(&key.id)
    .map(protection_to_str)
    .unwrap_or("unknown")
    .to_string();

  let linked_service_name = key
    .service_id
    .and_then(|sid| services.iter().find(|s| s.id == sid))
    .map(|s| s.name.clone());

  KeyView {
    id: key.id.to_string(),
    fingerprint: key.fingerprint.clone(),
    key_type: key.key_type.to_string(),
    yubikey: key.yubikey,
    created_at: key.created_at.to_string(),
    comment: key.comment.clone(),
    private_path: key.private_path.as_ref().and_then(|p| p.to_str().map(|s| s.to_string())),
    public_path: key.public_path.as_ref().and_then(|p| p.to_str().map(|s| s.to_string())),
    service_id: key.service_id.map(|u| u.to_string()),
    backup_prompted: key.backup_prompted,
    protection: prot,
    linked_service_name,
  }
}

/// Calcule la protection de chaque clef managée (avec chemin privé).
async fn compute_protection(
  keys: &[sshive::config::model::SshKey],
) -> HashMap<Uuid, ProtectionStatus> {
  let mut map = HashMap::new();
  for key in keys {
    if let Some(path) = &key.private_path {
      if let Ok(status) = sshive::subprocess::ssh_keygen::detect_protection(path).await {
        map.insert(key.id, status);
      }
    }
  }
  map
}

/// Construit l'AppState complet depuis la config et le snapshot de santé.
async fn build_app_state(config: Config, snapshot: HealthSnapshot, local_keys: Vec<sshive::config::model::SshKey>) -> AppState {
  let protection = compute_protection(&local_keys).await;

  let services: Vec<ServiceView> = config
    .services
    .iter()
    .map(|s| build_service_view(s, &snapshot, &local_keys))
    .collect();

  let keys: Vec<KeyView> = local_keys
    .iter()
    .map(|k| build_key_view(k, &protection, &config.services))
    .collect();

  let (critical, warning, ok) = snapshot.counts();

  AppState {
    services,
    keys,
    health_counts: HealthCounts {
      ok,
      warning,
      critical,
    },
    gpg_configured: config.gpg.key_fingerprint.is_some(),
    settings: AppSettings {
      rotation_warning_days: config.health.rotation_warning_days,
      gpg_fingerprint: config.gpg.key_fingerprint.clone(),
      min_passphrase_len: config.security.min_passphrase_len,
    },
  }
}

// ── Commandes Tauri ───────────────────────────────────────────────────────

/// Charge les secrets GPG si GPG est configuré, sinon retourne des secrets vides.
async fn load_secrets(config: &sshive::config::model::Config) -> sshive::secrets::model::Secrets {
  let Some(ref fp) = config.gpg.key_fingerprint else {
    return sshive::secrets::model::Secrets::default();
  };
  let path = config.gpg.secrets_path.clone().unwrap_or_else(|| {
    sshive::secrets::secrets_path_default().unwrap_or_default()
  });
  sshive::secrets::load_or_create_ref(fp, &path)
    .await
    .unwrap_or_default()
}

/// Charge la configuration, scanne ~/.ssh/*.pub, calcule la santé et retourne l'état complet.
/// Si la configuration n'existe pas, retourne un AppState vide (pas d'erreur fatale).
#[tauri::command]
async fn load_app() -> Result<AppState, String> {
  let config = sshive::config::loader::load_or_create()
    .await
    .unwrap_or_default();

  // Fusionner les clefs gérées (config.yaml) avec les clefs découvertes (~/.ssh/*.pub).
  // Les clefs déjà présentes dans config (par fingerprint) ne sont pas dupliquées.
  let managed_fps: std::collections::HashSet<String> =
    config.keys.iter().map(|k| k.fingerprint.clone()).collect();

  let scanned = sshive::config::ssh_scanner::scan_pub_keys()
    .await
    .unwrap_or_default();

  let mut all_keys = config.keys.clone();
  for k in scanned {
    if !managed_fps.contains(&k.fingerprint) {
      all_keys.push(k);
    }
  }

  let protection = compute_protection(&all_keys).await;
  let secrets = load_secrets(&config).await;
  let snapshot = HealthSnapshot::compute(&config, &all_keys, &protection, Some(&secrets));

  Ok(build_app_state(config, snapshot, all_keys).await)
}

/// Crée un nouveau service et le persiste dans la configuration.
#[tauri::command]
async fn create_service(
  name: String,
  service_type: String,
  url: Option<String>,
  user: Option<String>,
  port: Option<u16>,
  token_ref: Option<String>,
  deploy_mode: String,
) -> Result<ServiceView, String> {
  validate_name(&name)?;
  validate_url(&url)?;
  validate_user(&user)?;
  if let Some(ref t) = token_ref {
    validate_token_ref(t)?;
  }
  let svc_type = parse_service_type(&service_type)?;
  let dm = parse_deploy_mode(&deploy_mode)?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let service = Service {
    id: Uuid::new_v4(),
    name,
    service_type: svc_type,
    params: ServiceParams {
      url,
      user,
      port,
      token_ref,
    },
    active_key: None,
    pending_key: None,
    created_at: chrono::Local::now().date_naive(),
    last_rotation: None,
    deploy_mode: dm,
    deployments: vec![],
  };

  config.services.push(service.clone());
  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  let protection = compute_protection(&config.keys).await;
  let snapshot = HealthSnapshot::compute(&config, &config.keys, &protection, None);

  Ok(build_service_view(&service, &snapshot, &config.keys))
}

/// Met à jour les paramètres d'un service existant.
#[tauri::command]
async fn update_service(
  id: String,
  name: String,
  service_type: String,
  url: Option<String>,
  user: Option<String>,
  port: Option<u16>,
  token_ref: Option<String>,
  deploy_mode: String,
) -> Result<ServiceView, String> {
  validate_name(&name)?;
  validate_url(&url)?;
  validate_user(&user)?;
  if let Some(ref t) = token_ref {
    validate_token_ref(t)?;
  }
  let target_id: Uuid = id.parse().map_err(|_| format!("UUID invalide : {id}"))?;
  let svc_type = parse_service_type(&service_type)?;
  let dm = parse_deploy_mode(&deploy_mode)?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let service = config
    .services
    .iter_mut()
    .find(|s| s.id == target_id)
    .ok_or_else(|| format!("Service {id} introuvable"))?;

  service.name = name;
  service.service_type = svc_type;
  service.params = ServiceParams {
    url,
    user,
    port,
    token_ref,
  };
  service.deploy_mode = dm;

  let updated = service.clone();

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  let protection = compute_protection(&config.keys).await;
  let snapshot = HealthSnapshot::compute(&config, &config.keys, &protection, None);

  Ok(build_service_view(&updated, &snapshot, &config.keys))
}

/// Supprime un service par son identifiant.
#[tauri::command]
async fn delete_service(id: String) -> Result<(), String> {
  let target_id: Uuid = id.parse().map_err(|_| format!("UUID invalide : {id}"))?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  // Capture le nom avant suppression pour l'audit
  let service_name = config
    .services
    .iter()
    .find(|s| s.id == target_id)
    .map(|s| s.name.clone())
    .ok_or_else(|| format!("Service {id} introuvable"))?;

  config.services.retain(|s| s.id != target_id);

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  sshive::audit::append(sshive::audit::AuditEvent::ServiceDeleted {
    service: &service_name,
  });

  Ok(())
}

/// Attache une clef SSH existante à un service (active_key + service_id).
#[tauri::command]
async fn assign_key(service_id: String, key_id: String) -> Result<(), String> {
  let svc_id: Uuid = service_id.parse().map_err(|_| format!("UUID invalide : {service_id}"))?;
  let kid: Uuid = key_id.parse().map_err(|_| format!("UUID invalide : {key_id}"))?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  // Vérifie que le service existe
  config
    .services
    .iter()
    .find(|s| s.id == svc_id)
    .ok_or_else(|| format!("Service {service_id} introuvable"))?;

  // Vérifie que la clef existe dans config.keys (clef gérée uniquement)
  config
    .keys
    .iter()
    .find(|k| k.id == kid)
    .ok_or_else(|| format!("Clef {key_id} introuvable dans la configuration (les clefs scannées ne peuvent pas être attachées directement)"))?;

  // Mise à jour du service
  if let Some(svc) = config.services.iter_mut().find(|s| s.id == svc_id) {
    svc.active_key = Some(kid);
  }
  // Mise à jour de la clef
  if let Some(key) = config.keys.iter_mut().find(|k| k.id == kid) {
    key.service_id = Some(svc_id);
  }

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  Ok(())
}

/// Génère une nouvelle paire de clefs SSH et l'associe au service.
#[tauri::command]
async fn generate_key(
  service_id: String,
  key_type: String,
  passphrase: String,
) -> Result<KeyView, String> {
  let target_id: Uuid = service_id
    .parse()
    .map_err(|_| format!("UUID invalide : {service_id}"))?;

  let use_yubikey = key_type == "sk-ed25519";
  if key_type != "ed25519" && key_type != "sk-ed25519" {
    return Err(format!("type de clef inconnu : {key_type}"));
  }

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let (service_name, req) = {
    let service = config
      .services
      .iter()
      .find(|s| s.id == target_id)
      .ok_or_else(|| format!("Service {service_id} introuvable"))?;

    let ssh_dir = dirs::home_dir()
      .ok_or_else(|| "Impossible de trouver le répertoire home".to_string())?
      .join(".ssh");

    let name = service.name.clone();
    let req = KeyGenRequest {
      service_name: name.clone(),
      service_id: target_id,
      ssh_dir,
      use_yubikey,
      passphrase: if passphrase.is_empty() { None } else { Some(passphrase) },
    };
    (name, req)
  }; // borrow immutable libéré ici

  let new_key = sshive::subprocess::ssh_keygen::generate_key(req)
    .await
    .map_err(|e| e.to_string())?;

  // Mettre à jour la config : clef pending + enregistrement
  let service_mut = config
    .services
    .iter_mut()
    .find(|s| s.id == target_id)
    .ok_or_else(|| format!("Service {service_id} introuvable"))?;

  service_mut.pending_key = Some(new_key.id);
  config.keys.push(new_key.clone());

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  sshive::audit::append(sshive::audit::AuditEvent::KeyGenerated {
    service: &service_name,
    fingerprint: &new_key.fingerprint,
    key_type: &new_key.key_type.to_string(),
  });

  let protection = compute_protection(&config.keys).await;
  let view = build_key_view(&new_key, &protection, &config.services);

  Ok(view)
}

/// Déploie une clef sur un service via le deployer adapté.
/// Retourne un message de confirmation.
#[tauri::command]
async fn deploy_key(service_id: String, key_id: String) -> Result<String, String> {
  let svc_id: Uuid = service_id
    .parse()
    .map_err(|_| format!("UUID invalide : {service_id}"))?;
  let kid: Uuid = key_id
    .parse()
    .map_err(|_| format!("UUID invalide : {key_id}"))?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let service = config
    .services
    .iter()
    .find(|s| s.id == svc_id)
    .ok_or_else(|| format!("Service {service_id} introuvable"))?
    .clone();

  let key = config
    .keys
    .iter()
    .find(|k| k.id == kid)
    .ok_or_else(|| format!("Clef {key_id} introuvable"))?
    .clone();

  let secrets = load_secrets(&config).await;
  let deployer = sshive::deployer::deployer_for(&service, &secrets);
  let token = secrets.token_for(&service);
  let ctx = sshive::deployer::DeployContext::build(&service, &key, token)
    .unwrap_or_else(|_| sshive::deployer::DeployContext::build_api(&service, &key, secrets.token_for(&service)));

  let remote_ref = deployer
    .deploy(&ctx)
    .await
    .map_err(|e| e.to_string())?;

  // Mettre à jour la config : déplacer pending → active, enregistrer le déploiement
  {
    let svc_mut = config
      .services
      .iter_mut()
      .find(|s| s.id == svc_id)
      .ok_or_else(|| format!("Service {service_id} introuvable"))?;

    svc_mut.active_key = Some(kid);
    svc_mut.pending_key = None;
    svc_mut.last_rotation = Some(chrono::Local::now().date_naive());
    svc_mut.deployments.push(sshive::config::model::Deployment {
      key_id: kid,
      deployed_at: chrono::Local::now().date_naive(),
      remote_ref: remote_ref.clone(),
      last_verified: Some(chrono::Local::now().date_naive()),
    });
  }

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  let msg = remote_ref
    .map(|r| format!("Clef déployée avec succès (ref: {r})"))
    .unwrap_or_else(|| "Clef déployée avec succès".to_string());

  sshive::audit::append(sshive::audit::AuditEvent::KeyDeployed {
    service: &service.name,
    fingerprint: &key.fingerprint,
    result: "ok",
  });

  Ok(msg)
}

/// Révoque une clef d'un service.
#[tauri::command]
async fn revoke_key(service_id: String, key_id: String) -> Result<(), String> {
  let svc_id: Uuid = service_id
    .parse()
    .map_err(|_| format!("UUID invalide : {service_id}"))?;
  let kid: Uuid = key_id
    .parse()
    .map_err(|_| format!("UUID invalide : {key_id}"))?;

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let service = config
    .services
    .iter()
    .find(|s| s.id == svc_id)
    .ok_or_else(|| format!("Service {service_id} introuvable"))?
    .clone();

  let key = config
    .keys
    .iter()
    .find(|k| k.id == kid)
    .ok_or_else(|| format!("Clef {key_id} introuvable"))?
    .clone();

  // Récupère la remote_ref depuis le dernier déploiement de cette clef
  let remote_ref: Option<String> = service
    .deployments
    .iter()
    .filter(|d| d.key_id == kid)
    .last()
    .and_then(|d| d.remote_ref.clone());

  let secrets = load_secrets(&config).await;
  let deployer = sshive::deployer::deployer_for(&service, &secrets);
  let token = secrets.token_for(&service);
  let ctx = sshive::deployer::DeployContext::build(&service, &key, token)
    .unwrap_or_else(|_| sshive::deployer::DeployContext::build_api(&service, &key, secrets.token_for(&service)));

  deployer
    .revoke(&ctx, remote_ref.as_deref())
    .await
    .map_err(|e| e.to_string())?;

  // Mettre à jour la config : retirer la clef active/pending si c'est celle-là
  {
    let svc_mut = config
      .services
      .iter_mut()
      .find(|s| s.id == svc_id)
      .ok_or_else(|| format!("Service {service_id} introuvable"))?;

    if svc_mut.active_key == Some(kid) {
      svc_mut.active_key = None;
    }
    if svc_mut.pending_key == Some(kid) {
      svc_mut.pending_key = None;
    }
  }

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  sshive::audit::append(sshive::audit::AuditEvent::KeyRevoked {
    service: &service.name,
    fingerprint: &key.fingerprint,
    result: "ok",
  });

  Ok(())
}

/// Ajoute une passphrase à une clef non protégée.
#[tauri::command]
async fn add_passphrase(key_id: String, passphrase: String) -> Result<(), String> {
  let kid: Uuid = key_id
    .parse()
    .map_err(|_| format!("UUID invalide : {key_id}"))?;

  let config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let key = config
    .keys
    .iter()
    .find(|k| k.id == kid)
    .ok_or_else(|| format!("Clef {key_id} introuvable"))?;

  let private_path = key
    .private_path
    .as_ref()
    .ok_or_else(|| format!("Clef {key_id} n'a pas de chemin de clef privée"))?;

  let pass = sshive::subprocess::pinentry::Passphrase::new(passphrase);
  sshive::subprocess::ssh_keygen::add_passphrase(private_path, &pass)
    .await
    .map_err(|e| e.to_string())?;

  Ok(())
}

/// Met à jour les paramètres de santé et sécurité dans la configuration.
#[tauri::command]
async fn update_settings(
  rotation_warning_days: u32,
  min_passphrase_len: u8,
) -> Result<(), String> {
  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  config.health.rotation_warning_days = rotation_warning_days;
  config.security.min_passphrase_len = min_passphrase_len;

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  Ok(())
}

/// Liste les clefs GPG disponibles dans le keyring.
#[tauri::command]
async fn list_gpg_keys() -> Result<Vec<GpgKeyView>, String> {
  let keys = sshive::subprocess::gpg::list_secret_keys()
    .await
    .map_err(|e| e.to_string())?;

  Ok(
    keys
      .into_iter()
      .map(|k| GpgKeyView {
        fingerprint: k.fingerprint,
        uid: k.uid,
        expires: k.expires.map(|d| d.to_string()),
      })
      .collect(),
  )
}

/// Configure la clef GPG utilisée pour chiffrer secrets.yaml.gpg.
#[tauri::command]
async fn setup_gpg(fingerprint: String) -> Result<(), String> {
  validate_gpg_fingerprint(&fingerprint)?;
  let normalized = fingerprint
    .chars()
    .filter(|c| !c.is_whitespace())
    .map(|c| c.to_ascii_uppercase())
    .collect::<String>();

  let mut config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  config.gpg.key_fingerprint = Some(normalized);

  sshive::config::writer::save_config(&config)
    .await
    .map_err(|e| e.to_string())?;

  Ok(())
}

/// Retourne le contenu du fichier .pub d'une clef SSH.
#[tauri::command]
async fn get_public_key(key_id: String) -> Result<String, String> {
  let kid: Uuid = key_id
    .parse()
    .map_err(|_| format!("UUID invalide : {key_id}"))?;

  let config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let key = config
    .keys
    .iter()
    .find(|k| k.id == kid)
    .ok_or_else(|| format!("Clef {key_id} introuvable"))?;

  let public_path = key
    .public_path
    .as_ref()
    .ok_or_else(|| format!("Clef {key_id} n'a pas de chemin de clef publique"))?;

  read_file_bounded(public_path)
    .ok_or_else(|| format!("Impossible de lire {} (fichier absent ou > 16 KiB)", public_path.display()))
}

/// Liste les token_ref configurés dans secrets.yaml.gpg (sans exposer les valeurs).
#[tauri::command]
async fn list_token_refs() -> Result<Vec<String>, String> {
  let config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let secrets = load_secrets(&config).await;

  let mut refs: Vec<String> = secrets.tokens.keys().cloned().collect();
  refs.sort();

  Ok(refs)
}

/// Ajoute ou met à jour un token dans secrets.yaml.gpg.
#[tauri::command]
async fn set_token(token_ref: String, token_value: String) -> Result<(), String> {
  validate_token_ref(&token_ref)?;
  validate_token_value(&token_value)?;
  let config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let fp = config
    .gpg
    .key_fingerprint
    .as_ref()
    .ok_or_else(|| "GPG non configuré".to_string())?
    .clone();

  let path = config
    .gpg
    .secrets_path
    .clone()
    .unwrap_or_else(|| sshive::secrets::secrets_path_default().unwrap_or_default());

  let mut secrets = sshive::secrets::load_or_create_ref(&fp, &path)
    .await
    .map_err(|e| e.to_string())?;

  secrets.tokens.insert(token_ref, token_value);

  sshive::secrets::save_ref(&secrets, &fp, &path)
    .await
    .map_err(|e| e.to_string())
}

/// Supprime un token de secrets.yaml.gpg.
#[tauri::command]
async fn delete_token(token_ref: String) -> Result<(), String> {
  validate_token_ref(&token_ref)?;
  let config = sshive::config::loader::load_or_create()
    .await
    .map_err(|e| e.to_string())?;

  let fp = config
    .gpg
    .key_fingerprint
    .as_ref()
    .ok_or_else(|| "GPG non configuré".to_string())?
    .clone();

  let path = config
    .gpg
    .secrets_path
    .clone()
    .unwrap_or_else(|| sshive::secrets::secrets_path_default().unwrap_or_default());

  let mut secrets = sshive::secrets::load_or_create_ref(&fp, &path)
    .await
    .map_err(|e| e.to_string())?;

  secrets.tokens.remove(&token_ref);

  sshive::secrets::save_ref(&secrets, &fp, &path)
    .await
    .map_err(|e| e.to_string())
}

// ── Point d'entrée Tauri ─────────────────────────────────────────────────

pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    .invoke_handler(tauri::generate_handler![
      load_app,
      create_service,
      update_service,
      delete_service,
      assign_key,
      generate_key,
      deploy_key,
      revoke_key,
      add_passphrase,
      update_settings,
      list_gpg_keys,
      setup_gpg,
      get_public_key,
      list_token_refs,
      set_token,
      delete_token,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  // validate_token_ref
  #[test] fn token_ref_valide() { assert!(validate_token_ref("sshive/github/gfriloux").is_ok()); }
  #[test] fn token_ref_vide_rejete() { assert!(validate_token_ref("").is_err()); }
  #[test] fn token_ref_trop_long_rejete() { assert!(validate_token_ref(&"a".repeat(129)).is_err()); }
  #[test] fn token_ref_slash_debut_rejete() { assert!(validate_token_ref("/bad").is_err()); }
  #[test] fn token_ref_double_slash_rejete() { assert!(validate_token_ref("a//b").is_err()); }
  #[test] fn token_ref_dotdot_rejete() { assert!(validate_token_ref("a/../b").is_err()); }
  #[test] fn token_ref_char_invalide_rejete() { assert!(validate_token_ref("a b").is_err()); }

  // validate_name
  #[test] fn name_valide() { assert!(validate_name("GitHub gfriloux").is_ok()); }
  #[test] fn name_vide_rejete() { assert!(validate_name("").is_err()); }
  #[test] fn name_trop_long_rejete() { assert!(validate_name(&"a".repeat(257)).is_err()); }
  #[test] fn name_ctrl_rejete() { assert!(validate_name("bad\x00name").is_err()); }

  // validate_gpg_fingerprint
  #[test] fn fingerprint_40hex_valide() {
    assert!(validate_gpg_fingerprint("ABCDEF1234567890ABCDEF1234567890ABCDEF12").is_ok());
  }
  #[test] fn fingerprint_lowercase_valide() {
    assert!(validate_gpg_fingerprint("abcdef1234567890abcdef1234567890abcdef12").is_ok());
  }
  #[test] fn fingerprint_avec_espaces_valide() {
    assert!(validate_gpg_fingerprint("ABCD EF12 3456 7890 ABCD  EF12 3456 7890 ABCD EF12").is_ok());
  }
  #[test] fn fingerprint_trop_court_rejete() {
    assert!(validate_gpg_fingerprint("ABCDEF1234567890").is_err());
  }
  #[test] fn fingerprint_non_hex_rejete() {
    assert!(validate_gpg_fingerprint("ABCDEF1234567890ABCDEF1234567890ABCDEFZZ").is_err());
  }

  // validate_token_value
  #[test] fn token_value_valide() { assert!(validate_token_value("ghp_abc123XYZ").is_ok()); }
  #[test] fn token_value_vide_rejete() { assert!(validate_token_value("").is_err()); }
  #[test] fn token_value_ctrl_rejete() { assert!(validate_token_value("abc\x01def").is_err()); }

  // read_file_bounded
  #[test] fn file_bounded_absent_retourne_none() {
    assert!(read_file_bounded(std::path::Path::new("/dev/null/inexistant")).is_none());
  }
}
