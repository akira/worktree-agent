<script>
  let { agent, onSelect } = $props();

  function formatDate(dateStr) {
    const date = new Date(dateStr);
    return date.toLocaleString();
  }

  function getStatusColor(status) {
    const colors = {
      running: 'var(--accent)',
      completed: 'var(--success)',
      failed: 'var(--error)',
      merged: 'var(--merged)',
    };
    return colors[status] || 'var(--text-secondary)';
  }

  function handleClick() {
    onSelect(agent);
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect(agent);
    }
  }
</script>

<div
  class="task-card"
  role="button"
  tabindex="0"
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <div class="card-header">
    <span class="agent-id">#{agent.id}</span>
    <span class="branch">{agent.branch}</span>
  </div>

  <p class="task-description">{agent.task}</p>

  <div class="card-footer">
    <span class="provider">{agent.provider}</span>
    <span class="time">{formatDate(agent.launched_at)}</span>
  </div>

  <div class="status-indicator" style="background-color: {getStatusColor(agent.status)}"></div>
</div>

<style>
  .task-card {
    background-color: var(--bg-card);
    border-radius: 0.375rem;
    padding: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
    position: relative;
    overflow: hidden;
    border: 1px solid transparent;
  }

  .task-card:hover {
    background-color: #3d4f66;
    border-color: var(--border);
    transform: translateY(-1px);
  }

  .task-card:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .agent-id {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .branch {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: 'SF Mono', monospace;
    background-color: var(--bg-secondary);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .task-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .provider {
    text-transform: capitalize;
  }

  .status-indicator {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
  }
</style>
