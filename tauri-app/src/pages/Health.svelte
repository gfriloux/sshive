<script>
  const { services, counts } = $props();

  // Sort: critical first, then warning, then ok
  let sorted = $derived([...services].sort((a, b) => {
    const order = { critical: 0, warning: 1, info: 2, ok: 3 };
    return (order[a.health_level] ?? 4) - (order[b.health_level] ?? 4);
  }));

  let allOk = $derived(counts.critical === 0 && counts.warning === 0);

  function levelLabel(level) {
    switch (level) {
      case 'ok':       return 'OK';
      case 'warning':  return 'Attention';
      case 'critical': return 'Critique';
      default:         return 'Info';
    }
  }

  function typeLabel(serviceType) {
    switch (serviceType) {
      case 'github':             return 'GitHub';
      case 'gitlab':             return 'GitLab';
      case 'gitlab-self-hosted': return 'GitLab SH';
      default:                   return 'SSH';
    }
  }

  function healthIcon(level) {
    switch (level) {
      case 'ok':
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>`;
      case 'warning':
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`;
      case 'critical':
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>`;
      default:
        return `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`;
    }
  }
</script>

<div class="health-page">
  <!-- Summary tiles -->
  <div class="summary-bar">
    <div class="tile ok">
      <div class="tile-count">{counts.ok}</div>
      <div class="tile-label">OK</div>
    </div>
    <div class="tile warning">
      <div class="tile-count">{counts.warning}</div>
      <div class="tile-label">Attention</div>
    </div>
    <div class="tile critical">
      <div class="tile-count">{counts.critical}</div>
      <div class="tile-label">Critique</div>
    </div>
  </div>

  <!-- All OK state -->
  {#if allOk && services.length > 0}
    <div class="all-ok">
      <div class="all-ok-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      </div>
      <h3>Tout est sain</h3>
      <p>Tous vos services SSH sont en bonne santé.</p>
    </div>
  {:else if services.length === 0}
    <div class="empty-state">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
      <p>Aucun service à surveiller</p>
    </div>
  {:else}
    <div class="health-list stagger">
      {#each sorted as service (service.id)}
        <div class="health-item level-{service.health_level}">
          <div class="health-item-icon">{@html healthIcon(service.health_level)}</div>
          <div class="health-item-body">
            <div class="health-item-head">
              <span class="health-item-name">{service.name}</span>
              <div class="health-item-meta">
                <span class="badge neutral" style="font-size:0.65rem;">{typeLabel(service.service_type)}</span>
                <span class="badge {service.health_level}" style="font-size:0.65rem;">{levelLabel(service.health_level)}</span>
              </div>
            </div>
            {#if service.health_reasons?.length > 0}
              <ul class="health-item-reasons">
                {#each service.health_reasons as reason}
                  <li>{reason}</li>
                {/each}
              </ul>
            {:else}
              <p class="health-item-ok-text">Aucun problème détecté</p>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .health-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Summary bar */
  .summary-bar {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    padding: 20px 24px 16px;
    flex-shrink: 0;
    border-bottom: 1px solid var(--border);
  }

  .tile {
    padding: 16px 20px;
    border-radius: var(--radius-lg);
    border: 1px solid transparent;
    display: flex;
    flex-direction: column;
    gap: 4px;
    transition: box-shadow var(--transition-normal), transform var(--transition-fast);
  }
  .tile:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(0,0,0,0.25);
  }

  .tile.ok {
    background: var(--ok-glow);
    border-color: rgba(61,170,106,0.25);
  }
  .tile.warning {
    background: var(--warning-glow);
    border-color: rgba(212,150,42,0.25);
  }
  .tile.critical {
    background: var(--critical-glow);
    border-color: rgba(201,64,64,0.25);
  }

  .tile-count {
    font-size: 2rem;
    font-weight: 700;
    line-height: 1;
  }
  .tile.ok       .tile-count { color: var(--ok); }
  .tile.warning  .tile-count { color: var(--warning); }
  .tile.critical .tile-count { color: var(--critical); }

  .tile-label {
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    opacity: 0.7;
  }
  .tile.ok       .tile-label { color: var(--ok); }
  .tile.warning  .tile-label { color: var(--warning); }
  .tile.critical .tile-label { color: var(--critical); }

  /* All OK state */
  .all-ok {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--ok);
    text-align: center;
    padding: 40px;
    animation: fade-in 400ms ease;
  }
  .all-ok-icon {
    opacity: 0.6;
  }
  .all-ok h3 {
    font-size: 1.25rem;
    font-weight: 600;
  }
  .all-ok p {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  /* Health list */
  .health-list {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .health-item {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px 16px;
    border-radius: var(--radius-lg);
    border: 1px solid transparent;
    background: var(--bg-panel);
    transition: border-color var(--transition-normal), box-shadow var(--transition-normal);
  }
  .health-item:hover {
    box-shadow: 0 2px 12px rgba(0,0,0,0.2);
  }

  .level-ok       { border-color: rgba(61,170,106,0.15); }
  .level-warning  { border-color: rgba(212,150,42,0.2);  }
  .level-critical { border-color: rgba(201,64,64,0.25);  }

  .health-item-icon {
    margin-top: 1px;
    flex-shrink: 0;
  }
  .level-ok       .health-item-icon { color: var(--ok); }
  .level-warning  .health-item-icon { color: var(--warning); }
  .level-critical .health-item-icon { color: var(--critical); }
  .level-info     .health-item-icon { color: var(--accent); }

  .health-item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .health-item-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-wrap: wrap;
  }
  .health-item-name {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
  }
  .health-item-meta {
    display: flex;
    gap: 6px;
  }

  .health-item-reasons {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .health-item-reasons li {
    font-size: 0.8rem;
    color: var(--text-secondary);
    padding-left: 12px;
    position: relative;
  }
  .health-item-reasons li::before {
    content: '•';
    position: absolute;
    left: 0;
    opacity: 0.5;
  }
  .level-warning .health-item-reasons li  { color: var(--warning); opacity: 0.9; }
  .level-critical .health-item-reasons li { color: var(--critical); opacity: 0.9; }

  .health-item-ok-text {
    font-size: 0.8rem;
    color: var(--text-disabled);
    font-style: italic;
  }
</style>
