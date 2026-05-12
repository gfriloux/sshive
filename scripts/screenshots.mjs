#!/usr/bin/env node
// Génère les 5 screenshots de documentation via Playwright + Vite mock.
// Usage  : node scripts/screenshots.mjs
// Prereq : npm install dans tauri-app/

import { execSync, spawn } from 'child_process';
import { mkdir } from 'fs/promises';
import { fileURLToPath } from 'url';
import path from 'path';

const ROOT     = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const APP_DIR  = path.join(ROOT, 'tauri-app');
const OUT_DIR  = path.join(ROOT, 'docs', 'src', 'assets', 'screenshots');
const PORT     = 1422;
const BASE_URL = `http://localhost:${PORT}`;
const VIEWPORT = { width: 1280, height: 800 };

// Playwright est dans tauri-app/node_modules (module CJS → export via .default)
const pw = await import(path.join(APP_DIR, 'node_modules', 'playwright', 'index.js'));
const chromium = pw.chromium ?? pw.default?.chromium;

// Sur NixOS, cherche chrome-headless-shell dans le store Nix
let CHROMIUM_EXEC;
try {
  const found = execSync(
    'find /nix/store -maxdepth 3 -name "chrome-headless-shell" -type f 2>/dev/null | head -1',
    { encoding: 'utf8' }
  ).trim();
  if (found) { execSync(`test -x "${found}"`); CHROMIUM_EXEC = found; }
} catch { /* hors NixOS — playwright cherche ses propres browsers */ }

// ── Serveur Vite mock ─────────────────────────────────────────────────────

function startVite() {
  return new Promise((resolve, reject) => {
    const proc = spawn('npx', ['vite', '--port', String(PORT), '--strictPort'], {
      cwd: APP_DIR,
      env: { ...process.env, VITE_MOCK: 'true' },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    const timeout = setTimeout(
      () => reject(new Error('Vite ne démarre pas (timeout 30s)')),
      30_000
    );
    proc.stdout.on('data', d => {
      if (d.toString().includes(String(PORT))) { clearTimeout(timeout); resolve(proc); }
    });
    proc.stderr.on('data', d => process.stderr.write(d));
    proc.on('error', reject);
  });
}

// ── Helpers ───────────────────────────────────────────────────────────────

async function ready(page) {
  await page.waitForSelector('.loading-overlay', { state: 'hidden', timeout: 10_000 })
    .catch(() => {});
  await page.waitForTimeout(500);
}

async function shot(page, name) {
  await page.screenshot({ path: path.join(OUT_DIR, name), fullPage: false });
  console.log(`  ✓  ${name}`);
}

// ── Main ──────────────────────────────────────────────────────────────────

let vite; let browser;
try {
  await mkdir(OUT_DIR, { recursive: true });

  console.log('⟳  Démarrage Vite (mock)…');
  vite = await startVite();
  await new Promise(r => setTimeout(r, 800));

  console.log('⟳  Lancement Chromium…\n');
  browser = await chromium.launch(CHROMIUM_EXEC ? { executablePath: CHROMIUM_EXEC } : {});
  const ctx  = await browser.newContext({ viewport: VIEWPORT });
  const page = await ctx.newPage();

  // 1. Services list ──────────────────────────────────────────────────────
  console.log('📸  services-list.png');
  await page.goto(BASE_URL, { waitUntil: 'networkidle' });
  await ready(page);
  await shot(page, 'services-list.png');

  // 2. Detail panel ───────────────────────────────────────────────────────
  console.log('📸  detail-panel.png');
  await page.locator('.service-row').first().click();
  await page.waitForTimeout(500);
  await shot(page, 'detail-panel.png');

  // 3. Health page ────────────────────────────────────────────────────────
  console.log('📸  health-page.png');
  await page.locator('.nav-item').filter({ hasText: /sant[eé]/i }).click();
  await page.waitForTimeout(400);
  await shot(page, 'health-page.png');

  // 4. Add service modal ──────────────────────────────────────────────────
  console.log('📸  add-service-modal.png');
  await page.locator('.nav-item').filter({ hasText: /services/i }).click();
  await page.waitForTimeout(300);
  // Cherche le bouton d'ajout (peut être "+", "Ajouter", ou aria-label)
  const addBtn = page.locator('button').filter({ hasText: /^\+$|ajouter/i }).first();
  const fallback = page.locator('[aria-label*="outer"], [title*="outer"]').first();
  if (await addBtn.count() > 0) await addBtn.click();
  else await fallback.click();
  await page.waitForSelector('.modal-box, [class*="modal"]', { timeout: 5000 })
    .catch(() => {});
  await page.waitForTimeout(400);
  await shot(page, 'add-service-modal.png');

  // 5. Deploy modal ───────────────────────────────────────────────────────
  console.log('📸  deploy-modal.png');
  await page.keyboard.press('Escape');
  await page.waitForTimeout(300);
  // Sélectionne le 4e service (GitHub CI/CD — mode guidé)
  const rows = page.locator('.service-row');
  const n = await rows.count();
  await rows.nth(Math.min(3, n - 1)).click();
  await page.waitForTimeout(400);
  await page.locator('button').filter({ hasText: /d[eé]ployer/i }).first().click();
  await page.waitForSelector('.modal-box, [class*="modal"]', { timeout: 5000 })
    .catch(() => {});
  await page.waitForTimeout(500);
  await shot(page, 'deploy-modal.png');

  console.log(`\n✅  5 screenshots → ${OUT_DIR}`);

} catch (err) {
  console.error('\n❌', err.message);
  process.exitCode = 1;
} finally {
  await browser?.close();
  vite?.kill();
}
