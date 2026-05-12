<script>
  import { invoke } from '@tauri-apps/api/core';
  import Modal from './Modal.svelte';

  const { services = [], onConfirm, onClose } = $props();

  let keyType       = $state('ed25519');
  let passphrase    = $state('');
  let passphraseConf = $state('');
  let serviceId     = $state('');

  let generating    = $state(false);
  let genError      = $state(null);
  let generatedKey  = $state(null);

  // Passphrase strength
  let passStrength = $derived(
    passphrase.length === 0 ? 'none'
    : passphrase.length < 12 ? 'weak'
    : passphrase.length < 16 ? 'medium'
    : 'strong'
  );

  let passMatch = $derived(passphrase === passphraseConf || passphraseConf === '');
  let passConfMatch = $derived(passphrase === passphraseConf);
  let canGenerate = $derived(
    passphrase.length >= 1 &&
    passConfMatch &&
    (passphrase.length === 0 || passphrase === passphraseConf)
  );

  function strengthLabel(s) {
    switch (s) {
      case 'weak':   return 'Faible';
      case 'medium': return 'Correcte';
      case 'strong': return 'Forte';
      default:       return '';
    }
  }
  function strengthColor(s) {
    switch (s) {
      case 'weak':   return 'var(--critical)';
      case 'medium': return 'var(--warning)';
      case 'strong': return 'var(--ok)';
      default:       return 'transparent';
    }
  }
  function strengthWidth(s) {
    switch (s) {
      case 'weak':   return '33%';
      case 'medium': return '66%';
      case 'strong': return '100%';
      default:       return '0%';
    }
  }

  async function handleGenerate() {
    if (!canGenerate) return;
    generating = true;
    genError = null;
    try {
      const result = await invoke('generate_key', {
        serviceId: serviceId || null,
        keyType,
        passphrase: passphrase || null,
      });
      generatedKey = result;
    } catch (e) {
      genError = String(e);
      generating = false;
    }
  }

  function handleDone() {
    onConfirm(generatedKey);
  }

  const keyTypes = [
    {
      id: 'ed25519',
      label: 'Ed25519',
      desc: 'Recommandé — rapide et sécurisé',
      recommended: true,
    },
    {
      id: 'sk-ed25519',
      label: 'sk-ed25519',
      desc: 'YubiKey / clef matérielle FIDO2',
      recommended: false,
    },
  ];
</script>

