<script>
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import PassphraseModal from '../components/PassphraseModal.svelte';

  const { keys, services, onGenerate } = $props();

  function linkedServiceName(key) {
    if (key.linked_service_name) return key.linked_service_name;
    if (key.service_id) {
      const s = services.find(s => s.id === key.service_id);
      return s?.name ?? 'Service inconnu';
    }
    return null;
  }

  function protectionClass(protection) {
    switch (protection) {
      case 'protected':   return 'ok';
      case 'unprotected': return 'warning';
      default:            return 'neutral';
    }
  }

  function protectionLabel(protection) {
    switch (protection) {
      case 'protected':   return 'Protégée';
      case 'unprotected': return 'Non protégée';
      default:            return 'Inconnue';
    }
  }

  let copyFeedbacks = $state({});

  async function copyFingerprint(id, fingerprint) {
    await writeText(fingerprint);
    copyFeedbacks = { ...copyFeedbacks, [id]: true };
    setTimeout(() => {
      copyFeedbacks = { ...copyFeedbacks, [id]: false };
    }, 2000);
  }

  // ---- Passphrase modal ----
  let passphraseKeyId = $state(null);
</script>

{#if passphraseKeyId !== null}
  <PassphraseModal keyId={passphraseKeyId} onClose={() => passphraseKeyId = null} />
{/if}

<div class="keys-page">
  <div class="keys-header">
    <div>
      <h2 style="color: var(--text-primary);">Clefs SSH</h2>
      <p style="color: var(--text-secondary); font-size: 0.8rem; margin-top: 2px;">{keys.length} clef{keys.length !== 1 ? 's' : ''} enregistrée{keys.length !== 1 ? 's' : ''}</p>
    </div>
    <button class="btn btn-primary" onclick={onGenerate}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      Générer une clef
    </button>
  </div>

  {#if keys.length === 0}
    <div class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="7.5" cy="15.5" r="5.5"/><path d="M21 2l-9.6 9.6M15.5 7.5l2 2"/></svg>
      <p>Aucune clef générée</p>
      <button class="btn btn-primary" onclick={onGenerate}>Générer ma première clef</button>
    </div>
  {:else}
    <div class="keys-grid stagger">
      {#each keys as key (key.id)}
        <div class="key-card">
          <!-- Header -->
          <div class="key-card-header">
            <div class="key-badges">
              <span class="badge accent">{key.key_type}</span>
              {#if key.yubikey}
                <span class="badge warning">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="2" width="14" height="20" rx="2"/><circle cx="12" cy="14" r="2"/></svg>
                  YubiKey
                </span>
              {/if}
            </div>
            <button
              class="copy-fp-btn"
              title="Copier le fingerprint"
              onclick={() => copyFingerprint(key.id, key.fingerprint)}
            >
              {#if copyFeedbacks[key.id]}
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              {:else}
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              {/if}
            </button>
          </div>

          <!-- Fingerprint -->
          <div class="key-fingerprint">
            <code>{key.fingerprint}</code>
          </div>

          <!-- Comment -->
          {#if key.comment}
            <div class="key-comment">{key.comment}</div>
          {/if}

          <!-- Footer infos -->
          <div class="key-footer">
            <div class="key-info-row">
              <span class="key-info-label">Protection</span>
              <span class="badge {protectionClass(key.protection)}" style="font-size: 0.65rem; padding: 1px 6px;">
                {protectionLabel(key.protection)}
              </span>
            </div>
            {#if linkedServiceName(key)}
              <div class="key-info-row">
                <span class="key-info-label">Service</span>
                <span class="key-service-tag">{linkedServiceName(key)}</span>
              </div>
            {/if}
            <div class="key-info-row">
              <span class="key-info-label">Créée le</span>
              <span class="key-info-val">{key.created_at}</span>
            </div>
          </div>

          <!-- Paths -->
          {#if key.public_path}
            <div class="key-path">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
              <span>{key.public_path}</span>
            </div>
          {/if}

          {#if !key.backup_prompted}
            <div class="backup-warning">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
              Backup non effectué
            </div>
          {/if}

          <!-- Key actions -->
          <div class="key-actions">
            <button
              class="btn btn-secondary btn-sm key-action-btn"
              onclick={() => passphraseKeyId = key.id}
            >
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
              Changer la passphrase
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .keys-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .keys-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .keys-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
    align-content: start;
  }

  /* Key card */
  .key-card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition:
      border-color var(--transition-normal),
      box-shadow var(--transition-normal),
      transform var(--transition-fast);
  }
  .key-card:hover {
    border-color: var(--border-active);
    box-shadow: 0 4px 20px rgba(0,0,0,0.3), var(--shadow-glow-accent);
    transform: translateY(-2px);
  }

  .key-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .key-badges {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .copy-fp-btn {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: color var(--transition-fast), background var(--transition-fast);
  }
  .copy-fp-btn:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
  }

  .key-fingerprint {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }
  .key-fingerprint code {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-mono);
    word-break: break-all;
    line-height: 1.6;
    display: block;
  }

  .key-comment {
    font-size: 0.78rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .key-footer {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .key-info-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
  }
  .key-info-label { color: var(--text-disabled); }
  .key-info-val   { color: var(--text-secondary); }
  .key-service-tag {
    font-size: 0.72rem;
    color: var(--accent);
    font-weight: 500;
  }

  .key-path {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: var(--font-mono);
    font-size: 0.65rem;
    color: var(--text-disabled);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .key-path svg { flex-shrink: 0; }

  .backup-warning {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 0.72rem;
    color: var(--warning);
    background: var(--warning-glow);
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(212,150,42,0.2);
  }

  /* Key actions */
  .key-actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-top: 4px;
    border-top: 1px solid var(--border);
  }
  .key-action-btn {
    width: 100%;
    justify-content: center;
    font-size: 0.75rem;
  }
</style>
