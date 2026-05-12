<script>
  const { page, healthCounts } = $props();

  const pageTitles = {
    services: 'Services SSH',
    keys:     'Clefs SSH',
    health:   'Santé',
    settings: 'Paramètres',
  };
</script>

<header class="topbar">
  <!-- Logo -->
  <div class="logo">
    <svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
      <!-- Key icon -->
      <circle cx="10" cy="14" r="6" stroke="#4A80D4" stroke-width="1.8" fill="none"/>
      <circle cx="10" cy="14" r="2.5" fill="#4A80D4" opacity="0.5"/>
      <line x1="15.5" y1="14" x2="26" y2="14" stroke="#4A80D4" stroke-width="1.8" stroke-linecap="round"/>
      <line x1="22" y1="14" x2="22" y2="17" stroke="#4A80D4" stroke-width="1.8" stroke-linecap="round"/>
      <line x1="24.5" y1="14" x2="24.5" y2="16.5" stroke="#4A80D4" stroke-width="1.8" stroke-linecap="round"/>
    </svg>
    <span class="logo-text">SSHive</span>
  </div>

  <!-- Page title -->
  <div class="page-title">
    {pageTitles[page] ?? page}
  </div>

  <!-- Spacer -->
  <div class="spacer"></div>

  <!-- Health chips -->
  <div class="health-chips">
    {#if healthCounts.ok > 0}
      <div class="chip ok">
        <span class="status-dot ok"></span>
        <span>{healthCounts.ok}</span>
      </div>
    {/if}
    {#if healthCounts.warning > 0}
      <div class="chip warning">
        <span class="status-dot warning"></span>
        <span>{healthCounts.warning}</span>
      </div>
    {/if}
    {#if healthCounts.critical > 0}
      <div class="chip critical">
        <span class="status-dot critical"></span>
        <span>{healthCounts.critical}</span>
      </div>
    {/if}
    {#if healthCounts.ok === 0 && healthCounts.warning === 0 && healthCounts.critical === 0}
      <div class="chip neutral">
        <span>Aucun service</span>
      </div>
    {/if}
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    height: 56px;
    padding: 0 20px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    gap: 16px;
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }
  .logo-text {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: 0.02em;
  }

  .page-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
    padding-left: 16px;
    border-left: 1px solid var(--border);
  }

  .spacer {
    flex: 1;
  }

  .health-chips {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 99px;
    font-size: 0.75rem;
    font-weight: 600;
    border: 1px solid transparent;
    transition: box-shadow var(--transition-normal);
  }
  .chip.ok {
    background: var(--ok-glow);
    color: var(--ok);
    border-color: rgba(61,170,106,0.25);
  }
  .chip.warning {
    background: var(--warning-glow);
    color: var(--warning);
    border-color: rgba(212,150,42,0.25);
  }
  .chip.critical {
    background: var(--critical-glow);
    color: var(--critical);
    border-color: rgba(201,64,64,0.25);
  }
  .chip.neutral {
    background: rgba(255,255,255,0.04);
    color: var(--text-disabled);
    border-color: var(--border);
  }
</style>
