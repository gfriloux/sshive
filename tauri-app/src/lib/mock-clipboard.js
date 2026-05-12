// Remplace @tauri-apps/plugin-clipboard-manager en mode VITE_MOCK=true
export async function writeText(_text) {
  // no-op en mode mock
}
export async function readText() {
  return '';
}
