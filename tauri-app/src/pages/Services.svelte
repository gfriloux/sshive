<script>
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';

  const {
    services,
    selectedId,
    onSelect,
    onAdd,
    onDelete,
    onDeploy,
    onRevoke,
    keys,
  } = $props();

  // ---- Token modal ----
  let showTokenModal = $state(false);
  let tokenModalValue = $state('');
  let tokenSaving = $state(false);
  let tokenSaveError = $state(null);
  let tokenSaved = $state(false);

  $effect(() => {
    if (selectedId !== null) {
      showTokenModal = false;
      tokenModalValue = '';
      tokenSaveError = null;
      tokenSaved = false;
    }
  });

  async function saveToken() {
    if (!selected?.token_ref || !tokenModalValue.trim()) return;
    tokenSaving = true;
    tokenSaveError = null;
    try {
      await invoke('set_token', { tokenRef: selected.token_ref, tokenValue: tokenModalValue.trim() });
      tokenSaved = true;
      tokenModalValue = '';
      setTimeout(() => { tokenSaved = false; showTokenModal = false; }, 1800);
    } catch (e) {
      tokenSaveError = String(e);
    } finally {
      tokenSaving = false;
    }
  }

  // ---- Derived: selected service ----
  let selected = $derived(services.find(s => s.id === selectedId) ?? null);
  let selectedKey = $derived(
    selected?.active_key
      ? keys.find(k => k.id === selected.active_key) ?? null
      : null
  );

  // ---- Type icon helpers ----
  function typeIcon(serviceType) {
    switch (serviceType) {
      case 'github':
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.58 2 12.26c0 4.54 2.87 8.39 6.84 9.75.5.09.68-.22.68-.49v-1.7c-2.78.62-3.37-1.37-3.37-1.37-.45-1.18-1.11-1.49-1.11-1.49-.91-.64.07-.63.07-.63 1 .07 1.53 1.06 1.53 1.06.89 1.57 2.34 1.12 2.91.85.09-.66.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.71 0 0 .84-.28 2.75 1.05A9.3 9.3 0 0 1 12 7.4c.85 0 1.7.12 2.5.34 1.91-1.33 2.75-1.05 2.75-1.05.55 1.41.2 2.45.1 2.71.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.81-4.57 5.06.36.32.68.94.68 1.9v2.82c0 .27.18.59.69.49C19.13 20.65 22 16.8 22 12.26 22 6.58 17.52 2 12 2z"/></svg>`;
      case 'gitlab':
      case 'gitlab-self-hosted':
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z"/></svg>`;
      default:
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.8 19.8 0 0 1-8.63-3.07A19.5 19.5 0 0 1 4.69 12 19.8 19.8 0 0 1 1.62 3.37 2 2 0 0 1 3.6 1h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L7.91 8.6a16 16 0 0 0 6 6l.96-.96a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 21.73 16l.27.92z"/></svg>`;
    }
  }

  function typeLabel(serviceType) {
    switch (serviceType) {
      case 'github':            return 'GitHub';
      case 'gitlab':            return 'GitLab';
      case 'gitlab-self-hosted': return 'GitLab SH';
      default:                  return 'SSH';
    }
  }

  function serviceHostname(service) {
    if (service.params?.url) return service.params.url;
    if (service.params?.user && service.params?.port) return `${service.params.user}@<host>:${service.params.port}`;
    if (service.params?.user) return service.params.user;
    return null;
  }

  function rotationBadgeClass(days) {
    if (days === null || days === undefined) return 'neutral';
    if (days <= 7)  return 'critical';
    if (days <= 14) return 'warning';
    return 'ok';
  }

  function rotationLabel(days) {
    if (days === null || days === undefined) return 'N/A';
    if (days <= 0) return 'Expirée';
    return `${days}j`;
  }

  function deployModeLabel(mode) {
    switch (mode) {
      case 'automatic':   return 'Auto';
      case 'guided':      return 'Guidé';
      case 'external-cm': return 'CM ext.';
      default:            return mode;
    }
  }

  // ---- Public key copy ----
  let publicKey = $state(null);
  let copyFeedback = $state(false);
  let loadingPublicKey = $state(false);

  $effect(() => {
    let cancelled = false;
    if (selected) {
      publicKey = selected.public_key ?? null;
      if (!publicKey && selected.active_key) {
        loadingPublicKey = true;
        invoke('get_public_key', { keyId: selected.active_key })
          .then(k => { if (!cancelled) publicKey = k; })
          .catch(() => { if (!cancelled) publicKey = null; })
          .finally(() => { if (!cancelled) loadingPublicKey = false; });
      }
    } else {
      publicKey = null;
    }
    return () => { cancelled = true; };
  });

  async function copyPublicKey() {
    if (!publicKey) return;
    await writeText(publicKey);
    copyFeedback = true;
    setTimeout(() => { copyFeedback = false; }, 2000);
  }

  // ---- Health banner ----
  function healthBannerClass(level) {
    switch (level) {
      case 'ok':       return 'health-ok';
      case 'warning':  return 'health-warning';
      case 'critical': return 'health-critical';
      default:         return 'health-info';
    }
  }

  function healthIcon(level) {
    switch (level) {
      case 'ok':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>`;
      case 'warning':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`;
      case 'critical':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>`;
      default:
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`;
    }
  }

  // ---- Confirm delete ----
  let confirmingDelete = $state(false);

  $effect(() => {
    if (selectedId !== null) confirmingDelete = false;
  });

  // ---- Assign existing key ----
  let showKeyPicker   = $state(false);
  let assigningKey    = $state(false);
  let assignError     = $state(null);

  // Clefs disponibles : gérées (private_path présent) et non liées à un autre service
  let assignableKeys = $derived(
    keys.filter(k => k.private_path && (!k.service_id || k.service_id === selectedId))
  );

  async function handleAssignKey(keyId) {
    assigningKey = true;
    assignError = null;
    try {
      await invoke('assign_key', { serviceId: selectedId, keyId });
      showKeyPicker = false;
      // Recharge l'état global via le callback onSelect (force refresh)
      onSelect(selectedId);
    } catch (e) {
      assignError = String(e);
    } finally {
      assigningKey = false;
    }
  }

  $effect(() => {
    if (selectedId) { showKeyPicker = false; assignError = null; }
  });
