<script lang="ts">
  import { client, type WorldResponse, type ListEntitiesResponse } from '$lib/api';
  import { onMount } from 'svelte';

  let world: WorldResponse | null = $state(null);
  let entities: ListEntitiesResponse | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);

  async function loadData() {
    loading = true;
    error = null;
    try {
      [world, entities] = await Promise.all([
        client.current.getWorld(),
        client.current.listEntities({ limit: 100 }),
      ]);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
    const interval = setInterval(loadData, 2000);
    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>Dashboard - RGB</title>
</svelte:head>

<div class="page">
  {#if loading && !world}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">
      {error}
      <button class="btn" onclick={loadData}>Retry</button>
    </div>
  {:else if world}
    <div class="stats">
      <div class="stat">
        <span class="value">{world.entity_count.toLocaleString()}</span>
        <span class="label">Entities</span>
      </div>
      <div class="stat">
        <span class="value">{world.archetype_count}</span>
        <span class="label">Archetypes</span>
      </div>
      <div class="stat">
        <span class="value">{world.component_count}</span>
        <span class="label">Components</span>
      </div>
    </div>

    {#if entities && entities.entities.length > 0}
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Components</th>
            </tr>
          </thead>
          <tbody>
            {#each entities.entities as entity}
              <tr>
                <td class="id">
                  <a href="/entities/{entity.id}">{entity.id}</a>
                </td>
                <td class="name">{entity.name ?? '-'}</td>
                <td class="components">{entity.components.join(', ')}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .page {
    flex: 1;
    padding: 24px;
    max-width: 1000px;
    margin: 0 auto;
    width: 100%;
    min-height: 0;
    overflow: auto;
  }

  .btn {
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 500;
    color: #fff;
    background: #1a1a1a;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .btn:hover {
    background: #333;
  }

  @media (prefers-color-scheme: dark) {
    .btn {
      background: #e5e5e5;
      color: #0a0a0a;
    }
    .btn:hover {
      background: #fff;
    }
  }

  .loading {
    color: rgba(0, 0, 0, 0.4);
    font-size: 13px;
  }

  @media (prefers-color-scheme: dark) {
    .loading {
      color: rgba(255, 255, 255, 0.4);
    }
  }

  .error {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    font-size: 13px;
    color: #dc2626;
    background: rgba(220, 38, 38, 0.08);
    border-radius: 8px;
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 10px;
    overflow: hidden;
    margin-bottom: 24px;
  }

  @media (prefers-color-scheme: dark) {
    .stats {
      background: rgba(255, 255, 255, 0.06);
    }
  }

  .stat {
    padding: 24px;
    background: rgba(0, 0, 0, 0.02);
    text-align: center;
  }

  @media (prefers-color-scheme: dark) {
    .stat {
      background: rgba(255, 255, 255, 0.02);
    }
  }

  .value {
    display: block;
    font-size: 32px;
    font-weight: 600;
    letter-spacing: -0.02em;
    margin-bottom: 4px;
  }

  .label {
    font-size: 12px;
    color: rgba(0, 0, 0, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  @media (prefers-color-scheme: dark) {
    .label {
      color: rgba(255, 255, 255, 0.5);
    }
  }

  .table-wrap {
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 10px;
    overflow: hidden;
  }

  @media (prefers-color-scheme: dark) {
    .table-wrap {
      border-color: rgba(255, 255, 255, 0.08);
    }
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    padding: 10px 16px;
    text-align: left;
    font-weight: 500;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: rgba(0, 0, 0, 0.5);
    background: rgba(0, 0, 0, 0.02);
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }

  @media (prefers-color-scheme: dark) {
    th {
      color: rgba(255, 255, 255, 0.5);
      background: rgba(255, 255, 255, 0.02);
      border-bottom-color: rgba(255, 255, 255, 0.06);
    }
  }

  td {
    padding: 10px 16px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  }

  @media (prefers-color-scheme: dark) {
    td {
      border-bottom-color: rgba(255, 255, 255, 0.04);
    }
  }

  tr:last-child td {
    border-bottom: none;
  }

  tr:hover td {
    background: rgba(0, 0, 0, 0.02);
  }

  @media (prefers-color-scheme: dark) {
    tr:hover td {
      background: rgba(255, 255, 255, 0.02);
    }
  }

  .id {
    font-family: ui-monospace, monospace;
    font-size: 12px;
  }

  .id a {
    color: inherit;
    text-decoration: none;
  }

  .id a:hover {
    text-decoration: underline;
  }

  .name {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    color: rgba(0, 0, 0, 0.7);
  }

  @media (prefers-color-scheme: dark) {
    .name {
      color: rgba(255, 255, 255, 0.7);
    }
  }

  .components {
    color: rgba(0, 0, 0, 0.5);
    font-size: 12px;
  }

  @media (prefers-color-scheme: dark) {
    .components {
      color: rgba(255, 255, 255, 0.5);
    }
  }

</style>
