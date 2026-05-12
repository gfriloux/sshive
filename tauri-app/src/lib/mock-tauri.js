// Remplace @tauri-apps/api/core en mode VITE_MOCK=true
import { MOCK_APP_STATE, MOCK_GPG_KEYS, MOCK_KEYS } from './mock-data.js';

export async function invoke(cmd, _args = {}) {
  await new Promise(r => setTimeout(r, 60)); // simule la latence IPC

  switch (cmd) {
    case 'load_app':
      return structuredClone(MOCK_APP_STATE);

    case 'list_gpg_keys':
      return MOCK_GPG_KEYS;

    case 'list_token_refs':
      return ['sshive/github/gfriloux'];

    case 'get_public_key': {
      const key = MOCK_KEYS.find(k => k.id === _args.keyId);
      return key?.public_key ?? 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMockPublicKeyForScreenshots mock@sshive';
    }

    case 'create_service':
    case 'update_service':
    case 'delete_service':
    case 'generate_key':
    case 'deploy_key':
    case 'revoke_key':
    case 'add_passphrase':
    case 'update_settings':
    case 'setup_gpg':
    case 'set_token':
    case 'delete_token':
      return null;

    default:
      console.warn('[mock] commande inconnue :', cmd);
      return null;
  }
}