</script>

<div class="services-layout">
  <!-- Service list -->
  <div class="list-pane">
    <div class="list-header">
      <div class="col-header col-status">ST</div>
      <div class="col-header col-type">TYPE</div>
      <div class="col-header col-name">SERVICE</div>
      <div class="col-header col-rotation">ROT.</div>
    </div>

    <div class="list-body stagger">
      {#each services as service (service.id)}
        <button
          class="service-row"
          class:selected={service.id === selectedId}
          onclick={() => onSelect(service.id)}
        >
          <!-- Status dot -->
          <div class="col-status">
            <span class="status-dot {service.health_level}"></span>
          </div>

          <!-- Type icon -->
          <div class="col-type">
            <span class="type-icon">{@html typeIcon(service.service_type)}</span>
          </div>

          <!-- Name + hostname -->
          <div class="col-name">
            <span class="service-name truncate">{service.name}</span>
            {#if serviceHostname(service)}
              <span class="service-host truncate">{serviceHostname(service)}</span>
            {/if}
          </div>

          <!-- Rotation badge -->
          <div class="col-rotation">
            <span class="badge {rotationBadgeClass(service.rotation_age_days)}">
              {rotationLabel(service.rotation_age_days)}
            </span>
          </div>
        </button>
      {:else}
        <div class="empty-state">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>
          <p>Aucun service configuré</p>
        </div>
      {/each}
    </div>

    <div class="list-footer">
      <button class="btn btn-primary" onclick={onAdd}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        Ajouter un service
      </button>
    </div>
  </div>

  <!-- Detail panel -->
  {#if selected}
    {#key selected.id}
    <div class="detail-panel">
      <!-- Header -->
      <div class="detail-header">
        <div class="detail-header-left">
          <span class="status-dot {selected.health_level}"></span>
          <h2 class="detail-title">{selected.name}</h2>
        </div>
        <button class="btn btn-ghost btn-sm" aria-label="Fermer le panneau" onclick={() => onSelect(null)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>

      {#if serviceHostname(selected)}
        <div class="detail-host">{serviceHostname(selected)}</div>
      {/if}

      <!-- Scrollable content -->
      <div class="detail-scroll">

        <!-- Health banner -->
        <div class="health-banner {healthBannerClass(selected.health_level)}">
          <span class="health-icon">{@html healthIcon(selected.health_level)}</span>
          <div class="health-content">
            {#if selected.health_reasons?.length > 0}
              <ul class="health-reasons">
                {#each selected.health_reasons as reason}
                  <li>{reason}</li>
                {/each}
              </ul>
            {:else}
              <span>Tout est nominal</span>
            {/if}
          </div>
        </div>

        <!-- Informations -->
        <div class="detail-section">
          <div class="section-label">Informations</div>
          <div class="info-box">
            <div class="info-row">
              <span class="label">Type</span>
              <span class="value">{typeLabel(selected.service_type)}</span>
            </div>
            <div class="info-row">
              <span class="label">Déploiement</span>
              <span class="value">{deployModeLabel(selected.deploy_mode)}</span>
            </div>
            <div class="info-row">
              <span class="label">Créé le</span>
              <span class="value">{selected.created_at}</span>
            </div>
            {#if selected.last_rotation}
              <div class="info-row">
                <span class="label">Dernière rotation</span>
                <span class="value">{selected.last_rotation}</span>
              </div>
            {/if}
            {#if selectedKey}
              <div class="info-row">
                <span class="label">YubiKey</span>
                <span class="value">{selectedKey.yubikey ? 'Oui' : 'Non'}</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Fingerprint -->
        {#if selectedKey}
          <div class="detail-section">
            <div class="section-label">Fingerprint</div>
            <div class="fingerprint-box">
              <span class="badge neutral" style="margin-bottom:6px;">{selectedKey.key_type}</span>
              <code class="fingerprint-code">{selectedKey.fingerprint}</code>
            </div>
          </div>
        {/if}

        <!-- Attach existing key (only when no active key) -->
        {#if !selected.active_key}
          <div class="detail-section">
            <div class="section-label">Clef SSH</div>
            {#if !showKeyPicker}
              <div class="no-key-state">
                <p class="no-key-hint">Aucune clef active. Vous pouvez générer une nouvelle clef ou en attacher une existante.</p>
                <div class="no-key-actions">
                  <button class="btn btn-primary btn-sm" onclick={() => showKeyPicker = true}
                    disabled={assignableKeys.length === 0}
                    title={assignableKeys.length === 0 ? 'Aucune clef disponible dans la configuration' : ''}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 7a2 2 0 0 1 2 2m4 0a6 6 0 0 1-7.743 5.743L11 17H9v2H7v2H4a1 1 0 0 1-1-1v-2.586a1 1 0 0 1 .293-.707l5.964-5.964A6 6 0 1 1 21 9z"/></svg>
                    Attacher une clef existante
                  </button>
                </div>
              </div>
            {:else}
              <div class="key-picker">
                <div class="key-picker-header">
                  <span class="section-label" style="margin-bottom:0;">Sélectionner une clef</span>
                  <button class="btn btn-ghost btn-sm" aria-label="Fermer" onclick={() => { showKeyPicker = false; assignError = null; }}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                  </button>
                </div>
                {#if assignableKeys.length === 0}
                  <p class="no-key-hint">Aucune clef gérée disponible. Générez d'abord une clef depuis la page Clefs.</p>
                {:else}
                  <div class="key-picker-list">
                    {#each assignableKeys as k}
                      <button
                        class="key-picker-row"
                        disabled={assigningKey}
                        onclick={() => handleAssignKey(k.id)}
                      >
                        <div class="key-picker-info">
                          <span class="key-picker-fp">{k.fingerprint}</span>
                          <span class="key-picker-meta">{k.key_type}{k.yubikey ? ' · YubiKey' : ''} · {k.created_at}</span>
                        </div>
                        {#if assigningKey}
                          <span class="spinner" style="width:12px;height:12px;border-width:2px;"></span>
                        {:else}
                          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
                        {/if}
                      </button>
                    {/each}
                  </div>
                {/if}
                {#if assignError}
                  <p class="assign-error">{assignError}</p>
                {/if}
              </div>
            {/if}
          </div>
        {/if}

        <!-- Public key -->
        <div class="detail-section">
          <div class="section-label">Clef publique</div>
          {#if loadingPublicKey}
            <div style="display:flex; justify-content:center; padding:16px;">
              <span class="spinner"></span>
            </div>
          {:else if publicKey}
            <div class="pubkey-box">
              <textarea class="pubkey-textarea" readonly value={publicKey}></textarea>
              <button class="btn btn-secondary btn-sm copy-btn" onclick={copyPublicKey}>
                {#if copyFeedback}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                  Copié !
                {:else}
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                  Copier
                {/if}
              </button>
            </div>
          {:else}
            <p class="no-pubkey">Aucune clef active</p>
          {/if}
        </div>

        <!-- Actions -->
        <div class="detail-section">
          <div class="section-label">Actions</div>

          <!-- Token warning -->
          {#if selected.health_reasons?.some(r => r.toLowerCase().includes('token'))}
            <div class="token-warning">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0;"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
              <span>Token API non configuré — le déploiement automatique ne fonctionnera pas.</span>
              <button class="btn btn-warning btn-sm" onclick={() => showTokenModal = true}>Configurer</button>
            </div>
          {/if}

          <!-- Token modal (inline) -->
          {#if showTokenModal}
            <div class="token-modal">
              <div class="token-modal-header">
                <span class="token-modal-title">Configurer le token API</span>
                <button class="btn btn-ghost btn-sm" aria-label="Fermer" onclick={() => { showTokenModal = false; tokenSaveError = null; }}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                </button>
              </div>
              {#if selected.token_ref}
                <div class="form-group-inline">
                  <label class="form-label-sm" for="detail-token-ref">Référence</label>
                  <input id="detail-token-ref" class="input input-mono input-sm" type="text" readonly value={selected.token_ref} />
                </div>
              {/if}
              <div class="form-group-inline">
                <label class="form-label-sm" for="detail-token-val">Valeur du token</label>
                <input id="detail-token-val" class="input input-sm" type="password" placeholder="ghp_xxxxxxxxxxxx" bind:value={tokenModalValue} />
              </div>
              {#if tokenSaveError}
                <div class="form-error-sm">{tokenSaveError}</div>
              {/if}
              {#if tokenSaved}
                <div class="form-ok-sm">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                  Token enregistré
                </div>
              {/if}
              <button
                class="btn btn-primary btn-sm"
                style="width:100%;justify-content:center;"
                disabled={tokenSaving || !tokenModalValue.trim()}
                onclick={saveToken}
              >
                {#if tokenSaving}
                  <span class="spinner" style="width:12px;height:12px;border-width:2px;"></span>
                  Enregistrement…
                {:else}
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
                  Enregistrer
                {/if}
              </button>
            </div>
          {/if}

          <div class="actions-grid">
            <button
              class="btn btn-secondary action-btn"
              disabled={!selected.active_key}
              onclick={() => onDeploy({ service: selected, key: selectedKey })}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
              Déployer
            </button>

            <button
              class="btn btn-secondary action-btn"
              title="Renouveler la clef"
              onclick={() => onDeploy({ service: selected, key: null, renew: true })}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M23 4v6h-6M1 20v-6h6"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
              Renouveler
            </button>

            <button
              class="btn btn-secondary action-btn"
              disabled={!selected.active_key}
              onclick={() => onRevoke({ serviceId: selected.id, keyId: selected.active_key })}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
              Révoquer
            </button>

            {#if !confirmingDelete}
              <button
                class="btn btn-danger action-btn"
                onclick={() => confirmingDelete = true}
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><path d="M10 11v6M14 11v6"/><path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/></svg>
                Supprimer
              </button>
            {:else}
              <div class="confirm-delete">
                <span>Confirmer la suppression ?</span>
                <div class="confirm-btns">
                  <button class="btn btn-danger btn-sm" onclick={() => onDelete(selected.id)}>Oui, supprimer</button>
                  <button class="btn btn-ghost btn-sm" onclick={() => confirmingDelete = false}>Annuler</button>
                </div>
              </div>
            {/if}
          </div>
        </div>

      </div>
    </div>
    {/key}
  {/if}
</div>

<style>
  .services-layout {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  /* ---- List pane ---- */
  .list-pane {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    min-width: 0;
  }

  .list-header {
    display: grid;
    grid-template-columns: 28px 36px 1fr 72px;
    padding: 0 16px;
    height: 32px;
    align-items: center;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .col-header {
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-disabled);
  }
  .col-header.col-rotation { text-align: right; }

  .list-body {
    flex: 1;
    overflow-y: auto;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .service-row {
    display: grid;
    grid-template-columns: 28px 36px 1fr 72px;
    align-items: center;
    padding: 8px 8px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition:
      background var(--transition-normal),
      border-color var(--transition-normal),
      box-shadow var(--transition-normal),
      transform var(--transition-fast);
    position: relative;
    outline: none;
  }
  .service-row:hover {
    background: var(--bg-elevated);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0,0,0,0.2);
  }
  .service-row.selected {
    background: var(--accent-glow);
    border-color: var(--border-active);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .col-status  { display: flex; align-items: center; justify-content: center; }
  .col-type    { display: flex; align-items: center; justify-content: center; }
  .col-name    { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .col-rotation { display: flex; align-items: center; justify-content: flex-end; }

  .type-icon   { color: var(--text-secondary); display: flex; align-items: center; }
  .service-row.selected .type-icon { color: var(--accent); }

  .service-name {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-primary);
    display: block;
    max-width: 100%;
  }
  .service-host {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-mono);
    display: block;
    max-width: 100%;
    opacity: 0.85;
  }

  .list-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  /* ---- Detail panel ---- */
  .detail-panel {
    width: 340px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--bg-panel);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slide-in-right 280ms cubic-bezier(0.22, 1, 0.36, 1) both;
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 16px 8px;
    flex-shrink: 0;
  }
  .detail-header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .detail-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .detail-host {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-mono);
    padding: 0 16px 12px;
    flex-shrink: 0;
    opacity: 0.9;
  }

  .detail-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  /* Health banner */
  .health-banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    margin-bottom: 16px;
    font-size: 0.8rem;
  }
  .health-ok       { background: var(--ok-glow);       color: var(--ok);       border: 1px solid rgba(61,170,106,0.2); }
  .health-warning  { background: var(--warning-glow);  color: var(--warning);  border: 1px solid rgba(212,150,42,0.2); }
  .health-critical { background: var(--critical-glow); color: var(--critical); border: 1px solid rgba(201,64,64,0.2); }
  .health-info     { background: var(--accent-glow);   color: var(--accent);   border: 1px solid rgba(74,128,212,0.2); }

  .health-icon { display: flex; align-items: center; flex-shrink: 0; margin-top: 1px; }
  .health-content { flex: 1; min-width: 0; }
  .health-reasons {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .health-reasons li::before {
    content: '• ';
    opacity: 0.6;
  }

  /* Sections */
  .detail-section {
    margin-bottom: 20px;
  }
  .info-box {
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    overflow: hidden;
  }
  .info-box .info-row {
    padding: 8px 12px;
  }
  .info-box .info-row .label { font-size: 0.78rem; }
  .info-box .info-row .value { font-size: 0.78rem; }

  /* Fingerprint */
  .fingerprint-box {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .fingerprint-code {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-mono);
    word-break: break-all;
    line-height: 1.6;
  }

  /* Public key */
  .pubkey-box {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .pubkey-textarea {
    width: 100%;
    height: 80px;
    resize: none;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-mono);
    font-family: var(--font-mono);
    font-size: 0.68rem;
    padding: 8px 10px;
    line-height: 1.5;
    outline: none;
    overflow-y: auto;
  }
  .copy-btn { align-self: flex-end; }
  .no-pubkey {
    color: var(--text-disabled);
    font-size: 0.8rem;
    font-style: italic;
  }

  /* Actions */
  .actions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .action-btn {
    justify-content: center;
    font-size: 0.78rem;
    padding: 8px 10px;
  }

  .confirm-delete {
    grid-column: 1 / -1;
    background: var(--critical-glow);
    border: 1px solid rgba(201,64,64,0.3);
    border-radius: var(--radius-md);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 0.8rem;
    color: var(--critical);
    animation: scale-in 150ms ease;
  }
  .confirm-btns {
    display: flex;
    gap: 8px;
  }

  /* Token warning */
  .token-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    background: var(--warning-glow);
    border: 1px solid rgba(212,150,42,0.3);
    border-radius: var(--radius-md);
    color: var(--warning);
    font-size: 0.76rem;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .token-warning span {
    flex: 1;
    min-width: 0;
    line-height: 1.4;
  }

  /* btn-warning */
  :global(.btn-warning) {
    background: rgba(212,150,42,0.15);
    border: 1px solid rgba(212,150,42,0.35);
    color: var(--warning);
  }
  :global(.btn-warning:hover) {
    background: rgba(212,150,42,0.25);
  }

  /* Token inline modal */
  .token-modal {
    background: var(--bg-elevated);
    border: 1px solid var(--border-active);
    border-radius: var(--radius-md);
    padding: 12px;
    margin-bottom: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    animation: scale-in 150ms ease;
  }
  .token-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }
  .token-modal-title {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .form-group-inline {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .form-label-sm {
    font-size: 0.72rem;
    color: var(--text-secondary);
    font-weight: 500;
  }
  .input-sm {
    padding: 6px 8px;
    font-size: 0.75rem;
  }
  .form-error-sm {
    font-size: 0.72rem;
    color: var(--critical);
    padding: 5px 8px;
    background: var(--critical-glow);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(201,64,64,0.2);
  }
  .form-ok-sm {
    font-size: 0.72rem;
    color: var(--ok);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  /* ---- No-key state + key picker ---- */
  .no-key-state {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    border: 1px dashed var(--border);
  }
  .no-key-hint {
    font-size: 0.75rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }
  .no-key-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .key-picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .key-picker-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .key-picker-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 180px;
    overflow-y: auto;
  }
  .key-picker-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    cursor: pointer;
    font-family: var(--font-ui);
    color: var(--text-primary);
    text-align: left;
    transition: background var(--t-fast), border-color var(--t-fast);
  }
  .key-picker-row:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--accent);
  }
  .key-picker-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .key-picker-fp {
    font-family: var(--font-mono);
    font-size: 0.68rem;
    color: var(--text-mono);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .key-picker-meta {
    font-size: 0.7rem;
    color: var(--text-secondary);
  }
  .assign-error {
    font-size: 0.75rem;
    color: var(--critical);
    padding: 6px 8px;
    background: var(--critical-glow);
    border-radius: var(--radius-sm);
  }
</style>
