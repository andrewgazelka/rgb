<script lang="ts">
  import { client, getApiUrl, setApiUrl, type WorldResponse, type ListEntitiesResponse } from '$lib/api';
  import { onMount } from 'svelte';

  let world: WorldResponse | null = $state(null);
  let entities: ListEntitiesResponse | null = $state(null);
  let loading = $state(true);
  let error: string | null = $state(null);
  let apiUrl = $state('');
  let showSettings = $state(false);

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

  function handleUrlChange() {
    setApiUrl(apiUrl);
    showSettings = false;
    loadData();
  }

  onMount(() => {
    apiUrl = getApiUrl();
    loadData();
    // Refresh every 2 seconds
    const interval = setInterval(loadData, 2000);
    return () => clearInterval(interval);
  });
</script>

<svelte:head>
  <title>RGB ECS Dashboard</title>
</svelte:head>

<main class="container">
  <header class="header">
    <h1>RGB ECS Dashboard</h1>
    <nav class="nav">
      <a href="/world" class="nav-link">World Map</a>
      <a href="/map" class="nav-link secondary">Chunk Map</a>
      <button class="settings-btn" onclick={() => showSettings = !showSettings}>
        {showSettings ? 'Close' : 'Settings'}
      </button>
    </nav>
  </header>

  {#if showSettings}
    <div class="settings-panel">
      <label>
        <span>API Server URL:</span>
        <input
          type="text"
          bind:value={apiUrl}
          placeholder="http://localhost:8080"
        />
      </label>
      <button onclick={handleUrlChange}>Connect</button>
    </div>
  {/if}

  {#if loading && !world}
    <p>Loading...</p>
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={loadData}>Retry</button>
    </div>
  {:else if world}
    <section class="stats">
      <h2>World Stats</h2>
      <div class="stat-grid">
        <div class="stat">
          <span class="stat-value">{world.entity_count}</span>
          <span class="stat-label">Entities</span>
        </div>
        <div class="stat">
          <span class="stat-value">{world.archetype_count}</span>
          <span class="stat-label">Archetypes</span>
        </div>
        <div class="stat">
          <span class="stat-value">{world.component_count}</span>
          <span class="stat-label">Component Types</span>
        </div>
      </div>
    </section>

    {#if entities}
      <section class="entities">
        <h2>Entities ({entities.total})</h2>
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
                <td>
                  <a href="/entities/{entity.id}">{entity.id}</a>
                </td>
                <td>{entity.name ?? '(unnamed)'}</td>
                <td>{entity.components.join(', ')}</td>
              </tr>
            {/each}
          </tbody>
        </table>
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

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  h1 {
    margin: 0;
  }

  .nav {
    display: flex;
    gap: 16px;
  }

  .nav-link {
    padding: 8px 16px;
    background: #0066cc;
    color: #fff;
    text-decoration: none;
    border-radius: 4px;
  }

  .nav-link:hover {
    background: #0055aa;
  }

  .nav-link.secondary {
    background: #475569;
  }

  .nav-link.secondary:hover {
    background: #334155;
  }

  .settings-btn {
    padding: 8px 16px;
    background: #666;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .settings-btn:hover {
    background: #555;
  }

  .settings-panel {
    background: #f5f5f5;
    padding: 16px;
    border-radius: 8px;
    margin-bottom: 20px;
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .settings-panel label {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .settings-panel input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-family: monospace;
  }

  .settings-panel button {
    padding: 8px 16px;
    background: #0066cc;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .settings-panel button:hover {
    background: #0055aa;
  }

  .error {
    background: #fee;
    border: 1px solid #c00;
    padding: 10px;
    border-radius: 4px;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 16px;
    margin: 16px 0;
  }

  .stat {
    background: #f5f5f5;
    padding: 16px;
    border-radius: 8px;
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: 2rem;
    font-weight: bold;
    color: #333;
  }

  .stat-label {
    display: block;
    font-size: 0.875rem;
    color: #666;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 10px;
  }

  th, td {
    border: 1px solid #ddd;
    padding: 8px;
    text-align: left;
  }

  th {
    background: #f5f5f5;
  }

  tr:hover {
    background: #f9f9f9;
  }

  a {
    color: #0066cc;
  }
</style>