<Modal {onClose} maxWidth="460px" title={null}>
  {#snippet children()}
  {#if !generatedKey}
    <!-- Generation form -->
    <div class="keygen-header">
      <h3 class="modal-title-inner">Générer une clef SSH</h3>
    </div>

    <!-- Key type -->
    <div class="form-group">
      <div class="section-label">Type de clef</div>
      <div class="keytype-options">
        {#each keyTypes as kt}
          <label class="keytype-option" class:selected={keyType === kt.id}>
            <input type="radio" name="keyType" value={kt.id} bind:group={keyType} />
            <div class="keytype-body">
              <div class="keytype-head">
                <span class="keytype-label">{kt.label}</span>
                {#if kt.recommended}
                  <span class="badge ok" style="font-size:0.6rem;padding:1px 6px;">Recommandé</span>
                {:else}
                  <span class="badge warning" style="font-size:0.6rem;padding:1px 6px;">
                    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="2"/><circle cx="12" cy="14" r="2"/></svg>
                    YubiKey requis
                  </span>
                {/if}
              </div>
              <span class="keytype-desc">{kt.desc}</span>
            </div>
          </label>
        {/each}
      </div>
    </div>

    <!-- Service associé (optionnel) -->
    <div class="form-group">
      <label class="form-label" for="key-service">Service associé (optionnel)</label>
      <select id="key-service" class="input" bind:value={serviceId}>
        <option value="">-- Aucun service --</option>
        {#each services as s}
          <option value={s.id}>{s.name}</option>
        {/each}
      </select>
    </div>

    <!-- Passphrase -->
    <div class="form-group">
      <label class="form-label" for="key-pass">Passphrase</label>
      <input
        id="key-pass"
        class="input"
        type="password"
        placeholder="Entrez une passphrase sécurisée"
        bind:value={passphrase}
        autocomplete="new-password"
      />
      {#if passphrase.length > 0}
        <!-- Strength bar -->
        <div class="strength-bar-wrap">
          <div class="strength-bar">
            <div
              class="strength-fill"
              style="width:{strengthWidth(passStrength)};background:{strengthColor(passStrength)};"
            ></div>
          </div>
          <span class="strength-label" style="color:{strengthColor(passStrength)};">
            {strengthLabel(passStrength)}
          </span>
        </div>
        <span class="form-hint">
          {passphrase.length} caractère{passphrase.length > 1 ? 's' : ''} — minimum recommandé : 16
        </span>
      {/if}
    </div>

    <div class="form-group">
      <label class="form-label" for="key-pass-conf">Confirmation</label>
      <input
        id="key-pass-conf"
        class="input"
        class:input-error={passphraseConf.length > 0 && !passConfMatch}
        type="password"
        placeholder="Confirmez la passphrase"
        bind:value={passphraseConf}
        autocomplete="new-password"
      />
      {#if passphraseConf.length > 0 && !passConfMatch}
        <span class="form-error-inline">Les passphrases ne correspondent pas</span>
      {/if}
    </div>

    {#if genError}
      <div class="gen-error">{genError}</div>
    {/if}

    <!-- Actions -->
    <div class="keygen-nav">
      <button class="btn btn-ghost" onclick={onClose}>Annuler</button>
      <button
        class="btn btn-primary"
        disabled={!canGenerate || generating}
        onclick={handleGenerate}
      >
        {#if generating}
          <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
          Génération…
        {:else}
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="7.5" cy="15.5" r="5.5"/><path d="M21 2l-9.6 9.6M15.5 7.5l2 2"/></svg>
          Générer la clef
        {/if}
      </button>
    </div>

  {:else}
    <!-- Result screen -->
    <div class="result-screen">
      <div class="result-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      </div>
      <h3 style="color: var(--ok);">Clef générée avec succès</h3>
      <p class="result-type">{generatedKey.key_type} — {generatedKey.yubikey ? 'YubiKey' : 'Logicielle'}</p>

      <div class="result-fp-label">Fingerprint</div>
      <div class="result-fp-box">
        <code>{generatedKey.fingerprint}</code>
      </div>

      {#if generatedKey.public_path}
        <div class="result-path">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
          {generatedKey.public_path}
        </div>
      {/if}

      <button class="btn btn-primary" style="margin-top:8px;width:100%;justify-content:center;" onclick={handleDone}>
        Fermer
      </button>
    </div>
  {/if}
  {/snippet}
</Modal>

<style>
  .keygen-header {
    margin-bottom: 18px;
  }
  .modal-title-inner {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

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
  .form-error-inline {
    font-size: 0.72rem;
    color: var(--critical);
  }
  .gen-error {
    padding: 10px 12px;
    background: var(--critical-glow);
    border: 1px solid rgba(201,64,64,0.3);
    border-radius: var(--radius-md);
    color: var(--critical);
    font-size: 0.8rem;
    margin-bottom: 14px;
  }

  .input-error {
    border-color: rgba(201, 64, 64, 0.5) !important;
    box-shadow: 0 0 0 3px var(--critical-glow) !important;
  }

  /* Key type selector */
  .keytype-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .keytype-option {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    border: 2px solid var(--border);
    background: var(--bg-elevated);
    cursor: pointer;
    transition: border-color var(--transition-normal), background var(--transition-normal);
  }
  .keytype-option input[type="radio"] {
    margin-top: 3px;
    accent-color: var(--accent);
    flex-shrink: 0;
  }
  .keytype-option.selected {
    border-color: var(--accent);
    background: var(--accent-glow);
  }
  .keytype-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .keytype-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .keytype-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }
  .keytype-desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* Strength bar */
  .strength-bar-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .strength-bar {
    flex: 1;
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
  }
  .strength-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease, background 300ms ease;
  }
  .strength-label {
    font-size: 0.7rem;
    font-weight: 600;
    width: 60px;
    text-align: right;
  }

  /* Nav */
  .keygen-nav {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  /* Result */
  .result-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
    animation: scale-in 200ms ease;
  }
  .result-icon {
    color: var(--ok);
    opacity: 0.8;
  }
  .result-type {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .result-fp-label {
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-disabled);
    align-self: flex-start;
  }
  .result-fp-box {
    width: 100%;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 14px;
  }
  .result-fp-box code {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-mono);
    word-break: break-all;
    display: block;
    text-align: left;
  }
  .result-path {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-disabled);
    align-self: flex-start;
  }

  /* Select */
  select.input {
    background: var(--bg-base);
    cursor: pointer;
  }
  select.input option {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }
</style>
