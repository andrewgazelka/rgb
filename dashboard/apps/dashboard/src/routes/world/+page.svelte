<script lang="ts">
  import { client } from '$lib/api';
  import type { QueryResultRow } from '@rgb/api-client';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  interface EntityMarker {
    entity: number;
    name: string | null;
    x: number;
    z: number;
    y: number;
  }

  let entities: EntityMarker[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let canvas: HTMLCanvasElement;
  let scale = $state(2);
  let offsetX = $state(0);
  let offsetY = $state(0);
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let hoveredEntity: EntityMarker | null = $state(null);
  let mouseX = $state(0);
  let mouseY = $state(0);
  let selectedEntity: EntityMarker | null = $state(null);

  async function loadEntities() {
    loading = true;
    error = null;
    try {
      const result = await client.current.query({
        with: ['Position'],
        limit: 1000,
      });
      entities = result.entities.map((row: QueryResultRow) => {
        const pos = row.components['Position'] as { x: number; y: number; z: number } | undefined;
        return {
          entity: row.entity,
          name: row.name,
          x: pos?.x ?? 0,
          z: pos?.z ?? 0,
          y: pos?.y ?? 0,
        };
      });
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load entities';
    } finally {
      loading = false;
    }
  }

  function worldToScreen(worldX: number, worldZ: number): { x: number; y: number } {
    const centerX = canvas.width / 2 + offsetX;
    const centerY = canvas.height / 2 + offsetY;
    return {
      x: centerX + worldX * scale,
      y: centerY + worldZ * scale,
    };
  }

  function screenToWorld(screenX: number, screenY: number): { x: number; z: number } {
    const centerX = canvas.width / 2 + offsetX;
    const centerY = canvas.height / 2 + offsetY;
    return {
      x: (screenX - centerX) / scale,
      z: (screenY - centerY) / scale,
    };
  }

  function getEntityColor(marker: EntityMarker): string {
    // Color based on Y height
    const minY = 0;
    const maxY = 256;
    const t = Math.max(0, Math.min(1, (marker.y - minY) / (maxY - minY)));

    // Gradient from deep blue (low) through green to red (high)
    if (t < 0.5) {
      const s = t * 2;
      const r = Math.round(0 + s * 100);
      const g = Math.round(100 + s * 155);
      const b = Math.round(200 - s * 100);
      return `rgb(${r}, ${g}, ${b})`;
    } else {
      const s = (t - 0.5) * 2;
      const r = Math.round(100 + s * 155);
      const g = Math.round(255 - s * 155);
      const b = Math.round(100 - s * 100);
      return `rgb(${r}, ${g}, ${b})`;
    }
  }

  function drawMap() {
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas with dark background
    ctx.fillStyle = '#0f172a';
    ctx.fillRect(0, 0, width, height);

    const centerX = width / 2 + offsetX;
    const centerY = height / 2 + offsetY;

    // Draw grid
    ctx.strokeStyle = '#1e293b';
    ctx.lineWidth = 1;

    const gridSize = 16 * scale; // 16 block grid
    const startGridX = Math.floor(-centerX / gridSize) - 1;
    const endGridX = Math.ceil((width - centerX) / gridSize) + 1;
    const startGridZ = Math.floor(-centerY / gridSize) - 1;
    const endGridZ = Math.ceil((height - centerY) / gridSize) + 1;

    for (let gx = startGridX; gx <= endGridX; gx++) {
      const x = centerX + gx * gridSize;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    for (let gz = startGridZ; gz <= endGridZ; gz++) {
      const y = centerY + gz * gridSize;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw chunk boundaries (16x16) with slightly brighter lines
    ctx.strokeStyle = '#334155';
    const chunkSize = 16 * scale;
    const startChunkX = Math.floor(-centerX / chunkSize) - 1;
    const endChunkX = Math.ceil((width - centerX) / chunkSize) + 1;
    const startChunkZ = Math.floor(-centerY / chunkSize) - 1;
    const endChunkZ = Math.ceil((height - centerY) / chunkSize) + 1;

    for (let cx = startChunkX; cx <= endChunkX; cx++) {
      if (cx % 2 === 0) {
        const x = centerX + cx * chunkSize;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }
    }

    for (let cz = startChunkZ; cz <= endChunkZ; cz++) {
      if (cz % 2 === 0) {
        const y = centerY + cz * chunkSize;
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
        ctx.stroke();
      }
    }

    // Draw axes
    ctx.strokeStyle = '#475569';
    ctx.lineWidth = 2;

    // X axis (horizontal)
    ctx.beginPath();
    ctx.moveTo(0, centerY);
    ctx.lineTo(width, centerY);
    ctx.stroke();

    // Z axis (vertical)
    ctx.beginPath();
    ctx.moveTo(centerX, 0);
    ctx.lineTo(centerX, height);
    ctx.stroke();

    // Draw origin marker
    ctx.fillStyle = '#fff';
    ctx.beginPath();
    ctx.arc(centerX, centerY, 4, 0, Math.PI * 2);
    ctx.fill();

    // Draw axis labels
    ctx.fillStyle = '#94a3b8';
    ctx.font = '12px system-ui';
    ctx.fillText('+X', width - 25, centerY - 8);
    ctx.fillText('-X', 10, centerY - 8);
    ctx.fillText('+Z', centerX + 8, height - 10);
    ctx.fillText('-Z', centerX + 8, 20);

    // Draw entities
    const markerRadius = Math.max(4, Math.min(12, scale * 2));

    for (const marker of entities) {
      const screen = worldToScreen(marker.x, marker.z);

      // Skip if off-screen
      if (screen.x < -markerRadius || screen.x > width + markerRadius ||
          screen.y < -markerRadius || screen.y > height + markerRadius) {
        continue;
      }

      const isHovered = hoveredEntity?.entity === marker.entity;
      const isSelected = selectedEntity?.entity === marker.entity;

      // Draw shadow
      ctx.beginPath();
      ctx.arc(screen.x + 2, screen.y + 2, markerRadius + (isHovered ? 2 : 0), 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
      ctx.fill();

      // Draw marker
      ctx.beginPath();
      ctx.arc(screen.x, screen.y, markerRadius + (isHovered ? 2 : 0), 0, Math.PI * 2);
      ctx.fillStyle = getEntityColor(marker);
      ctx.fill();

      // Draw border
      ctx.strokeStyle = isSelected ? '#fbbf24' : (isHovered ? '#fff' : 'rgba(255, 255, 255, 0.5)');
      ctx.lineWidth = isSelected ? 3 : (isHovered ? 2 : 1);
      ctx.stroke();

      // Draw name label for hovered/selected
      if (isHovered || isSelected) {
        const label = marker.name ?? `Entity ${marker.entity}`;
        ctx.font = 'bold 12px system-ui';
        const textWidth = ctx.measureText(label).width;

        // Background
        ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
        ctx.fillRect(screen.x - textWidth / 2 - 6, screen.y - markerRadius - 28, textWidth + 12, 22);

        // Text
        ctx.fillStyle = '#fff';
        ctx.textAlign = 'center';
        ctx.fillText(label, screen.x, screen.y - markerRadius - 12);
        ctx.textAlign = 'left';
      }
    }

    // Draw coordinate info in corner
    if (hoveredEntity) {
      ctx.fillStyle = 'rgba(15, 23, 42, 0.9)';
      ctx.fillRect(10, height - 70, 160, 60);
      ctx.fillStyle = '#e2e8f0';
      ctx.font = '11px monospace';
      ctx.fillText(`X: ${hoveredEntity.x.toFixed(1)}`, 20, height - 50);
      ctx.fillText(`Y: ${hoveredEntity.y.toFixed(1)}`, 20, height - 35);
      ctx.fillText(`Z: ${hoveredEntity.z.toFixed(1)}`, 20, height - 20);
    }
  }

  function findEntityAt(screenX: number, screenY: number): EntityMarker | null {
    const markerRadius = Math.max(4, Math.min(12, scale * 2));

    for (const marker of entities) {
      const screen = worldToScreen(marker.x, marker.z);
      const dx = screenX - screen.x;
      const dy = screenY - screen.y;
      const dist = Math.sqrt(dx * dx + dy * dy);

      if (dist <= markerRadius + 4) {
        return marker;
      }
    }
    return null;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const mouseCanvasX = e.clientX - rect.left;
    const mouseCanvasY = e.clientY - rect.top;

    const worldBefore = screenToWorld(mouseCanvasX, mouseCanvasY);

    const delta = e.deltaY > 0 ? -0.2 : 0.2;
    scale = Math.max(0.1, Math.min(20, scale + delta * scale));

    // Adjust offset to zoom towards mouse position
    const worldAfter = screenToWorld(mouseCanvasX, mouseCanvasY);
    offsetX += (worldAfter.x - worldBefore.x) * scale;
    offsetY += (worldAfter.z - worldBefore.z) * scale;

    drawMap();
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button === 0) {
      isDragging = true;
      dragStartX = e.clientX - offsetX;
      dragStartY = e.clientY - offsetY;
    }
  }

  function handleMouseMove(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    mouseX = e.clientX - rect.left;
    mouseY = e.clientY - rect.top;

    if (isDragging) {
      offsetX = e.clientX - dragStartX;
      offsetY = e.clientY - dragStartY;
      drawMap();
    } else {
      const entity = findEntityAt(mouseX, mouseY);
      if (entity !== hoveredEntity) {
        hoveredEntity = entity;
        canvas.style.cursor = entity ? 'pointer' : 'grab';
        drawMap();
      }
    }
  }

  function handleMouseUp() {
    isDragging = false;
  }

  function handleClick(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const entity = findEntityAt(x, y);
    if (entity) {
      selectedEntity = entity;
      drawMap();
    } else {
      selectedEntity = null;
      drawMap();
    }
  }

  function handleDoubleClick(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const entity = findEntityAt(x, y);
    if (entity) {
      goto(`/entities/${entity.entity}`);
    }
  }

  function resetView() {
    offsetX = 0;
    offsetY = 0;
    scale = 2;
    selectedEntity = null;
    drawMap();
  }

  function centerOnSelected() {
    if (selectedEntity) {
      offsetX = -selectedEntity.x * scale;
      offsetY = -selectedEntity.z * scale;
      drawMap();
    }
  }

  $effect(() => {
    if (entities.length > 0) {
      drawMap();
    }
  });

  onMount(() => {
    loadEntities();

    const resizeObserver = new ResizeObserver(() => {
      if (canvas) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        drawMap();
      }
    });

    if (canvas) {
      resizeObserver.observe(canvas);
      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;
    }

    const interval = setInterval(loadEntities, 2000);

    return () => {
      clearInterval(interval);
      resizeObserver.disconnect();
    };
  });
</script>

<svelte:head>
  <title>World Map - RGB ECS Dashboard</title>
</svelte:head>

<main class="container">
  <nav class="breadcrumb">
    <a href="/">Dashboard</a> &gt; <span>World Map</span>
  </nav>

  <header class="header">
    <h1>World Map</h1>
    <div class="controls">
      <span class="info">
        {#if loading && entities.length === 0}
          Loading...
        {:else}
          {entities.length} entities
        {/if}
      </span>
      <span class="zoom">Zoom: {scale.toFixed(1)}x</span>
      {#if selectedEntity}
        <button onclick={centerOnSelected}>Center on Selected</button>
      {/if}
      <button onclick={resetView}>Reset View</button>
      <button onclick={loadEntities} disabled={loading}>
        {loading ? 'Loading...' : 'Refresh'}
      </button>
    </div>
  </header>

  {#if error}
    <div class="error">
      <p>Error: {error}</p>
      <button onclick={loadEntities}>Retry</button>
    </div>
  {:else}
    <div class="map-wrapper">
      <div class="map-container">
        <canvas
          bind:this={canvas}
          class="map-canvas"
          onwheel={handleWheel}
          onmousedown={handleMouseDown}
          onmousemove={handleMouseMove}
          onmouseup={handleMouseUp}
          onmouseleave={handleMouseUp}
          onclick={handleClick}
          ondblclick={handleDoubleClick}
        ></canvas>
      </div>

      <aside class="sidebar">
        <div class="panel">
          <h3>Selected Entity</h3>
          {#if selectedEntity}
            <div class="entity-info">
              <p class="entity-name">{selectedEntity.name ?? `Entity ${selectedEntity.entity}`}</p>
              <p class="entity-id">ID: {selectedEntity.entity}</p>
              <div class="coords">
                <span>X: {selectedEntity.x.toFixed(2)}</span>
                <span>Y: {selectedEntity.y.toFixed(2)}</span>
                <span>Z: {selectedEntity.z.toFixed(2)}</span>
              </div>
              <a href="/entities/{selectedEntity.entity}" class="view-btn">View Details</a>
            </div>
          {:else}
            <p class="no-selection">Click an entity to select</p>
          {/if}
        </div>

        <div class="panel">
          <h3>Height Legend</h3>
          <div class="height-legend">
            <div class="gradient"></div>
            <div class="labels">
              <span>Y=256</span>
              <span>Y=128</span>
              <span>Y=0</span>
            </div>
          </div>
        </div>

        <div class="panel">
          <h3>Controls</h3>
          <ul class="help-list">
            <li>Scroll to zoom</li>
            <li>Drag to pan</li>
            <li>Click to select</li>
            <li>Double-click to view</li>
          </ul>
        </div>
      </aside>
    </div>
  {/if}
</main>

<style>
  .container {
    max-width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding: 20px;
    font-family: system-ui, sans-serif;
    box-sizing: border-box;
    background: #0f172a;
    color: #e2e8f0;
  }

  .breadcrumb {
    margin-bottom: 16px;
    font-size: 0.875rem;
    color: #64748b;
  }

  .breadcrumb a {
    color: #3b82f6;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .header h1 {
    margin: 0;
    color: #f8fafc;
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .info, .zoom {
    font-size: 0.875rem;
    color: #94a3b8;
  }

  .error {
    background: #450a0a;
    border: 1px solid #dc2626;
    padding: 16px;
    border-radius: 8px;
    color: #fecaca;
  }

  .map-wrapper {
    flex: 1;
    display: flex;
    gap: 16px;
    min-height: 0;
  }

  .map-container {
    flex: 1;
    border: 1px solid #1e293b;
    border-radius: 12px;
    overflow: hidden;
    min-height: 400px;
  }

  .map-canvas {
    width: 100%;
    height: 100%;
    cursor: grab;
  }

  .map-canvas:active {
    cursor: grabbing;
  }

  .sidebar {
    width: 240px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .panel {
    background: #1e293b;
    border-radius: 12px;
    padding: 16px;
  }

  .panel h3 {
    margin: 0 0 12px 0;
    font-size: 0.875rem;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .entity-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .entity-name {
    margin: 0;
    font-weight: 600;
    color: #f8fafc;
  }

  .entity-id {
    margin: 0;
    font-size: 0.75rem;
    color: #64748b;
    font-family: monospace;
  }

  .coords {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-family: monospace;
    font-size: 0.8125rem;
    color: #94a3b8;
    background: #0f172a;
    padding: 8px 12px;
    border-radius: 6px;
  }

  .view-btn {
    display: block;
    text-align: center;
    padding: 8px 16px;
    background: #3b82f6;
    color: #fff;
    border-radius: 6px;
    text-decoration: none;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.15s;
  }

  .view-btn:hover {
    background: #2563eb;
  }

  .no-selection {
    margin: 0;
    color: #64748b;
    font-style: italic;
    font-size: 0.875rem;
  }

  .height-legend {
    display: flex;
    gap: 12px;
  }

  .gradient {
    width: 20px;
    border-radius: 4px;
    background: linear-gradient(
      to bottom,
      rgb(255, 100, 0),
      rgb(100, 255, 100),
      rgb(0, 100, 200)
    );
  }

  .labels {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    font-size: 0.75rem;
    color: #94a3b8;
    font-family: monospace;
  }

  .help-list {
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.8125rem;
    color: #94a3b8;
  }

  .help-list li {
    padding: 4px 0;
    border-bottom: 1px solid #334155;
  }

  .help-list li:last-child {
    border-bottom: none;
  }

  button {
    padding: 8px 16px;
    border: 1px solid #334155;
    border-radius: 6px;
    background: #1e293b;
    color: #e2e8f0;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.15s;
  }

  button:hover:not(:disabled) {
    background: #334155;
    border-color: #475569;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .map-wrapper {
      flex-direction: column;
    }

    .sidebar {
      width: 100%;
      flex-direction: row;
      flex-wrap: wrap;
    }

    .panel {
      flex: 1;
      min-width: 200px;
    }
  }
</style>
