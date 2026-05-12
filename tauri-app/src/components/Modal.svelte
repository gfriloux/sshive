<script>
  import { onMount } from 'svelte';

  const { onClose, maxWidth = '480px', title = null, children } = $props();

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') {
      onClose();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="modal-overlay" role="dialog" aria-modal="true" onclick={handleOverlayClick}>
  <div class="modal-box" style="max-width: {maxWidth};">
    {#if title}
      <div class="modal-header">
        <h3 class="modal-title">{title}</h3>
        <button class="modal-close-btn" aria-label="Fermer" onclick={onClose}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    {/if}
    <div class="modal-content">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 500;
    background: rgba(14, 21, 32, 0.85);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    animation: fade-in 180ms ease;
  }

  .modal-box {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    width: 100%;
    box-shadow: var(--shadow-modal);
    animation: scale-in 200ms cubic-bezier(0.22, 1, 0.36, 1);
    max-height: calc(100vh - 48px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 0;
    flex-shrink: 0;
  }

  .modal-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .modal-close-btn {
    background: transparent;
    border: none;
    color: var(--text-disabled);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: color var(--transition-fast), background var(--transition-fast);
  }
  .modal-close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-elevated);
  }

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px 24px;
    min-height: 0;
  }
</style>
