<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  const { settings, gpgKeys = [], onSave } = $props();

  // Local copy for editing — synced from prop via effect
  let rotationDays = $state(0);
  let minPassLen   = $state(0);
  $effect(() => { rotationDays = settings.rotation_warning_days ?? 90; });
  $effect(() => { minPassLen   = settings.min_passphrase_len ?? 12; });
  let saving          = $state(false);
  let saved           = $state(false);
  let saveError       = $state(null);

  // ---- Tokens API section ----
  let tokenRefs       = $state([]);
  let tokensLoading   = $state(false);
  let tokensError     = $state(null);
  let showAddToken    = $state(false);
  let newTokenRef     = $state('');
  let newTokenValue   = $state('');
  let addingToken     = $state(false);
  let addTokenError   = $state(null);
  let deletingRef     = $state(null);

  async function loadTokenRefs() {
    tokensLoading = true;
    tokensError = null;
    try {
      tokenRefs = await invoke('list_token_refs');
    } catch (e) {
      tokensError = String(e);
    } finally {
      tokensLoading = false;
    }
  }

  onMount(() => { loadTokenRefs(); });

  async function handleAddToken() {
    if (!newTokenRef.trim() || !newTokenValue.trim()) return;
    addingToken = true;
    addTokenError = null;
    try {
      await invoke('set_token', { tokenRef: newTokenRef.trim(), tokenValue: newTokenValue.trim() });
      newTokenRef = '';
      newTokenValue = '';
      showAddToken = false;
      await loadTokenRefs();
    } catch (e) {
      addTokenError = String(e);
    } finally {
      addingToken = false;
    }
  }

  async function handleDeleteToken(ref) {
    deletingRef = ref;
    try {
      await invoke('delete_token', { tokenRef: ref });
      await loadTokenRefs();
    } catch (e) {
      tokensError = String(e);
    } finally {
      deletingRef = null;
    }
  }

  async function handleSave() {
    saving = true;
    saveError = null;
    try {
      await onSave({
        rotation_warning_days: rotationDays,
        min_passphrase_len: minPassLen,
        gpg_fingerprint: settings.gpg_fingerprint,
      });
      saved = true;
      setTimeout(() => { saved = false; }, 2500);
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
    }
  }

  function rotationColor(val) {
    if (val <= 30)  return 'var(--critical)';
    if (val <= 60)  return 'var(--warning)';
    return 'var(--ok)';
  }
</script>

