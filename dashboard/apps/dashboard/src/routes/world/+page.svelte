<script lang="ts">
  import { client } from '$lib/api';
  import type { QueryResultRow } from '@rgb/api-client';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { registerPageHandlers } from '$lib/keybinds.svelte';

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
  let selectedEntity: EntityMarker | null = $state(null);

  async function loadEntities() {
    loading = true;
    error = null;
    try {
      const result = await client.current.query({ with: ['Position'], limit: 1000 });
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
    return { x: centerX + worldX * scale, y: centerY + worldZ * scale };
  }

  function screenToWorld(screenX: number, screenY: number): { x: number; z: number } {
    const centerX = canvas.width / 2 + offsetX;
    const centerY = canvas.height / 2 + offsetY;
    return { x: (screenX - centerX) / scale, z: (screenY - centerY) / scale };
  }

  function drawMap() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    ctx.fillStyle = isDark ? '#0a0a0f' : '#fafafa';
    ctx.fillRect(0, 0, width, height);

    const centerX = width / 2 + offsetX;
    const centerY = height / 2 + offsetY;

    // Grid
    ctx.strokeStyle = isDark ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.04)';
    ctx.lineWidth = 1;
    const gridSize = 16 * scale;

    const startX = Math.floor(-centerX / gridSize) - 1;
    const endX = Math.ceil((width - centerX) / gridSize) + 1;
    const startZ = Math.floor(-centerY / gridSize) - 1;
    const endZ = Math.ceil((height - centerY) / gridSize) + 1;

    for (let gx = startX; gx <= endX; gx++) {
      const x = centerX + gx * gridSize;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    for (let gz = startZ; gz <= endZ; gz++) {
      const y = centerY + gz * gridSize;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Axes
    ctx.strokeStyle = isDark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, centerY);
    ctx.lineTo(width, centerY);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(centerX, 0);
    ctx.lineTo(centerX, height);
    ctx.stroke();

    // Origin
    ctx.beginPath();
    ctx.arc(centerX, centerY, 4, 0, Math.PI * 2);
    ctx.fillStyle = isDark ? '#fff' : '#111';
    ctx.fill();

    // Entities
    const markerRadius = Math.max(4, Math.min(10, scale * 1.5));

    for (const marker of entities) {
      const screen = worldToScreen(marker.x, marker.z);
      if (screen.x < -markerRadius || screen.x > width + markerRadius ||
          screen.y < -markerRadius || screen.y > height + markerRadius) continue;

      const isHovered = hoveredEntity?.entity === marker.entity;
      const isSelected = selectedEntity?.entity === marker.entity;

      ctx.beginPath();
      ctx.arc(screen.x, screen.y, markerRadius + (isHovered ? 2 : 0), 0, Math.PI * 2);
      ctx.fillStyle = '#ef4444';
      ctx.fill();

      if (isSelected || isHovered) {
        ctx.strokeStyle = isDark ? '#fff' : '#111';
        ctx.lineWidth = 2;
        ctx.stroke();
      }
    }

    // Tooltip
    if (hoveredEntity) {
      const screen = worldToScreen(hoveredEntity.x, hoveredEntity.z);
      const label = hoveredEntity.name ?? `#${hoveredEntity.entity}`;
      ctx.font = '12px -apple-system, sans-serif';
      const textWidth = ctx.measureText(label).width;

      ctx.fillStyle = isDark ? 'rgba(0,0,0,0.8)' : 'rgba(255,255,255,0.9)';
      ctx.fillRect(screen.x - textWidth / 2 - 8, screen.y - markerRadius - 30, textWidth + 16, 24);

      ctx.fillStyle = isDark ? '#fff' : '#111';
      ctx.textAlign = 'center';
      ctx.fillText(label, screen.x, screen.y - markerRadius - 14);
      ctx.textAlign = 'left';
    }
  }

  function findEntityAt(screenX: number, screenY: number): EntityMarker | null {
    const markerRadius = Math.max(4, Math.min(10, scale * 1.5));
    for (const marker of entities) {
      const screen = worldToScreen(marker.x, marker.z);
      const dx = screenX - screen.x;
      const dy = screenY - screen.y;
      if (Math.sqrt(dx * dx + dy * dy) <= markerRadius + 4) return marker;
    }
    return null;
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    const worldBefore = screenToWorld(mouseX, mouseY);

    const delta = e.deltaY > 0 ? -0.2 : 0.2;
    scale = Math.max(0.1, Math.min(20, scale + delta * scale));

    const worldAfter = screenToWorld(mouseX, mouseY);
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
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

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
    const entity = findEntityAt(e.clientX - rect.left, e.clientY - rect.top);
    selectedEntity = entity;
    drawMap();
  }

  function handleDoubleClick(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const entity = findEntityAt(e.clientX - rect.left, e.clientY - rect.top);
    if (entity) goto(`/entities/${entity.entity}`);
  }

  function resetView() {
    offsetX = 0;
    offsetY = 0;
    scale = 2;
    selectedEntity = null;
    drawMap();
  }

  const PAN_AMOUNT = 50;

  function panLeft() {
    offsetX += PAN_AMOUNT;
    drawMap();
  }

  function panRight() {
    offsetX -= PAN_AMOUNT;
    drawMap();
  }

  function panUp() {
    offsetY += PAN_AMOUNT;
    drawMap();
  }

  function panDown() {
    offsetY -= PAN_AMOUNT;
    drawMap();
  }

  function zoomIn() {
    scale = Math.min(20, scale * 1.2);
    drawMap();
  }

  function zoomOut() {
    scale = Math.max(0.1, scale / 1.2);
    drawMap();
  }

  $effect(() => {
    if (entities.length > 0) drawMap();
  });

  onMount(() => {
    loadEntities();

    // Register keybind handlers
    const unregisterKeybinds = registerPageHandlers({
      panUp,
      panDown,
      panLeft,
      panRight,
      zoomIn,
      zoomOut,
      reset: resetView,
      refresh: loadEntities,
    });

    const resizeObserver = new ResizeObserver(() => {
      if (canvas) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        drawMap();
      }
    });

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', drawMap);

    if (canvas) {
      resizeObserver.observe(canvas);
      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;
    }

    const interval = setInterval(loadEntities, 2000);

    return () => {
      clearInterval(interval);
      resizeObserver.disconnect();
      mediaQuery.removeEventListener('change', drawMap);
      unregisterKeybinds();
    };
  });
