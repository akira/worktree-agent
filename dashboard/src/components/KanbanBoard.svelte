<script>
  import TaskCard from './TaskCard.svelte';

  let { agents, onSelect } = $props();

  const columns = [
    { id: 'running', title: 'Running', status: 'running' },
    { id: 'completed', title: 'Completed', status: 'completed' },
    { id: 'failed', title: 'Failed', status: 'failed' },
    { id: 'merged', title: 'Merged', status: 'merged' },
  ];

  function getAgentsByStatus(status) {
    return agents.filter(a => a.status === status);
  }

  function getColumnClass(status) {
    return `column-${status}`;
  }
</script>

<div class="kanban-board">
  {#each columns as column}
    {@const columnAgents = getAgentsByStatus(column.status)}
    <div class="kanban-column {getColumnClass(column.status)}">
      <div class="column-header">
        <h2>{column.title}</h2>
        <span class="count">{columnAgents.length}</span>
      </div>
      <div class="column-content">
        {#if columnAgents.length === 0}
          <div class="empty-state">No tasks</div>
        {:else}
          {#each columnAgents as agent (agent.id)}
            <TaskCard {agent} {onSelect} />
          {/each}
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .kanban-board {
    display: flex;
    gap: 1rem;
    min-height: calc(100vh - 150px);
    padding-bottom: 1rem;
  }

  .kanban-column {
    flex: 1;
    min-width: 280px;
    max-width: 400px;
    background-color: var(--bg-secondary);
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
  }

  .column-header {
    padding: 1rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .column-header h2 {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .column-running .column-header h2 {
    color: var(--accent);
  }

  .column-completed .column-header h2 {
    color: var(--success);
  }

  .column-failed .column-header h2 {
    color: var(--error);
  }

  .column-merged .column-header h2 {
    color: var(--merged);
  }

  .count {
    background-color: var(--bg-card);
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .column-content {
    flex: 1;
    padding: 0.75rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .empty-state {
    text-align: center;
    color: var(--text-muted);
    padding: 2rem 1rem;
    font-size: 0.875rem;
  }
</style>
