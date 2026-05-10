/// Normalise un nom de service en clef de référence pour secrets.tokens.
/// Exposé pour l'UI (service_form.rs).
pub fn sanitize_token_ref_pub(name: &str) -> String {
  name
    .chars()
    .map(|c| {
      if c.is_alphanumeric() {
        c.to_ascii_lowercase()
      } else {
        '_'
      }
    })
    .collect::<String>()
    .trim_matches('_')
    .to_string()
}