<div class="settings-page">
  <div class="settings-content">

    <!-- Rotation section -->
    <section class="settings-section">
      <div class="section-label">Rotation des clefs</div>
      <div class="settings-card">
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-title">Avertissement de rotation</span>
            <span class="setting-desc">Durée avant laquelle vous êtes averti qu'une clef doit être renouvelée.</span>
          </div>
          <div class="setting-control rotation-control">
            <span class="rotation-value" style="color: {rotationColor(rotationDays)};">{rotationDays} jours</span>
            <input
              type="range"
              min="7"
              max="365"
              step="1"
              class="range-input"
              bind:value={rotationDays}
            />
            <div class="range-marks">
              <span>7j</span>
              <span>90j</span>
              <span>180j</span>
              <span>365j</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Security section -->
    <section class="settings-section">
      <div class="section-label">Sécurité</div>
      <div class="settings-card">
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-title">Longueur minimale de passphrase</span>
            <span class="setting-desc">Nombre de caractères minimum requis pour les passphrases de clefs.</span>
          </div>
          <div class="setting-control">
            <input
              type="number"
              min="8"
              max="64"
              class="input number-input"
              bind:value={minPassLen}
            />
            <span class="setting-unit">caractères</span>
          </div>
        </div>
      </div>
    </section>

    <!-- GPG section -->
    <section class="settings-section">
      <div class="section-label">Chiffrement GPG</div>
      <div class="settings-card">
        {#if settings.gpg_fingerprint}
          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-title">Clef GPG active</span>
              <span class="setting-desc">Clef utilisée pour chiffrer les sauvegardes et les données sensibles.</span>
            </div>
            <div class="setting-control">
              <code class="gpg-fp">{settings.gpg_fingerprint}</code>
            </div>
          </div>
        {:else}
          <div class="gpg-unconfigured">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            <span>GPG non configuré — les sauvegardes ne sont pas chiffrées.</span>
          </div>
        {/if}

        {#if gpgKeys.length > 0}
          <div class="divider"></div>
          <div class="setting-item">
            <div class="setting-info">
              <span class="setting-title">Clefs GPG disponibles</span>
            </div>
          </div>
          <div class="gpg-keys-list">
            {#each gpgKeys as gpgKey}
              <div class="gpg-key-row">
                <code class="gpg-fp-sm">{gpgKey.fingerprint ?? gpgKey.id ?? gpgKey}</code>
                {#if gpgKey.name || gpgKey.email}
                  <span class="gpg-uid">{gpgKey.name ?? ''} {gpgKey.email ? `<${gpgKey.email}>` : ''}</span>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Tokens API section -->
    <section class="settings-section">
      <div class="section-label">Tokens API</div>
      <div class="settings-card">

        {#if tokensLoading}
          <div class="tokens-loading">
            <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
            Chargement…
          </div>
        {:else if tokensError}
          <div class="tokens-error">{tokensError}</div>
        {:else if tokenRefs.length === 0}
          <div class="tokens-empty">Aucun token configuré.</div>
        {:else}
          <div class="tokens-list">
            {#each tokenRefs as ref}
              <div class="token-row">
                <code class="token-ref-code">{ref}</code>
                <button
                  class="btn btn-danger btn-sm"
                  disabled={deletingRef === ref}
                  onclick={() => handleDeleteToken(ref)}
                >
                  {#if deletingRef === ref}
                    <span class="spinner" style="width:12px;height:12px;border-width:2px;"></span>
                  {:else}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/></svg>
                    Supprimer
                  {/if}
                </button>
              </div>
            {/each}
          </div>
        {/if}

        <div class="tokens-footer">
          {#if !showAddToken}
            <button class="btn btn-secondary btn-sm" onclick={() => showAddToken = true}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
              Ajouter un token
            </button>
          {:else}
            <div class="add-token-form">
              <div class="form-group">
                <label class="form-label" for="new-token-ref">Référence</label>
                <input
                  id="new-token-ref"
                  class="input input-mono"
                  type="text"
                  placeholder="sshive/mon_service"
                  bind:value={newTokenRef}
                />
              </div>
              <div class="form-group">
                <label class="form-label" for="new-token-val">Valeur du token</label>
                <input
                  id="new-token-val"
                  class="input"
                  type="password"
                  placeholder="ghp_xxxxxxxxxxxx"
                  bind:value={newTokenValue}
                />
              </div>
              {#if addTokenError}
                <div class="form-error">{addTokenError}</div>
              {/if}
              <div class="add-token-actions">
                <button class="btn btn-ghost btn-sm" onclick={() => { showAddToken = false; addTokenError = null; newTokenRef = ''; newTokenValue = ''; }}>
                  Annuler
                </button>
                <button
                  class="btn btn-primary btn-sm"
                  disabled={addingToken || !newTokenRef.trim() || !newTokenValue.trim()}
                  onclick={handleAddToken}
                >
                  {#if addingToken}
                    <span class="spinner" style="width:12px;height:12px;border-width:2px;"></span>
                    Enregistrement…
                  {:else}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                    Enregistrer
                  {/if}
                </button>
              </div>
            </div>
          {/if}
        </div>

      </div>
    </section>

    <!-- Save button -->
    <div class="save-row">
      {#if saveError}
        <span class="save-error">{saveError}</span>
      {/if}
      {#if saved}
        <span class="save-ok">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
          Paramètres enregistrés
        </span>
      {/if}
      <button class="btn btn-primary" onclick={handleSave} disabled={saving}>
        {#if saving}
          <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
          Enregistrement…
        {:else}
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
          Enregistrer
        {/if}
      </button>
    </div>

  </div>
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px;
    max-width: 680px;
    width: 100%;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .settings-card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .setting-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding: 16px 20px;
  }
  .setting-item:not(:first-child) {
    border-top: 1px solid var(--border);
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
  }
  .setting-title {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
  }
  .setting-desc {
    font-size: 0.75rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .setting-control {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-end;
    flex-shrink: 0;
    min-width: 200px;
  }

  /* Rotation slider */
  .rotation-control { min-width: 220px; }
  .rotation-value {
    font-size: 1rem;
    font-weight: 700;
    font-family: var(--font-ui);
    align-self: flex-end;
  }

  .range-input {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--bg-elevated);
    outline: none;
    cursor: pointer;
  }
  .range-input::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent);
    cursor: pointer;
    box-shadow: 0 0 0 3px var(--accent-glow);
    transition: box-shadow var(--transition-fast);
  }
  .range-input::-webkit-slider-thumb:hover {
    box-shadow: 0 0 0 5px var(--accent-glow);
  }

  .range-marks {
    display: flex;
    justify-content: space-between;
    width: 100%;
    font-size: 0.65rem;
    color: var(--text-disabled);
  }

  /* Number input */
  .number-input {
    width: 80px;
    text-align: center;
  }
  .setting-unit {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* GPG */
  .gpg-unconfigured {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 20px;
    color: var(--warning);
    font-size: 0.8rem;
    background: var(--warning-glow);
  }

  .gpg-fp {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-mono);
    background: var(--bg-base);
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .gpg-keys-list {
    padding: 0 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .gpg-key-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .gpg-fp-sm {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-mono);
  }
  .gpg-uid {
    font-size: 0.72rem;
    color: var(--text-secondary);
  }

  /* Tokens section */
  .tokens-loading {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px 20px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }
  .tokens-error {
    padding: 12px 20px;
    font-size: 0.8rem;
    color: var(--critical);
    background: var(--critical-glow);
  }
  .tokens-empty {
    padding: 14px 20px;
    font-size: 0.8rem;
    color: var(--text-disabled);
    font-style: italic;
  }
  .tokens-list {
    display: flex;
    flex-direction: column;
  }
  .token-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    border-bottom: 1px solid var(--border);
    gap: 12px;
  }
  .token-row:last-child {
    border-bottom: none;
  }
  .token-ref-code {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-mono);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tokens-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }
  .add-token-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    animation: fade-in 150ms ease;
  }
  .add-token-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .form-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
  .form-error {
    font-size: 0.78rem;
    color: var(--critical);
    padding: 8px 10px;
    background: var(--critical-glow);
    border: 1px solid rgba(201,64,64,0.25);
    border-radius: var(--radius-sm);
  }

  /* Save row */
  .save-row {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: flex-end;
  }
  .save-error {
    font-size: 0.8rem;
    color: var(--critical);
  }
  .save-ok {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 0.8rem;
    color: var(--ok);
    animation: fade-in 200ms ease;
  }
</style>
