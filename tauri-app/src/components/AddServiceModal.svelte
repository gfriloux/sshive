<script>
  import { invoke } from '@tauri-apps/api/core';
  import Modal from './Modal.svelte';

  const { onConfirm, onClose } = $props();

  // ---- Stepper state ----
  let step = $state(1); // 1: type, 2: params, 3: deploy + confirm

  // ---- Form values ----
  let serviceType  = $state('github');
  let name         = $state('');
  let url          = $state('');
  let user         = $state('');
  let port         = $state('22');
  let tokenValue   = $state('');
  let deployMode   = $state('automatic');

  let saving       = $state(false);
  let saveError    = $state(null);

  // ---- Auto-generate token ref from name ----
  let tokenRef = $derived(
    name.trim()
      ? `sshive/${name.trim().toLowerCase().replace(/\s+/g, '_')}`
      : 'sshive/service'
  );

  // ---- Derived: step validity ----
  let step1Valid = $derived(serviceType !== '');
  let step2Valid = $derived(
    name.trim() !== '' && (
      serviceType === 'github' ||
      serviceType === 'gitlab' ||
      (serviceType === 'gitlab-self-hosted' && url.trim() !== '') ||
      (serviceType === 'ssh-generic' && url.trim() !== '' && user.trim() !== '')
    )
  );

  // ---- Type options ----
  const serviceTypes = [
    {
      id: 'github',
      label: 'GitHub',
      desc: 'github.com',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.58 2 12.26c0 4.54 2.87 8.39 6.84 9.75.5.09.68-.22.68-.49v-1.7c-2.78.62-3.37-1.37-3.37-1.37-.45-1.18-1.11-1.49-1.11-1.49-.91-.64.07-.63.07-.63 1 .07 1.53 1.06 1.53 1.06.89 1.57 2.34 1.12 2.91.85.09-.66.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.28 2.75 1.05A9.3 9.3 0 0 1 12 7.4c.85 0 1.7.12 2.5.34 1.91-1.33 2.75-1.05 2.75-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.81-4.57 5.06.36.32.68.94.68 1.9v2.82c0 .27.18.59.69.49C19.13 20.65 22 16.8 22 12.26 22 6.58 17.52 2 12 2z"/></svg>`,
    },
    {
      id: 'gitlab',
      label: 'GitLab',
      desc: 'gitlab.com',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z"/></svg>`,
    },
    {
      id: 'gitlab-self-hosted',
      label: 'GitLab SH',
      desc: 'Self-hosted',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor"><path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z"/></svg>`,
    },
    {
      id: 'ssh-generic',
      label: 'SSH',
      desc: 'Générique',
      icon: `<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.8 19.8 0 0 1-8.63-3.07A19.5 19.5 0 0 1 4.69 12 19.8 19.8 0 0 1 1.62 3.37 2 2 0 0 1 3.6 1h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L7.91 8.6a16 16 0 0 0 6 6l.96-.96a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 21.73 16l.27.92z"/></svg>`,
    },
  ];

  const deployModes = [
    { id: 'automatic',   label: 'Automatique', desc: 'SSHive déploie la clef automatiquement via API/token.' },
    { id: 'guided',      label: 'Guidé',        desc: 'SSHive génère la commande ssh-copy-id à exécuter.' },
    { id: 'external-cm', label: 'CM externe',   desc: 'La clef est gérée par NixOS/Ansible/Puppet.' },
  ];

  async function handleCreate() {
    saving = true;
    saveError = null;
    try {
      const hasToken = (serviceType === 'github' || serviceType === 'gitlab' || serviceType === 'gitlab-self-hosted');
      const ref = hasToken ? tokenRef : null;
      const result = await invoke('create_service', {
        name: name.trim(),
        serviceType,
        url: url.trim() || null,
        user: user.trim() || null,
        port: port ? parseInt(port, 10) : null,
        tokenRef: ref,
        deployMode,
      });
      if (hasToken && tokenValue.trim()) {
        await invoke('set_token', { tokenRef: ref, tokenValue: tokenValue.trim() });
      }
      onConfirm(result);
    } catch (e) {
      saveError = String(e);
      saving = false;
    }
  }

  // ---- Summary values ----
  function deployModeLabel(id) {
    return deployModes.find(m => m.id === id)?.label ?? id;
  }
  function typeLabel(id) {
    return serviceTypes.find(t => t.id === id)?.label ?? id;
  }
</script>

