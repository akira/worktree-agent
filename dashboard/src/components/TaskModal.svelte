<script>
  import DiffViewer from './DiffViewer.svelte';

  let { agent, onClose, onRefresh } = $props();

  let activeTab = $state('diff');
  let diff = $state(null);
  let loadingDiff = $state(false);
  let diffError = $state(null);

  let mergeStrategy = $state('merge');
  let merging = $state(false);
  let mergeError = $state(null);
  let mergeSuccess = $state(null);

  let creatingPr = $state(false);
  let prError = $state(null);
  let prUrl = $state(null);

  let output = $state('');
  let loadingOutput = $state(false);

  async function fetchDiff() {
    loadingDiff = true;
    diffError = null;
    try {
      const response = await fetch(`/api/agents/${agent.id}/diff`);
      if (!response.ok) {
        const err = await response.json();
        throw new Error(err.error || 'Failed to fetch diff');
      }
      diff = await response.json();
    } catch (e) {
      diffError = e.message;
    } finally {
      loadingDiff = false;
    }
  }

  async function fetchOutput() {
    loadingOutput = true;
    try {
      const response = await fetch(`/api/agents/${agent.id}/output?lines=200`);
      if (!response.ok) {
        throw new Error('Failed to fetch output');
      }
      const data = await response.json();
      output = data.output;
    } catch (e) {
      output = `Error: ${e.message}`;
    } finally {
      loadingOutput = false;
    }
  }

  async function handleMerge() {
    merging = true;
    mergeError = null;
    mergeSuccess = null;
    try {
      const response = await fetch(`/api/agents/${agent.id}/merge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ strategy: mergeStrategy }),
      });
      const result = await response.json();
      if (!response.ok) {
        throw new Error(result.error || 'Merge failed');
      }
      if (result.success) {
        mergeSuccess = result.message;
        onRefresh();
      } else {
        mergeError = result.message;
      }
    } catch (e) {
      mergeError = e.message;
    } finally {
      merging = false;
    }
  }

  async function handleCreatePr() {
    creatingPr = true;
    prError = null;
    try {
      const response = await fetch(`/api/agents/${agent.id}/pr`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });
      const result = await response.json();
      if (!response.ok) {
        throw new Error(result.error || 'PR creation failed');
      }
      prUrl = result.url;
    } catch (e) {
      prError = e.message;
    } finally {
      creatingPr = false;
    }
  }

  async function handleRemove() {
    if (!confirm('Are you sure you want to remove this agent?')) return;

    try {
      const response = await fetch(`/api/agents/${agent.id}?force=true`, {
        method: 'DELETE',
      });
      if (!response.ok) {
        const err = await response.json();
        throw new Error(err.error || 'Failed to remove agent');
      }
      onRefresh();
      onClose();
    } catch (e) {
      alert(`Error: ${e.message}`);
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') {
      onClose();
    }
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function setTab(tab) {
    activeTab = tab;
    if (tab === 'diff' && !diff && !loadingDiff) {
      fetchDiff();
    } else if (tab === 'output' && !output && !loadingOutput) {
      fetchOutput();
    }
  }

  // Fetch diff on mount
  $effect(() => {
    if (agent) {
      fetchDiff();
    }
  });

  function formatDate(dateStr) {
    if (!dateStr) return 'N/A';
    return new Date(dateStr).toLocaleString();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="modal-backdrop" onclick={handleBackdropClick} role="presentation">
  <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-header">
      <div class="header-left">
        <h2 id="modal-title">Task #{agent.id}</h2>
        <span class="status status-{agent.status}">{agent.status}</span>
      </div>
      <button class="close-btn" onclick={onClose} aria-label="Close modal">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="task-info">
      <p class="task-description">{agent.task}</p>
      <div class="meta">
        <span><strong>Branch:</strong> {agent.branch}</span>
        <span><strong>Base:</strong> {agent.base_branch}</span>
        <span><strong>Provider:</strong> {agent.provider}</span>
        <span><strong>Launched:</strong> {formatDate(agent.launched_at)}</span>
        {#if agent.completed_at}
          <span><strong>Completed:</strong> {formatDate(agent.completed_at)}</span>
        {/if}
      </div>
    </div>

    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === 'diff'}
        onclick={() => setTab('diff')}
      >
        Diff
      </button>
      <button
        class="tab"
        class:active={activeTab === 'actions'}
        onclick={() => setTab('actions')}
      >
        Actions
      </button>
      <button
        class="tab"
        class:active={activeTab === 'output'}
        onclick={() => setTab('output')}
      >
        Output
      </button>
    </div>

    <div class="tab-content">
      {#if activeTab === 'diff'}
        {#if loadingDiff}
          <div class="loading">Loading diff...</div>
        {:else if diffError}
          <div class="error">Error: {diffError}</div>
        {:else if diff}
          <DiffViewer {diff} />
        {:else}
          <div class="empty">No diff available</div>
        {/if}
      {:else if activeTab === 'actions'}
        <div class="actions-panel">
          {#if agent.status !== 'merged'}
            <div class="action-section">
              <h3>Merge Changes</h3>
              <div class="merge-options">
                <label>
                  <input type="radio" bind:group={mergeStrategy} value="merge" />
                  Merge commit
                </label>
                <label>
                  <input type="radio" bind:group={mergeStrategy} value="rebase" />
                  Rebase
                </label>
                <label>
                  <input type="radio" bind:group={mergeStrategy} value="squash" />
                  Squash
                </label>
              </div>
              <button
                class="btn btn-success"
                onclick={handleMerge}
                disabled={merging || agent.status === 'running'}
              >
                {merging ? 'Merging...' : 'Merge to ' + agent.base_branch}
              </button>
              {#if mergeError}
                <p class="error-msg">{mergeError}</p>
              {/if}
              {#if mergeSuccess}
                <p class="success-msg">{mergeSuccess}</p>
              {/if}
            </div>

            <div class="action-section">
              <h3>Create Pull Request</h3>
              {#if prUrl}
                <p class="success-msg">
                  PR created: <a href={prUrl} target="_blank" rel="noopener">{prUrl}</a>
                </p>
              {:else}
                <button
                  class="btn btn-primary"
                  onclick={handleCreatePr}
                  disabled={creatingPr || agent.status === 'running'}
                >
                  {creatingPr ? 'Creating PR...' : 'Create PR'}
                </button>
                {#if prError}
                  <p class="error-msg">{prError}</p>
                {/if}
              {/if}
            </div>
          {:else}
            <div class="merged-notice">
              <p>This task has been merged into {agent.base_branch}.</p>
            </div>
          {/if}

          <div class="action-section danger-zone">
            <h3>Danger Zone</h3>
            <button class="btn btn-danger" onclick={handleRemove}>
              Remove Agent
            </button>
          </div>
        </div>
      {:else if activeTab === 'output'}
        {#if loadingOutput}
          <div class="loading">Loading output...</div>
        {:else}
          <pre class="output-content">{output || 'No output available'}</pre>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 2rem;
    z-index: 100;
  }

  .modal {
    background-color: var(--bg-secondary);
    border-radius: 0.5rem;
    width: 100%;
    max-width: 900px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 25px 50px -12px var(--shadow);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-left h2 {
    font-size: 1.25rem;
  }

  .status {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 9999px;
    text-transform: uppercase;
    font-weight: 600;
  }

  .status-running {
    background-color: rgba(59, 130, 246, 0.2);
    color: var(--accent);
  }

  .status-completed {
    background-color: rgba(34, 197, 94, 0.2);
    color: var(--success);
  }

  .status-failed {
    background-color: rgba(239, 68, 68, 0.2);
    color: var(--error);
  }

  .status-merged {
    background-color: rgba(168, 85, 247, 0.2);
    color: var(--merged);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    padding: 0.5rem;
    border-radius: 0.25rem;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background-color: var(--bg-card);
    color: var(--text-primary);
  }

  .task-info {
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .task-description {
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .meta strong {
    color: var(--text-muted);
    font-weight: 500;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    padding: 0 1.5rem;
  }

  .tab {
    background: none;
    border: none;
    padding: 0.75rem 1rem;
    color: var(--text-secondary);
    font-weight: 500;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 0.2s;
  }

  .tab:hover {
    color: var(--text-primary);
  }

  .tab.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.5rem;
    min-height: 300px;
  }

  .loading, .empty {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: var(--text-secondary);
  }

  .error {
    color: var(--error);
    padding: 1rem;
  }

  .actions-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .action-section {
    padding: 1rem;
    background-color: var(--bg-card);
    border-radius: 0.375rem;
  }

  .action-section h3 {
    font-size: 0.875rem;
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .merge-options {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.75rem;
  }

  .merge-options label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .error-msg {
    color: var(--error);
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .success-msg {
    color: var(--success);
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .success-msg a {
    color: var(--accent);
  }

  .danger-zone {
    border: 1px solid var(--error);
    background-color: rgba(239, 68, 68, 0.1);
  }

  .merged-notice {
    padding: 1rem;
    background-color: rgba(168, 85, 247, 0.1);
    border-radius: 0.375rem;
    color: var(--merged);
    text-align: center;
  }

  .output-content {
    background-color: var(--bg-primary);
    padding: 1rem;
    border-radius: 0.375rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--text-secondary);
    max-height: 400px;
    overflow-y: auto;
  }
</style>
