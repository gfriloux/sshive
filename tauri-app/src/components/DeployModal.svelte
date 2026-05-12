<script>
  import { invoke } from '@tauri-apps/api/core';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import Modal from './Modal.svelte';

  const { service, keyView, onClose } = $props();

  let deploying      = $state(false);
  let deployMessage  = $state(null);
  let deployError    = $state(null);
  let copied         = $state(false);
  let copyTarget     = $state(''); // which snippet was copied

  // Auto-trigger for automatic mode
  async function autoDeploy() {
    deploying = true;
    deployError = null;
    deployMessage = null;
    try {
      const msg = await invoke('deploy_key', {
        serviceId: service.id,
        keyId: keyView?.id ?? service.active_key,
      });
      deployMessage = msg;
    } catch (e) {
      deployError = String(e);
    } finally {
      deploying = false;
    }
  }

  // Copy to clipboard
  async function copySnippet(text, id) {
    await writeText(text);
    copyTarget = id;
    copied = true;
    setTimeout(() => { copied = false; copyTarget = ''; }, 2500);
  }

  // Build ssh-copy-id command
  let sshCopyIdCmd = $derived.by(() => {
    const k = keyView;
    const pubPath = k?.public_path ?? null;
    const user = service.params?.user ?? 'user';
    const portArg = service.params?.port ? ` -p ${service.params.port}` : '';
    const host = service.params?.url ?? '<hostname>';
    if (pubPath) {
      return `ssh-copy-id -i ${pubPath}${portArg} ${user}@${host}`;
    }
    return null;
  });

  // Whether public key path is available
  let hasPubPath = $derived(!!keyView?.public_path);

  // NixOS snippet — use real pubkey if available
  let nixosSnippet = $derived.by(() => {
    const username = service.params?.user ?? 'deploy';
    const pubKey = keyView?.public_key ?? '<coller-la-clef-publique-ici>';
    return `# Dans votre configuration.nix
users.users.${username}.openssh.authorizedKeys.keys = [
  "${pubKey}"
];`;
  });

  // Ansible snippet — use real pubkey path if available
  let ansibleSnippet = $derived.by(() => {
    const user = service.params?.user ?? 'deploy';
    const pubkeyPath = keyView?.public_path ?? '~/.ssh/id_ed25519.pub';
    return `- name: Deploy SSH key for ${service.name}
  authorized_key:
    user: "${user}"
    key: "{{ lookup('file', '${pubkeyPath}') }}"
    state: present`;
  });
</script>