<Modal {onClose} maxWidth="520px" title={null}>
  {#snippet children()}
  <div class="stepper-header">
    <h3 class="modal-title-inner">Ajouter un service</h3>
    <div class="steps">
      {#each [1, 2, 3] as s}
        <div class="step" class:active={step === s} class:done={step > s}>
          {#if step > s}
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
          {:else}
            {s}
          {/if}
        </div>
        {#if s < 3}
          <div class="step-line" class:done={step > s}></div>
        {/if}
      {/each}
    </div>
  </div>

  <!-- Step 1: Type selection -->
  {#if step === 1}
    <div class="step-content">
      <p class="step-subtitle">Choisissez le type de service</p>
      <div class="type-grid">
        {#each serviceTypes as t}
          <button
            class="type-btn"
            class:selected={serviceType === t.id}
            onclick={() => serviceType = t.id}
          >
            <span class="type-icon">{@html t.icon}</span>
            <span class="type-label">{t.label}</span>
            <span class="type-desc">{t.desc}</span>
          </button>
        {/each}
      </div>
    </div>

  <!-- Step 2: Parameters -->
  {:else if step === 2}
    <div class="step-content">
      <p class="step-subtitle">Paramètres du service</p>
      <div class="form-group">
        <label class="form-label" for="svc-name">Nom du service *</label>
        <input id="svc-name" class="input" type="text" placeholder="Ex: github-perso" bind:value={name} />
      </div>

      {#if serviceType === 'gitlab-self-hosted'}
        <div class="form-group">
          <label class="form-label" for="svc-url">URL de l'instance *</label>
          <input id="svc-url" class="input" type="text" placeholder="https://gitlab.monentreprise.com" bind:value={url} />
        </div>
      {/if}

      {#if serviceType === 'ssh-generic'}
        <div class="form-group">
          <label class="form-label" for="svc-host">Hostname / adresse IP *</label>
          <input id="svc-host" class="input input-mono" type="text" placeholder="mon-serveur.example.com" bind:value={url} />
        </div>
        <div class="form-group">
          <label class="form-label" for="svc-user">Utilisateur SSH *</label>
          <input id="svc-user" class="input" type="text" placeholder="deploy" bind:value={user} />
        </div>
        <div class="form-group">
          <label class="form-label" for="svc-port">Port SSH</label>
          <input id="svc-port" class="input" type="number" min="1" max="65535" placeholder="22" bind:value={port} />
        </div>
      {/if}

      {#if serviceType === 'github' || serviceType === 'gitlab' || serviceType === 'gitlab-self-hosted'}
        <div class="form-group">
          <label class="form-label" for="svc-user-login">Nom d'utilisateur / compte</label>
          <input id="svc-user-login" class="input" type="text" placeholder="moncompte" bind:value={user} />
        </div>
        <div class="form-group">
          <label class="form-label" for="svc-token-ref">Référence token (auto-générée)</label>
          <input id="svc-token-ref" class="input input-mono" type="text" readonly value={tokenRef} />
          <span class="form-hint">Identifiant interne du token — non modifiable</span>
        </div>
        <div class="form-group">
          <label class="form-label" for="svc-token-value">Token API (optionnel)</label>
          <input id="svc-token-value" class="input" type="password" placeholder="ghp_xxxxxxxxxxxx" bind:value={tokenValue} />
          <span class="form-hint">Laissez vide pour configurer plus tard dans les Réglages</span>
        </div>
      {/if}
    </div>

  <!-- Step 3: Deploy mode + summary -->
  {:else if step === 3}
    <div class="step-content">
      <p class="step-subtitle">Mode de déploiement</p>
      <div class="deploy-modes">
        {#each deployModes as mode}
          <label class="deploy-mode-option" class:selected={deployMode === mode.id}>
            <input type="radio" name="deployMode" value={mode.id} bind:group={deployMode} />
            <div class="deploy-mode-body">
              <span class="deploy-mode-label">{mode.label}</span>
              <span class="deploy-mode-desc">{mode.desc}</span>
            </div>
          </label>
        {/each}
      </div>

      <!-- Summary -->
      <div class="summary-box">
        <div class="section-label" style="margin-bottom: 10px;">Récapitulatif</div>
        <div class="summary-row">
          <span class="summary-k">Nom</span>
          <span class="summary-v">{name}</span>
        </div>
        <div class="summary-row">
          <span class="summary-k">Type</span>
          <span class="summary-v">{typeLabel(serviceType)}</span>
        </div>
        {#if url}
          <div class="summary-row">
            <span class="summary-k">{serviceType === 'ssh-generic' ? 'Hostname' : 'URL'}</span>
            <span class="summary-v mono">{url}</span>
          </div>
        {/if}
        {#if user}
          <div class="summary-row">
            <span class="summary-k">Utilisateur</span>
            <span class="summary-v mono">{user}</span>
          </div>
        {/if}
        {#if serviceType === 'ssh-generic' && port}
          <div class="summary-row">
            <span class="summary-k">Port</span>
            <span class="summary-v mono">{port}</span>
          </div>
        {/if}
        {#if serviceType === 'github' || serviceType === 'gitlab' || serviceType === 'gitlab-self-hosted'}
          <div class="summary-row">
            <span class="summary-k">Token ref</span>
            <span class="summary-v mono">{tokenRef}</span>
          </div>
          <div class="summary-row">
            <span class="summary-k">Token API</span>
            <span class="summary-v">{tokenValue.trim() ? 'Configuré' : 'Non configuré'}</span>
          </div>
        {/if}
        <div class="summary-row">
          <span class="summary-k">Déploiement</span>
          <span class="summary-v">{deployModeLabel(deployMode)}</span>
        </div>
      </div>

      {#if saveError}
        <div class="form-error">{saveError}</div>
      {/if}
    </div>
  {/if}

  <!-- Navigation buttons -->
  <div class="stepper-nav">
    {#if step > 1}
      <button class="btn btn-ghost" onclick={() => step--}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
        Retour
      </button>
    {:else}
      <button class="btn btn-ghost" onclick={onClose}>Annuler</button>
    {/if}

    <div style="flex:1;"></div>

    {#if step < 3}
      <button
        class="btn btn-primary"
        disabled={step === 1 ? !step1Valid : !step2Valid}
        onclick={() => step++}
      >
        Suivant
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
      </button>
    {:else}
      <button
        class="btn btn-primary"
        disabled={saving}
        onclick={handleCreate}
      >
        {#if saving}
          <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
          Création…
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
          Créer le service
        {/if}
      </button>
    {/if}
  </div>
  {/snippet}
</Modal>

<style>
  .stepper-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }
  .modal-title-inner {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .steps {
    display: flex;
    align-items: center;
    gap: 0;
  }
  .step {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text-disabled);
    font-size: 0.72rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-normal);
  }
  .step.active {
    border-color: var(--accent);
    background: var(--accent-glow);
    color: var(--accent);
  }
  .step.done {
    border-color: var(--ok);
    background: var(--ok-glow);
    color: var(--ok);
  }
  .step-line {
    width: 24px;
    height: 2px;
    background: var(--border);
    transition: background var(--transition-normal);
  }
  .step-line.done { background: var(--ok); opacity: 0.5; }

  .step-content {
    animation: fade-in 200ms ease;
    min-height: 200px;
  }
  .step-subtitle {
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-bottom: 16px;
  }

  /* Type grid */
  .type-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .type-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 18px 12px;
    border-radius: var(--radius-lg);
    border: 2px solid var(--border);
    background: var(--bg-elevated);
    cursor: pointer;
    color: var(--text-secondary);
    transition:
      border-color var(--transition-normal),
      background var(--transition-normal),
      color var(--transition-normal),
      box-shadow var(--transition-normal),
      transform var(--transition-fast);
    outline: none;
  }
  .type-btn:hover {
    background: var(--bg-hover);
    border-color: rgba(255,255,255,0.12);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
  }
  .type-btn.selected {
    border-color: var(--accent);
    background: var(--accent-glow);
    color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }
  .type-icon { font-size: 0; }
  .type-label {
    font-size: 0.85rem;
    font-weight: 600;
  }
  .type-desc {
    font-size: 0.7rem;
    opacity: 0.65;
  }

  /* Form */
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  .form-label {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
  .form-hint {
    font-size: 0.7rem;
    color: var(--text-disabled);
  }
  .form-error {
    margin-top: 12px;
    padding: 10px 12px;
    background: var(--critical-glow);
    border: 1px solid rgba(201,64,64,0.3);
    border-radius: var(--radius-md);
    color: var(--critical);
    font-size: 0.8rem;
  }

  /* Deploy modes */
  .deploy-modes {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 20px;
  }
  .deploy-mode-option {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    border: 2px solid var(--border);
    background: var(--bg-elevated);
    cursor: pointer;
    transition: border-color var(--transition-normal), background var(--transition-normal);
  }
  .deploy-mode-option input[type="radio"] {
    margin-top: 3px;
    accent-color: var(--accent);
    flex-shrink: 0;
  }
  .deploy-mode-option.selected {
    border-color: var(--accent);
    background: var(--accent-glow);
  }
  .deploy-mode-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .deploy-mode-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .deploy-mode-desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* Summary */
  .summary-box {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px;
  }
  .summary-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 5px 0;
    border-bottom: 1px solid var(--border);
    font-size: 0.8rem;
  }
  .summary-row:last-child { border-bottom: none; }
  .summary-k { color: var(--text-secondary); }
  .summary-v { color: var(--text-primary); font-weight: 500; }
  .summary-v.mono { font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-mono); }

  /* Nav */
  .stepper-nav {
    display: flex;
    align-items: center;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    gap: 8px;
  }
</style>