</script>

<svelte:head>
  <title>World - RGB</title>
</svelte:head>

<div class="page">
  <div class="toolbar">
    <div class="stats">
      <span class="stat">{entities.length} entities</span>
      <span class="stat">{scale.toFixed(1)}x</span>
    </div>
    <div class="actions">
      {#if selectedEntity}
        <a href="/entities/{selectedEntity.entity}" class="btn">View #{selectedEntity.entity}</a>
      {/if}
      <button class="btn" onclick={resetView}>Reset</button>
      <button class="btn" onclick={loadEntities} disabled={loading}>
        {loading ? '...' : 'Refresh'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {:else}
    <div class="canvas-wrap">
      <canvas
        bind:this={canvas}
        class="canvas"
        onwheel={handleWheel}
        onmousedown={handleMouseDown}
        onmousemove={handleMouseMove}
        onmouseup={handleMouseUp}
        onmouseleave={handleMouseUp}
        onclick={handleClick}
        ondblclick={handleDoubleClick}
      ></canvas>
    </div>
  {/if}

  {#if selectedEntity}
    <div class="selection">
      <span class="name">{selectedEntity.name ?? `Entity ${selectedEntity.entity}`}</span>
      <span class="coords">
        {selectedEntity.x.toFixed(1)}, {selectedEntity.y.toFixed(1)}, {selectedEntity.z.toFixed(1)}
      </span>
    </div>
  {/if}
</div>

<style>
  .page {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
    gap: 12px;
    min-height: 0;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .stats {
    display: flex;
    gap: 16px;
  }

  .stat {
    font-size: 13px;
    color: rgba(0, 0, 0, 0.5);
  }

  @media (prefers-color-scheme: dark) {
    .stat { color: rgba(255, 255, 255, 0.5); }
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .btn {
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.7);
    background: transparent;
    border: 1px solid rgba(0, 0, 0, 0.12);
    border-radius: 6px;
    cursor: pointer;
    text-decoration: none;
    transition: background 0.1s;
  }

  .btn:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.04);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .btn {
      color: rgba(255, 255, 255, 0.7);
      border-color: rgba(255, 255, 255, 0.12);
    }
    .btn:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.06);
    }
  }

  .error {
    padding: 12px 16px;
    font-size: 13px;
    color: #dc2626;
    background: rgba(220, 38, 38, 0.08);
    border-radius: 8px;
  }

  .canvas-wrap {
    flex: 1;
    min-height: 0;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  @media (prefers-color-scheme: dark) {
    .canvas-wrap { border-color: rgba(255, 255, 255, 0.08); }
  }

  .canvas {
    width: 100%;
    height: 100%;
    cursor: grab;
  }

  .canvas:active {
    cursor: grabbing;
  }

  .selection {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.95);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 8px;
    backdrop-filter: blur(8px);
    font-size: 13px;
  }

  @media (prefers-color-scheme: dark) {
    .selection {
      background: rgba(20, 20, 30, 0.95);
      border-color: rgba(255, 255, 255, 0.08);
    }
  }

  .name {
    font-weight: 500;
  }

  .coords {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    color: rgba(0, 0, 0, 0.5);
  }

  @media (prefers-color-scheme: dark) {
    .coords { color: rgba(255, 255, 255, 0.5); }
  }
</style>
