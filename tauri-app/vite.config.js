import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

const host = process.env.TAURI_DEV_HOST;
const isMock = process.env.VITE_MOCK === 'true';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: isMock ? {
      '@tauri-apps/api/core': path.resolve('./src/lib/mock-tauri.js'),
      '@tauri-apps/plugin-clipboard-manager': path.resolve('./src/lib/mock-clipboard.js'),
    } : {},
  },
  server: {
    port: 1421,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
  },
  // Empêche Vite de scanner les fichiers générés dans src-tauri/target/
  optimizeDeps: {
    entries: ['index.html'],
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome105', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
