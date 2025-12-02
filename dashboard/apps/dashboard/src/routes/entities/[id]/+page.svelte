<script lang="ts">
  import { page } from '$app/stores';
  import { client } from '$lib/api';
  import type { EntityResponse, HistoryEntry } from '$lib/api';
  import { onMount } from 'svelte';
  import HistoryTimeline from '$lib/components/HistoryTimeline.svelte';

  let entity: EntityResponse | null = $state(null);
  let history: HistoryEntry[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let editingComponent: string | null = $state(null);
  let editValue = $state('');
  let saving = $state(false);
  let activeTab: 'components' | 'history' = $state('components');

  $effect(() => {
    const id = $page.params.id;
    if (id) {
      loadEntity(Number(id));
    }
  });

  async function loadEntity(id: number) {
    loading = true;
    error = null;
    try {
      const [entityData, historyData] = await Promise.all([
        client.current.getEntity(id),
        client.current.getEntityHistory(id, 50),
      ]);
      entity = entityData;
      history = historyData.entries;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load entity';
    } finally {
      loading = false;
    }
  }

  async function loadHistory() {
    if (!entity) return;
    try {
      const historyData = await client.current.getEntityHistory(entity.id, 50);
      history = historyData.entries;
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  }

  function handleRevert() {
    if (entity) {
      loadEntity(entity.id);
    }
  }

  function startEditing(componentName: string, currentValue: unknown) {
    editingComponent = componentName;
    editValue = JSON.stringify(currentValue, null, 2);
  }

  function cancelEditing() {
    editingComponent = null;
    editValue = '';
  }

  async function saveComponent(componentName: string) {
    if (!entity) return;

    saving = true;
    try {
      const value = JSON.parse(editValue);
      await client.current.updateComponent(entity.id, componentName, value);
      await loadEntity(entity.id);
      editingComponent = null;
      editValue = '';
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to save component');
    } finally {
      saving = false;
    }
  }

  async function removeComponent(componentName: string) {
    if (!entity) return;
    if (!confirm(`Remove component "${componentName}"?`)) return;

    try {
      await client.current.removeComponent(entity.id, componentName);
      await loadEntity(entity.id);
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to remove component');
    }
  }

  async function despawnEntity() {
    if (!entity) return;
    if (!confirm(`Despawn entity ${entity.id}?`)) return;

    try {
      await client.current.despawnEntity(entity.id);
      window.location.href = '/';
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Failed to despawn entity');
    }
  }

  onMount(() => {
    const interval = setInterval(() => {
      if (entity && !editingComponent) {
        loadEntity(entity.id);
      }
    }, 2000);
    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>{entity?.name ?? `Entity ${$page.params.id}`} - RGB ECS Dashboard</title>
</svelte:head>

<main class="container">
  <nav class="breadcrumb">
    <a href="/">Dashboard</a> &gt; <span>Entity {$page.params.id}</span>
  </nav>

  {#if loading && !entity}
    <p>Loading...</p>
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={() => loadEntity(Number($page.params.id))}>Retry</button>
    </div>
  {:else if entity}
    <header class="entity-header">
      <h1>{entity.name ?? `Entity ${entity.id}`}</h1>
      <div class="entity-meta">
        <span class="entity-id">ID: {entity.id}</span>
        <button class="btn-danger" onclick={despawnEntity}>Despawn</button>
      </div>
    </header>

    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === 'components'}
        onclick={() => activeTab = 'components'}
      >
        Components ({Object.keys(entity.components).length})
      </button>
      <button
        class="tab"
        class:active={activeTab === 'history'}
        onclick={() => { activeTab = 'history'; loadHistory(); }}
      >
        History ({history.length})
      </button>
    </div>

    {#if activeTab === 'components'}
      <section class="components">
        {#each entity.components as comp}
          <div class="component">
            <div class="component-header">
              <h3>{comp.name}</h3>
              <div class="component-actions">
                {#if editingComponent === comp.name}
                  <button onclick={() => saveComponent(comp.name)} disabled={saving}>
                    {saving ? 'Saving...' : 'Save'}
                  </button>
                  <button onclick={cancelEditing} disabled={saving}>Cancel</button>
                {:else if !comp.is_opaque}
                  <button onclick={() => startEditing(comp.name, comp.value)}>Edit</button>
                  <button class="btn-danger" onclick={() => removeComponent(comp.name)}>Remove</button>
                {:else}
                  <button class="btn-danger" onclick={() => removeComponent(comp.name)}>Remove</button>
                {/if}
              </div>
            </div>

            {#if editingComponent === comp.name}
              <textarea
                bind:value={editValue}
                class="component-editor"
                rows="10"
                disabled={saving}
              ></textarea>
            {:else}
              <pre class="component-value">{JSON.stringify(comp.value, null, 2)}</pre>
            {/if}
          </div>
        {/each}

        {#if entity.components.length === 0}
          <p class="no-components">No components</p>
        {/if}
      </section>
    {:else}
      <section class="history-section">
        <HistoryTimeline entries={history} onRevert={handleRevert} />
      </section>
    {/if}
  {/if}
</main>

<style>
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    font-family: system-ui, sans-serif;
  }

  .breadcrumb {
    margin-bottom: 20px;
    font-size: 0.875rem;
    color: #666;
  }

  .breadcrumb a {
    color: #0066cc;
  }

  .error {
    background: #fee;
    border: 1px solid #c00;
    padding: 10px;
    border-radius: 4px;
  }

  .entity-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid #ddd;
  }

  .entity-header h1 {
    margin: 0;
  }

  .entity-meta {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .entity-id {
    font-family: monospace;
    background: #f5f5f5;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 24px;
    border-bottom: 2px solid #e5e7eb;
    padding-bottom: 0;
  }

  .tab {
    padding: 12px 24px;
    border: none;
    background: transparent;
    font-size: 0.9375rem;
    font-weight: 500;
    color: #6b7280;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    transition: all 0.15s ease;
  }

  .tab:hover {
    color: #374151;
    background: #f9fafb;
  }

  .tab.active {
    color: #2563eb;
    border-bottom-color: #2563eb;
  }

  .history-section {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 24px;
  }

  .components h2 {
    margin-bottom: 16px;
  }

  .component {
    background: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 8px;
    margin-bottom: 16px;
    overflow: hidden;
  }

  .component-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #f0f0f0;
    border-bottom: 1px solid #ddd;
  }

  .component-header h3 {
    margin: 0;
    font-size: 1rem;
    font-family: monospace;
  }

  .component-actions {
    display: flex;
    gap: 8px;
  }

  .component-value {
    margin: 0;
    padding: 16px;
    background: #fff;
    font-size: 0.875rem;
    overflow-x: auto;
  }

  .component-editor {
    width: 100%;
    padding: 16px;
    border: none;
    font-family: monospace;
    font-size: 0.875rem;
    resize: vertical;
    box-sizing: border-box;
  }

  .no-components {
    color: #666;
    font-style: italic;
  }

  button {
    padding: 6px 12px;
    border: 1px solid #ccc;
    border-radius: 4px;
    background: #fff;
    cursor: pointer;
    font-size: 0.875rem;
  }

  button:hover:not(:disabled) {
    background: #f0f0f0;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-danger {
    color: #c00;
    border-color: #c00;
  }

  .btn-danger:hover:not(:disabled) {
    background: #fee;
  }
</style>
