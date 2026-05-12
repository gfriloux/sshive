<script>
  const { page, onNavigate, criticalCount = 0 } = $props();

  const navItems = [
    {
      id: 'services',
      label: 'Services',
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/></svg>`,
    },
    {
      id: 'keys',
      label: 'Clefs',
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="5.5"/><path d="M21 2l-9.6 9.6M15.5 7.5l2 2"/></svg>`,
    },
    {
      id: 'health',
      label: 'Santé',
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>`,
    },
    {
      id: 'settings',
      label: 'Paramètres',
      icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`,
    },
  ];
</script>

<nav class="sidebar">
  <ul class="nav-list">
    {#each navItems as item}
      <li>
        <button
          class="nav-item"
          class:active={page === item.id}
          onclick={() => onNavigate(item.id)}
        >
          <span class="nav-icon">{@html item.icon}</span>
          <span class="nav-label">{item.label}</span>
          {#if item.id === 'health' && criticalCount > 0}
            <span class="critical-badge">
              <span class="status-dot critical"></span>
            </span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>

  <div class="sidebar-footer">
    <div class="version">v0.4.0</div>
  </div>
</nav>

<style>
  .sidebar {
    width: 200px;
    flex-shrink: 0;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 12px 0;
    overflow: hidden;
  }

  .nav-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 8px;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 10px;
    border-radius: var(--radius-md);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-ui);
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    position: relative;
    transition:
      background var(--transition-normal),
      color var(--transition-normal),
      box-shadow var(--transition-normal);
    outline: none;
  }

  .nav-item:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-glow);
    color: var(--accent);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .nav-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    opacity: 0.85;
  }
  .nav-item.active .nav-icon {
    opacity: 1;
  }

  .nav-label {
    flex: 1;
  }

  .critical-badge {
    display: flex;
    align-items: center;
  }

  .sidebar-footer {
    padding: 12px 16px 4px;
    border-top: 1px solid var(--border);
    margin-top: 8px;
  }

  .version {
    font-size: 0.7rem;
    color: var(--text-disabled);
    font-weight: 500;
    letter-spacing: 0.04em;
  }
</style>
