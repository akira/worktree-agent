<script>
  let { diff } = $props();

  function formatDiffLine(line) {
    if (line.startsWith('+++') || line.startsWith('---')) {
      return { class: 'diff-header', text: line };
    }
    if (line.startsWith('@@')) {
      return { class: 'diff-hunk', text: line };
    }
    if (line.startsWith('+')) {
      return { class: 'diff-addition', text: line };
    }
    if (line.startsWith('-')) {
      return { class: 'diff-deletion', text: line };
    }
    if (line.startsWith('diff --git')) {
      return { class: 'diff-file-header', text: line };
    }
    return { class: 'diff-context', text: line };
  }

  function splitDiff() {
    if (!diff?.diff) return [];
    return diff.diff.split('\n').map(formatDiffLine);
  }
</script>

<div class="diff-viewer">
  <div class="diff-stats">
    <span class="stat files">{diff?.stats?.files_changed || 0} files changed</span>
    <span class="stat additions">+{diff?.stats?.additions || 0}</span>
    <span class="stat deletions">-{diff?.stats?.deletions || 0}</span>
  </div>

  {#if diff?.files_changed?.length > 0}
    <div class="files-list">
      <h4>Changed Files</h4>
      <ul>
        {#each diff.files_changed as file}
          <li>{file}</li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="diff-content">
    {#each splitDiff() as line}
      <div class="diff-line {line.class}">{line.text}</div>
    {/each}
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .diff-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
  }

  .stat {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    background-color: var(--bg-card);
  }

  .stat.additions {
    color: var(--success);
  }

  .stat.deletions {
    color: var(--error);
  }

  .files-list {
    background-color: var(--bg-card);
    padding: 0.75rem;
    border-radius: 0.375rem;
  }

  .files-list h4 {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 0.5rem;
  }

  .files-list ul {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .files-list li {
    font-size: 0.8125rem;
    font-family: 'SF Mono', monospace;
    color: var(--text-secondary);
    background-color: var(--bg-secondary);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .diff-content {
    background-color: var(--bg-primary);
    border-radius: 0.375rem;
    padding: 0.5rem;
    overflow-x: auto;
    max-height: 400px;
    overflow-y: auto;
  }

  .diff-line {
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
    font-size: 0.8125rem;
    padding: 0.125rem 0.5rem;
    white-space: pre;
    border-radius: 0.125rem;
    margin: 1px 0;
  }

  .diff-addition {
    background-color: rgba(34, 197, 94, 0.15);
    color: #4ade80;
  }

  .diff-deletion {
    background-color: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  .diff-header {
    color: var(--text-primary);
    font-weight: 600;
  }

  .diff-hunk {
    color: var(--accent);
    background-color: rgba(59, 130, 246, 0.1);
  }

  .diff-file-header {
    color: var(--warning);
    font-weight: 600;
    margin-top: 0.5rem;
  }

  .diff-context {
    color: var(--text-muted);
  }
</style>
