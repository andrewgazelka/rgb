<script lang="ts">
  import { client } from '$lib/api';
  import type { ChunksResponse } from '$lib/api';
  import { onMount } from 'svelte';

  interface EntityPosition {
    entity: number;
    name: string | null;
    x: number;
    y: number;
    z: number;
  }

  let chunks: ChunksResponse | null = $state(null);
  let entities: EntityPosition[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let canvas: HTMLCanvasElement;
  let scale = $state(4);
  let offsetX = $state(0);
  let offsetY = $state(0);
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;
  let showEntities = $state(true);

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [chunksData, entitiesData] = await Promise.all([
        client.current.getChunks(),
        client.current.query({ with: ['Position'] })
      ]);
      chunks = chunksData;
      entities = entitiesData.entities.map(e => {
        const pos = e.components['Position'] as { x: number; y: number; z: number } | undefined;
        return {
          entity: e.entity,
          name: e.name,
          x: pos?.x ?? 0,
          y: pos?.y ?? 0,
          z: pos?.z ?? 0
        };
      });
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
    } finally {
      loading = false;
    }
  }

  function regionToColor(regionX: number, regionZ: number, isDark: boolean): string {
    const hash = Math.abs((regionX * 73 + regionZ * 137) % 360);
    const saturation = isDark ? 30 : 40;
    const lightness = isDark ? 25 : 85;
    return `hsl(${hash}, ${saturation}%, ${lightness}%)`;
  }

  function drawMap() {
    if (!canvas || !chunks) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    // Clear canvas
    ctx.fillStyle = isDark ? '#0f0f1a' : '#fafafa';
    ctx.fillRect(0, 0, width, height);

    const centerX = width / 2 + offsetX;
    const centerY = height / 2 + offsetY;

    // Draw chunks
    for (const chunk of chunks.chunks) {
      const regionX = Math.floor(chunk.x / 32);
      const regionZ = Math.floor(chunk.z / 32);

      const x = centerX + chunk.x * scale;
      const y = centerY + chunk.z * scale;

      ctx.fillStyle = regionToColor(regionX, regionZ, isDark);
      ctx.fillRect(x, y, scale - 0.5, scale - 0.5);
    }

    // Draw region grid
    ctx.strokeStyle = isDark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.08)';
    ctx.lineWidth = 1;
    const regionSize = 32 * scale;

    const startRegionX = Math.floor((-centerX) / regionSize) - 1;
    const endRegionX = Math.floor((width - centerX) / regionSize) + 1;
    const startRegionZ = Math.floor((-centerY) / regionSize) - 1;
    const endRegionZ = Math.floor((height - centerY) / regionSize) + 1;

    for (let rx = startRegionX; rx <= endRegionX; rx++) {
      const x = centerX + rx * regionSize;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }

    for (let rz = startRegionZ; rz <= endRegionZ; rz++) {
      const y = centerY + rz * regionSize;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw entities
    if (showEntities && entities.length > 0) {
      const blockScale = scale / 16;

      for (const entity of entities) {
        const x = centerX + entity.x * blockScale;
        const y = centerY + entity.z * blockScale;

        ctx.beginPath();
        ctx.arc(x, y, Math.max(3, scale / 4), 0, Math.PI * 2);
        ctx.fillStyle = '#ef4444';
        ctx.fill();
      }
    }

    // Draw origin
    ctx.beginPath();
    ctx.arc(centerX, centerY, 4, 0, Math.PI * 2);
    ctx.fillStyle = isDark ? '#fff' : '#111';
    ctx.fill();
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -1 : 1;
    scale = Math.max(1, Math.min(32, scale + delta));
    drawMap();
  }

  function handleMouseDown(e: MouseEvent) {
    isDragging = true;
    dragStartX = e.clientX - offsetX;
    dragStartY = e.clientY - offsetY;
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    offsetX = e.clientX - dragStartX;
    offsetY = e.clientY - dragStartY;
    drawMap();
  }

  function handleMouseUp() {
    isDragging = false;
  }

  function resetView() {
    offsetX = 0;
    offsetY = 0;
    scale = 4;
    drawMap();
  }

  $effect(() => {
    if (chunks) {
      drawMap();
    }
  });

  onMount(() => {
    loadData();

    const resizeObserver = new ResizeObserver(() => {
      if (canvas) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        drawMap();
      }
    });

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleColorSchemeChange = () => drawMap();
    mediaQuery.addEventListener('change', handleColorSchemeChange);

    if (canvas) {
      resizeObserver.observe(canvas);
      canvas.width = canvas.clientWidth;
      canvas.height = canvas.clientHeight;
    }

    const interval = setInterval(loadData, 2000);

    return () => {
      clearInterval(interval);
      resizeObserver.disconnect();
      mediaQuery.removeEventListener('change', handleColorSchemeChange);
    };
  });
</script>

<svelte:head>
  <title>Map - RGB</title>
</svelte:head>

<div class="page">
  <div class="toolbar">
    <div class="stats">
      <span class="stat">{chunks?.chunks.length ?? 0} chunks</span>
      <span class="stat">{entities.length} entities</span>
    </div>

    <div class="controls">
      <label class="toggle">
        <input type="checkbox" bind:checked={showEntities} onchange={drawMap} />
        <span>Entities</span>
      </label>

      <span class="zoom">{scale}x</span>

      <button class="btn" onclick={resetView}>Reset</button>
      <button class="btn" onclick={loadData} disabled={loading}>
        {loading ? '...' : 'Refresh'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error">
      {error}
      <button class="btn" onclick={loadData}>Retry</button>
    </div>
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
      ></canvas>
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
    .stat {
      color: rgba(255, 255, 255, 0.5);
    }
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: rgba(0, 0, 0, 0.6);
    cursor: pointer;
  }

  @media (prefers-color-scheme: dark) {
    .toggle {
      color: rgba(255, 255, 255, 0.6);
    }
  }

  .toggle input {
    cursor: pointer;
  }

  .zoom {
    font-size: 12px;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.4);
    min-width: 28px;
  }

  @media (prefers-color-scheme: dark) {
    .zoom {
      color: rgba(255, 255, 255, 0.4);
    }
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
    transition: background 0.1s, border-color 0.1s;
  }

  .btn:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.04);
    border-color: rgba(0, 0, 0, 0.2);
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
      border-color: rgba(255, 255, 255, 0.2);
    }
  }

  .error {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    font-size: 13px;
    color: #dc2626;
    background: #fef2f2;
    border-radius: 8px;
  }

  @media (prefers-color-scheme: dark) {
    .error {
      background: rgba(220, 38, 38, 0.1);
    }
  }

  .canvas-wrap {
    flex: 1;
    border-radius: 8px;
    overflow: hidden;
    border: 1px solid rgba(0, 0, 0, 0.08);
  }

  @media (prefers-color-scheme: dark) {
    .canvas-wrap {
      border-color: rgba(255, 255, 255, 0.08);
    }
  }

  .canvas {
    width: 100%;
    height: 100%;
    cursor: grab;
  }

  .canvas:active {
    cursor: grabbing;
  }
</style>
