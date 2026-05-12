<script>
  import { invoke } from '@tauri-apps/api/core';
  import Modal from './Modal.svelte';

  const { keyId, onClose } = $props();

  let newPassphrase    = $state('');
  let confirmPassphrase = $state('');
  let saving           = $state(false);
  let saveError        = $state(null);
  let saved            = $state(false);

  // ---- Password strength ----
  let strength = $derived.by(() => {
    const p = newPassphrase;
    if (!p) return 0;
    let score = 0;
    if (p.length >= 8)  score++;
    if (p.length >= 12) score++;
    if (p.length >= 16) score++;
    if (/[A-Z]/.test(p)) score++;
    if (/[0-9]/.test(p)) score++;
    if (/[^A-Za-z0-9]/.test(p)) score++;
    return score; // 0–6
  });

  let strengthLabel = $derived(
    strength === 0 ? '' :
    strength <= 2  ? 'Faible' :
    strength <= 4  ? 'Moyenne' :
                     'Forte'
  );

  let strengthClass = $derived(
    strength === 0 ? '' :
    strength <= 2  ? 'weak' :
    strength <= 4  ? 'medium' :
                     'strong'
  );

  let valid = $derived(
    newPassphrase.length >= 1 &&
    newPassphrase === confirmPassphrase
  );

  let mismatch = $derived(
    confirmPassphrase.length > 0 && newPassphrase !== confirmPassphrase
  );

  async function handleSave() {
    if (!valid) return;
    saving = true;
    saveError = null;
    try {
      await invoke('add_passphrase', { keyId, passphrase: newPassphrase });
      saved = true;
      setTimeout(() => { onClose(); }, 1500);
    } catch (e) {
      saveError = String(e);
      saving = false;
    }
  }
</script>

<Modal {onClose} maxWidth="400px" title={null}>
  {#snippet children()}
  <div class="pp-header">
    <h3 class="modal-title-inner">Changer la passphrase</h3>
  </div>

  {#if saved}
    <div class="pp-success">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      <p>Passphrase mise à jour avec succès</p>
    </div>
  {:else}
    <div class="form-group">
      <label class="form-label" for="pp-new">Nouvelle passphrase</label>
      <input
        id="pp-new"
        class="input"
        type="password"
        placeholder="Entrez la nouvelle passphrase"
        bind:value={newPassphrase}
        autocomplete="new-password"
      />
      <!-- Strength indicator -->
      {#if newPassphrase.length > 0}
        <div class="strength-bar-wrap">
          <div class="strength-bar">
            {#each [1,2,3,4,5,6] as seg}
              <div class="strength-seg {seg <= strength ? strengthClass : 'empty'}"></div>
            {/each}
          </div>
          <span class="strength-label {strengthClass}">{strengthLabel}</span>
        </div>
      {/if}
    </div>

    <div class="form-group">
      <label class="form-label" for="pp-confirm">Confirmer la passphrase</label>
      <input
        id="pp-confirm"
        class="input"
        class:input-error={mismatch}
        type="password"
        placeholder="Répétez la passphrase"
        bind:value={confirmPassphrase}
        autocomplete="new-password"
      />
      {#if mismatch}
        <span class="form-hint-error">Les passphrases ne correspondent pas</span>
      {/if}
    </div>

    {#if saveError}
      <div class="form-error">{saveError}</div>
    {/if}

    <div class="pp-footer">
      <button class="btn btn-ghost" onclick={onClose} disabled={saving}>Annuler</button>
      <button
        class="btn btn-primary"
        disabled={!valid || saving}
        onclick={handleSave}
      >
        {#if saving}
          <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
          Enregistrement…
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
          Enregistrer
        {/if}
      </button>
    </div>
  {/if}
  {/snippet}
</Modal>

<style>
  .pp-header {
    margin-bottom: 20px;
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
  .form-hint-error {
    font-size: 0.7rem;
    color: var(--critical);
  }
  .input-error {
    border-color: var(--critical) !important;
  }
  .form-error {
    margin-bottom: 12px;
    padding: 10px 12px;
    background: var(--critical-glow);
    border: 1px solid rgba(201,64,64,0.3);
    border-radius: var(--radius-md);
    color: var(--critical);
    font-size: 0.8rem;
  }

  /* Strength bar */
  .strength-bar-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .strength-bar {
    display: flex;
    gap: 3px;
    flex: 1;
  }
  .strength-seg {
    height: 4px;
    border-radius: 2px;
    flex: 1;
    transition: background var(--transition-normal);
  }
  .strength-seg.empty  { background: var(--bg-elevated); }
  .strength-seg.weak   { background: var(--critical); }
  .strength-seg.medium { background: var(--warning); }
  .strength-seg.strong { background: var(--ok); }
  .strength-label {
    font-size: 0.7rem;
    font-weight: 500;
    white-space: nowrap;
  }
  .strength-label.weak   { color: var(--critical); }
  .strength-label.medium { color: var(--warning); }
  .strength-label.strong { color: var(--ok); }

  /* Success */
  .pp-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 24px;
    color: var(--ok);
    animation: scale-in 200ms ease;
  }
  .pp-success p {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  /* Footer */
  .pp-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
</style>
