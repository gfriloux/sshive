<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  import TopBar from './lib/TopBar.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import Services from './pages/Services.svelte';
  import Keys from './pages/Keys.svelte';
  import Health from './pages/Health.svelte';
  import Settings from './pages/Settings.svelte';
  import GpgSetupModal from './components/GpgSetupModal.svelte';
  import AddServiceModal from './components/AddServiceModal.svelte';
  import KeyGenModal from './components/KeyGenModal.svelte';
  import DeployModal from './components/DeployModal.svelte';

  // ---- State ----
  let page              = $state('services');
  let services          = $state([]);
  let keys              = $state([]);
  let selectedServiceId = $state(null);
  let selectedKeyId     = $state(null);
  let appSettings       = $state({ rotation_warning_days: 90, gpg_fingerprint: null, min_passphrase_len: 12 });
  let loading           = $state(true);
  let error             = $state(null);
  let gpgSetupNeeded    = $state(false);
  let gpgKeys           = $state([]);
  let healthCounts      = $state({ ok: 0, warning: 0, critical: 0 });

  // Modal states
  let showAddService    = $state(false);
  let showKeyGen        = $state(false);
  let deployTarget      = $state(null); // { service, key }

  // ---- Derived ----
  let selectedService   = $derived(services.find(s => s.id === selectedServiceId) ?? null);

  // ---- Load ----
  async function loadApp() {
    loading = true;
    error = null;
    try {
      const state = await invoke('load_app');
      services     = state.services ?? [];
      keys         = state.keys ?? [];
      healthCounts = state.health_counts ?? { ok: 0, warning: 0, critical: 0 };
      appSettings  = state.settings ?? appSettings;
      if (!state.gpg_configured) {
        gpgSetupNeeded = true;
        gpgKeys = await invoke('list_gpg_keys').catch(() => []);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadApp();
  });

  // ---- Handlers ----
  function handleNavigate(p) {
    page = p;
    selectedServiceId = null;
    selectedKeyId = null;
  }

  function handleSelectService(id) {
    selectedServiceId = id === selectedServiceId ? null : id;
  }

  function handleAddService() {
    showAddService = true;
  }

  async function handleServiceCreated(detail) {
    showAddService = false;
    await loadApp();
    if (detail?.id) selectedServiceId = detail.id;
  }

  async function handleDeleteService(id) {
    try {
      await invoke('delete_service', { id });
      selectedServiceId = null;
      await loadApp();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleDeploy(detail) {
    deployTarget = detail;
  }

  async function handleRevoke(detail) {
    try {
      await invoke('revoke_key', { serviceId: detail.serviceId, keyId: detail.keyId });
      await loadApp();
    } catch (e) {
      error = String(e);
    }
  }

  function handleKeyGen() {
    showKeyGen = true;
  }

  async function handleKeyGenerated(detail) {
    showKeyGen = false;
    await loadApp();
  }

  async function handleGpgSetup(fingerprint) {
    if (!fingerprint) {
      // User skipped GPG setup
      gpgSetupNeeded = false;
      return;
    }
    try {
      await invoke('setup_gpg', { fingerprint });
      gpgSetupNeeded = false;
      await loadApp();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleSettingsSave(newSettings) {
    try {
      await invoke('update_settings', {
        rotationWarningDays: newSettings.rotation_warning_days,
        minPassphraseLen: newSettings.min_passphrase_len,
      });
      appSettings = newSettings;
      await loadApp();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="app-shell">

  <!-- Loading overlay -->
  {#if loading}
    <div class="loading-overlay">
      <div class="loading-inner">
        <span class="spinner lg"></span>
        <p>Chargement de SSHive…</p>
      </div>
    </div>
  {/if}

  <!-- Error banner -->
  {#if error}
    <div class="error-banner">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      <span>{error}</span>
      <button class="btn-ghost btn btn-sm" onclick={() => error = null}>Fermer</button>
    </div>
  {/if}

  <!-- Top bar -->
  <TopBar {page} {healthCounts} />

  <!-- Body -->
  <div class="app-body">
    <Sidebar {page} onNavigate={handleNavigate} criticalCount={healthCounts.critical} />

    <main class="main-content">
      {#if page === 'services'}
        <Services
          {services}
          selectedId={selectedServiceId}
          onSelect={handleSelectService}
          onAdd={handleAddService}
          onDelete={handleDeleteService}
          onDeploy={handleDeploy}
          onRevoke={handleRevoke}
          {keys}
        />
      {:else if page === 'keys'}
        <Keys {keys} {services} onGenerate={handleKeyGen} />
      {:else if page === 'health'}
        <Health {services} counts={healthCounts} />
      {:else if page === 'settings'}
        <Settings
          settings={appSettings}
          {gpgKeys}
          onSave={handleSettingsSave}
        />
      {/if}
    </main>
  </div>

  <!-- Modals -->
  {#if gpgSetupNeeded}
    <GpgSetupModal {gpgKeys} onConfirm={handleGpgSetup} />
  {/if}

  {#if showAddService}
    <AddServiceModal
      onConfirm={handleServiceCreated}
      onClose={() => showAddService = false}
    />
  {/if}

  {#if showKeyGen}
    <KeyGenModal
      services={services}
      onConfirm={handleKeyGenerated}
      onClose={() => showKeyGen = false}
    />
  {/if}

  {#if deployTarget}
    <DeployModal
      service={deployTarget.service}
      keyView={deployTarget.key}
      onClose={() => { deployTarget = null; loadApp(); }}
    />
  {/if}

</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    position: relative;
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Loading overlay */
  .loading-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: var(--bg-base);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fade-in 200ms ease;
  }
  .loading-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  /* Error banner */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--critical-glow);
    border-bottom: 1px solid rgba(201, 64, 64, 0.3);
    color: var(--critical);
    font-size: 0.8rem;
    z-index: 50;
  }
  .error-banner span {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
