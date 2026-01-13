<script>
  import { onMount } from 'svelte';
  import KanbanBoard from './components/KanbanBoard.svelte';
  import TaskModal from './components/TaskModal.svelte';

  let agents = $state([]);
  let selectedAgent = $state(null);
  let loading = $state(true);
  let error = $state(null);

  async function fetchAgents() {
    try {
      const response = await fetch('/api/agents');
      if (!response.ok) {
        throw new Error('Failed to fetch agents');
      }
      agents = await response.json();
      error = null;
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  function selectAgent(agent) {
    selectedAgent = agent;
  }

  function closeModal() {
    selectedAgent = null;
  }

  async function refreshAfterAction() {
    await fetchAgents();
    if (selectedAgent) {
      const updated = agents.find(a => a.id === selectedAgent.id);
      if (updated) {
        selectedAgent = updated;
      } else {
        selectedAgent = null;
      }
    }
  }

  onMount(() => {
    fetchAgents();
    // Poll for updates every 5 seconds
    const interval = setInterval(fetchAgents, 5000);
    return () => clearInterval(interval);
  });
</script>

<div class="app">
  <header>
    <h1>WTA Dashboard</h1>
    <p class="subtitle">Worktree Agent Task Manager</p>
  </header>

  <main>
    {#if loading}
      <div class="loading">Loading agents...</div>
    {:else if error}
      <div class="error-message">
        <p>Error: {error}</p>
        <button class="btn btn-primary" onclick={fetchAgents}>Retry</button>
      </div>
    {:else}
      <KanbanBoard {agents} onSelect={selectAgent} />
    {/if}
  </main>

  {#if selectedAgent}
    <TaskModal
      agent={selectedAgent}
      onClose={closeModal}
      onRefresh={refreshAfterAction}
    />
  {/if}
</div>

<style>
  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    padding: 1.5rem 2rem;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
  }

  header h1 {
    font-size: 1.5rem;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  main {
    flex: 1;
    padding: 1.5rem;
    overflow-x: auto;
  }

  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: var(--text-secondary);
  }

  .error-message {
    text-align: center;
    padding: 2rem;
    color: var(--error);
  }

  .error-message button {
    margin-top: 1rem;
  }
</style>
