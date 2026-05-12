<script>
  import Modal from './Modal.svelte';

  const { gpgKeys = [], onConfirm } = $props();

  let selectedFp = $state('');
  $effect(() => { selectedFp = gpgKeys.length > 0 ? (gpgKeys[0]?.fingerprint ?? gpgKeys[0]) : ''; });
  let confirming = $state(false);

  function handleConfirm() {
    if (!selectedFp) return;
    confirming = true;
    onConfirm(selectedFp);
  }

  function displayFp(key) {
    if (typeof key === 'string') return key;
    return key.fingerprint ?? key.id ?? JSON.stringify(key);
  }
  function displayUid(key) {
    if (typeof key === 'string') return null;
    const name = key.name ?? '';
    const email = key.email ? `<${key.email}>` : '';
    const uid = `${name} ${email}`.trim();
    return uid || null;
  }
</script>

<Modal onClose={() => {}} maxWidth="460px" title={null}>
  {#snippet children()}
  <div class="gpg-setup">
    <!-- Header -->
    <div class="gpg-header">
      <div class="gpg-shield-icon">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      </div>
      <div>
        <h3 class="gpg-title">Configuration GPG</h3>
        <p class="gpg-subtitle">Chiffrez vos sauvegardes avec une clef GPG</p>
      </div>
    </div>

    {#if gpgKeys.length === 0}
      <!-- No GPG keys available -->
      <div class="no-gpg">
        <div class="no-gpg-icon">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        </div>
        <p>Aucune clef GPG trouvée sur ce système.</p>
        <div class="gpg-help">
          <p class="help-title">Pour créer une clef GPG :</p>
          <div class="snippet-box">
            <pre class="snippet-code">gpg --full-generate-key</pre>
          </div>
          <p class="help-hint">Choisissez RSA 4096 bits ou Ed25519, puis relancez SSHive.</p>
        </div>
      </div>

      <div class="gpg-footer">
        <p class="skip-note">Vous pouvez configurer GPG plus tard dans les Paramètres.</p>
        <button class="btn btn-ghost" onclick={() => onConfirm(null)}>Ignorer pour l'instant</button>
      </div>

    {:else}
      <!-- GPG key selection -->
      <div class="gpg-desc">
        Sélectionnez la clef GPG à utiliser pour chiffrer les sauvegardes SSHive :
      </div>

      <div class="gpg-keys-list">
        {#each gpgKeys as key}
          {@const fp = displayFp(key)}
          {@const uid = displayUid(key)}
          <label class="gpg-key-option" class:selected={selectedFp === fp}>
            <input type="radio" name="gpgKey" value={fp} bind:group={selectedFp} />
            <div class="gpg-key-body">
              <code class="gpg-fp">{fp}</code>
              {#if uid}
                <span class="gpg-uid">{uid}</span>
              {/if}
            </div>
          </label>
        {/each}
      </div>

      <div class="gpg-footer">
        <div></div>
        <button
          class="btn btn-primary"
          disabled={!selectedFp || confirming}
          onclick={handleConfirm}
        >
          {#if confirming}
            <span class="spinner" style="width:14px;height:14px;border-width:2px;"></span>
            Configuration…
          {:else}
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Configurer GPG
          {/if}
        </button>
      </div>
    {/if}
  </div>
  {/snippet}
</Modal>

<style>
  .gpg-setup {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .gpg-header {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .gpg-shield-icon {
    color: var(--accent);
    opacity: 0.8;
    flex-shrink: 0;
  }
  .gpg-title {
    font-size: 1.05rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .gpg-subtitle {
    font-size: 0.78rem;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .gpg-desc {
    font-size: 0.82rem;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  /* No GPG keys */
  .no-gpg {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    text-align: center;
  }
  .no-gpg-icon {
    color: var(--warning);
    opacity: 0.7;
  }
  .no-gpg > p {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .gpg-help {
    width: 100%;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .help-title {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-secondary);
  }
  .snippet-box {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
  }
  .snippet-code {
    font-family: var(--font-mono);
    font-size: 0.78rem;
    color: var(--text-mono);
    margin: 0;
  }
  .help-hint {
    font-size: 0.72rem;
    color: var(--text-disabled);
    line-height: 1.5;
  }

  /* Key list */
  .gpg-keys-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 240px;
    overflow-y: auto;
  }

  .gpg-key-option {
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
  .gpg-key-option input[type="radio"] {
    margin-top: 4px;
    accent-color: var(--accent);
    flex-shrink: 0;
  }
  .gpg-key-option.selected {
    border-color: var(--accent);
    background: var(--accent-glow);
  }

  .gpg-key-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .gpg-fp {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-mono);
    word-break: break-all;
    display: block;
  }
  .gpg-uid {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  /* Footer */
  .gpg-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
  .skip-note {
    font-size: 0.72rem;
    color: var(--text-disabled);
  }
</style>