<Modal {onClose} maxWidth="500px" title={null}>
  {#snippet children()}
  <div class="deploy-header">
    <h3 class="modal-title-inner">Déployer la clef</h3>
    <div class="deploy-service-tag">
      <span class="status-dot {service.health_level}"></span>
      <span>{service.name}</span>
    </div>
  </div>

  <div class="deploy-mode-badge">
    Mode :
    {#if service.deploy_mode === 'automatic'}
      <span class="badge ok">Automatique</span>
    {:else if service.deploy_mode === 'guided'}
      <span class="badge accent">Guidé</span>
    {:else}
      <span class="badge neutral">CM externe</span>
    {/if}
  </div>

  <!-- Automatic mode -->
  {#if service.deploy_mode === 'automatic'}
    <div class="auto-deploy">
      {#if deploying}
        <div class="auto-progress">
          <span class="spinner lg"></span>
          <p>Déploiement en cours…</p>
          <p class="auto-hint">Connexion à l'API {service.service_type === 'github' ? 'GitHub' : 'GitLab'}</p>
        </div>
      {:else if deployMessage}
        <div class="auto-success">
          <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          <h4>Déploiement réussi</h4>
          <p>{deployMessage}</p>
        </div>
      {:else if deployError}
        <div class="auto-error">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          <p>{deployError}</p>
          <button class="btn btn-secondary btn-sm" onclick={autoDeploy}>Réessayer</button>
        </div>
      {:else}
        <div class="auto-ready">
          <p class="deploy-desc">La clef sera déployée automatiquement via l'API du service.</p>
          {#if keyView}
            <div class="fp-preview">
              <code>{keyView.fingerprint}</code>
            </div>
          {/if}
          <button class="btn btn-primary" style="width:100%;justify-content:center;" onclick={autoDeploy}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
            Lancer le déploiement
          </button>
        </div>
      {/if}
    </div>

  <!-- Guided mode -->
  {:else if service.deploy_mode === 'guided'}
    <div class="guided-deploy">
      {#if hasPubPath && sshCopyIdCmd}
        <p class="deploy-desc">Exécutez la commande suivante sur votre terminal pour déployer la clef :</p>
        <div class="snippet-box">
          <pre class="snippet-code">{sshCopyIdCmd}</pre>
          <button
            class="snippet-copy-btn"
            onclick={() => copySnippet(sshCopyIdCmd ?? '', 'ssh-copy-id')}
          >
            {#if copied && copyTarget === 'ssh-copy-id'}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              Copié
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              Copier
            {/if}
          </button>
        </div>
        <p class="deploy-hint">Assurez-vous d'avoir les droits SSH sur la cible et que ssh-copy-id est disponible.</p>
      {:else}
        <p class="deploy-desc">Le chemin de la clef publique n'est pas disponible. Copiez le contenu ci-dessous dans le fichier <code>~/.ssh/authorized_keys</code> de la machine cible :</p>
        {#if keyView?.public_key}
          <div class="snippet-box">
            <pre class="snippet-code">{keyView.public_key}</pre>
            <button
              class="snippet-copy-btn"
              onclick={() => copySnippet(keyView.public_key, 'pubkey-content')}
            >
              {#if copied && copyTarget === 'pubkey-content'}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
                Copié
              {:else}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
                Copier
              {/if}
            </button>
          </div>
        {:else}
          <p class="deploy-hint">Aucune clef publique disponible pour ce service.</p>
        {/if}
      {/if}
    </div>

  <!-- External CM mode -->
  {:else}
    <div class="external-deploy">
      <p class="deploy-desc">Ajoutez la clef publique dans votre gestionnaire de configuration :</p>

      <!-- NixOS -->
      <div class="cm-section">
        <div class="cm-section-header">
          <span class="cm-title">NixOS</span>
          <button
            class="btn btn-ghost btn-sm"
            onclick={() => copySnippet(nixosSnippet, 'nixos')}
          >
            {#if copied && copyTarget === 'nixos'}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              Copié
            {:else}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              Copier
            {/if}
          </button>
        </div>
        <div class="snippet-box">
          <pre class="snippet-code">{nixosSnippet}</pre>
        </div>
      </div>

      <!-- Ansible -->
      <div class="cm-section">
        <div class="cm-section-header">
          <span class="cm-title">Ansible</span>
          <button
            class="btn btn-ghost btn-sm"
            onclick={() => copySnippet(ansibleSnippet, 'ansible')}
          >
            {#if copied && copyTarget === 'ansible'}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
              Copié
            {:else}
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
              Copier
            {/if}
          </button>
        </div>
        <div class="snippet-box">
          <pre class="snippet-code">{ansibleSnippet}</pre>
        </div>
      </div>
    </div>
  {/if}

  <!-- Footer -->
  <div class="deploy-footer">
    <button class="btn btn-ghost" onclick={onClose}>
      {deployMessage ? 'Fermer' : 'Annuler'}
    </button>
  </div>
  {/snippet}
</Modal>

<style>
  .deploy-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  .modal-title-inner {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .deploy-service-tag {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .deploy-mode-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.78rem;
    color: var(--text-secondary);
    margin-bottom: 18px;
  }

  .deploy-desc {
    font-size: 0.82rem;
    color: var(--text-secondary);
    line-height: 1.6;
    margin-bottom: 14px;
  }
  .deploy-hint {
    font-size: 0.72rem;
    color: var(--text-disabled);
    margin-top: 10px;
    line-height: 1.5;
  }

  /* Automatic mode */
  .auto-deploy { min-height: 120px; }

  .auto-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 24px;
    text-align: center;
    animation: fade-in 200ms ease;
  }
  .auto-progress p { color: var(--text-secondary); font-size: 0.85rem; }
  .auto-hint { font-size: 0.75rem !important; color: var(--text-disabled) !important; }

  .auto-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 16px;
    text-align: center;
    color: var(--ok);
    animation: scale-in 200ms ease;
  }
  .auto-success h4 { font-size: 1rem; font-weight: 600; }
  .auto-success p  { font-size: 0.8rem; color: var(--text-secondary); }

  .auto-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 16px;
    text-align: center;
    color: var(--critical);
  }
  .auto-error p { font-size: 0.8rem; }

  .auto-ready {
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: fade-in 200ms ease;
  }
  .fp-preview {
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 12px;
  }
  .fp-preview code {
    font-family: var(--font-mono);
    font-size: 0.7rem;
    color: var(--text-mono);
    display: block;
    word-break: break-all;
  }

  /* Snippet box */
  .snippet-box {
    position: relative;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .snippet-code {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-mono);
    padding: 12px 14px;
    padding-right: 80px;
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.7;
    margin: 0;
  }
  .snippet-copy-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 0.72rem;
    padding: 4px 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: color var(--transition-fast), background var(--transition-fast);
  }
  .snippet-copy-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  /* External CM */
  .external-deploy {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .cm-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cm-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .cm-title {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  /* Guided deploy */
  .guided-deploy {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  /* Footer */
  .deploy-footer {
    display: flex;
    justify-content: flex-end;
    margin-top: 16px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
</style>
