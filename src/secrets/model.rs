#![allow(dead_code)]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Contenu de secrets.yaml.gpg — chiffré au repos via GPG.
/// Note : la protection principale est le chiffrement GPG du fichier.
/// La zéroïsation en mémoire (HashMap ne supporte pas Zeroize) est
/// une amélioration future via un type dédié.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Secrets {
  /// Tokens API indexés par `token_ref` (ex. "sshive/github_perso")
  #[serde(default)]
  pub tokens: HashMap<String, String>,
}
