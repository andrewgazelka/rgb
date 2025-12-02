<script lang="ts">
  import { client } from '$lib/api';
  import type { HistoryEntry } from '@rgb/api-client';

  interface Props {
    entries: HistoryEntry[];
    onRevert?: (entry: HistoryEntry) => void;
  }

  let { entries, onRevert }: Props = $props();

  let expandedEntry: number | null = $state(null);
  let reverting = $state<number | null>(null);

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  function formatRelativeTime(timestamp: number): string {
    const now = Date.now();
    const diff = now - timestamp;

    if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return `${Math.floor(diff / 86400000)}d ago`;
  }

  function getSourceColor(source: string): string {
    switch (source) {
      case 'dashboard': return '#3b82f6'; // blue
      case 'system': return '#8b5cf6'; // purple
      case 'spawn': return '#22c55e'; // green
      case 'revert': return '#f59e0b'; // amber
      default: return '#6b7280'; // gray
    }
  }

  function getSourceLabel(source: string): string {
    switch (source) {
      case 'dashboard': return 'Dashboard';
      case 'system': return 'System';
      case 'spawn': return 'Spawned';
      case 'revert': return 'Reverted';
      default: return source;
    }
  }

  function toggleExpand(id: number) {
    expandedEntry = expandedEntry === id ? null : id;
  }

  async function handleRevert(entry: HistoryEntry) {
    if (!entry.old_value) {
      alert('Cannot revert: no previous value (component was added)');
      return;
    }

    if (!confirm(`Revert ${entry.component} to previous value?`)) return;

    reverting = entry.id;
    try {
      const result = await client.current.revertToEntry(entry.id);
      if (result.success) {
        onRevert?.(entry);
      } else {
        alert(result.error || 'Revert failed');
      }
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Revert failed');
    } finally {
      reverting = null;
    }
  }

  function formatValue(value: unknown): string {
    if (value === null || value === undefined) return '(none)';
    return JSON.stringify(value, null, 2);
  }
</script>

<div class="timeline">
  {#if entries.length === 0}
    <p class="empty">No history recorded yet</p>
  {:else}
    {#each entries as entry (entry.id)}
      <div class="entry" class:expanded={expandedEntry === entry.id}>
        <div class="entry-line">
          <div class="dot" style="background-color: {getSourceColor(entry.source)}"></div>
          <div class="connector"></div>
        </div>

        <div class="entry-content">
          <button class="entry-header" onclick={() => toggleExpand(entry.id)}>
            <div class="entry-meta">
              <span class="component-name">{entry.component}</span>
              <span class="source-badge" style="background-color: {getSourceColor(entry.source)}20; color: {getSourceColor(entry.source)}">
                {getSourceLabel(entry.source)}
              </span>
            </div>
            <div class="entry-time">
              <span class="relative-time">{formatRelativeTime(entry.timestamp)}</span>
              <span class="absolute-time">{formatTime(entry.timestamp)}</span>
            </div>
            <svg class="chevron" class:rotated={expandedEntry === entry.id} viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
          </button>

          {#if expandedEntry === entry.id}
            <div class="entry-details">
              <div class="diff">
                <div class="diff-side old">
                  <div class="diff-label">Before</div>
                  <pre class="diff-value">{formatValue(entry.old_value)}</pre>
                </div>
                <div class="diff-arrow">
                  <svg viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z" clip-rule="evenodd" />
                  </svg>
                </div>
                <div class="diff-side new">
                  <div class="diff-label">After</div>
                  <pre class="diff-value">{formatValue(entry.new_value)}</pre>
                </div>
              </div>

              {#if entry.old_value !== null}
                <button
                  class="revert-btn"
                  onclick={() => handleRevert(entry)}
                  disabled={reverting === entry.id}
                >
                  {#if reverting === entry.id}
                    Reverting...
                  {:else}
                    <svg viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                    </svg>
                    Revert to this state
                  {/if}
                </button>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .empty {
    color: #6b7280;
    font-style: italic;
    padding: 24px;
    text-align: center;
  }

  .entry {
    display: flex;
    gap: 16px;
  }

  .entry-line {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 20px;
    flex-shrink: 0;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    z-index: 1;
    flex-shrink: 0;
  }

  .connector {
    width: 2px;
    flex: 1;
    background: #e5e7eb;
    margin-top: -2px;
  }

  .entry:last-child .connector {
    display: none;
  }

  .entry-content {
    flex: 1;
    min-width: 0;
    padding-bottom: 16px;
  }

  .entry-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .entry-header:hover {
    background: #f3f4f6;
    border-color: #d1d5db;
  }

  .entry.expanded .entry-header {
    border-radius: 8px 8px 0 0;
    border-bottom-color: transparent;
  }

  .entry-meta {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .component-name {
    font-family: monospace;
    font-weight: 600;
    color: #111827;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .source-badge {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 9999px;
    font-weight: 500;
    white-space: nowrap;
  }

  .entry-time {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    flex-shrink: 0;
  }

  .relative-time {
    font-size: 0.875rem;
    color: #374151;
    font-weight: 500;
  }

  .absolute-time {
    font-size: 0.75rem;
    color: #9ca3af;
  }

  .chevron {
    width: 20px;
    height: 20px;
    color: #9ca3af;
    transition: transform 0.15s ease;
    flex-shrink: 0;
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  .entry-details {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-top: none;
    border-radius: 0 0 8px 8px;
    padding: 16px;
  }

  .diff {
    display: flex;
    gap: 12px;
    align-items: stretch;
  }

  .diff-side {
    flex: 1;
    min-width: 0;
  }

  .diff-label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 8px;
  }

  .diff-side.old .diff-label {
    color: #dc2626;
  }

  .diff-side.new .diff-label {
    color: #16a34a;
  }

  .diff-value {
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    padding: 12px;
    margin: 0;
    font-size: 0.8125rem;
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
  }

  .diff-side.old .diff-value {
    background: #fef2f2;
    border-color: #fecaca;
  }

  .diff-side.new .diff-value {
    background: #f0fdf4;
    border-color: #bbf7d0;
  }

  .diff-arrow {
    display: flex;
    align-items: center;
    padding: 0 8px;
    color: #9ca3af;
  }

  .diff-arrow svg {
    width: 24px;
    height: 24px;
  }

  .revert-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
    padding: 8px 16px;
    background: #fef3c7;
    border: 1px solid #fcd34d;
    border-radius: 6px;
    color: #92400e;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .revert-btn:hover:not(:disabled) {
    background: #fde68a;
  }

  .revert-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .revert-btn svg {
    width: 16px;
    height: 16px;
  }

  @media (max-width: 640px) {
    .diff {
      flex-direction: column;
    }

    .diff-arrow {
      transform: rotate(90deg);
      padding: 8px 0;
    }
  }
</style>
