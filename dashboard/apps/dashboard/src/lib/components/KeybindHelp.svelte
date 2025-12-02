<script lang="ts">
  import { getState, closeHelp, getKeybindsByCategory } from '$lib/keybinds.svelte';

  const state = getState();
  const keybindsByCategory = getKeybindsByCategory();

  const categoryLabels: Record<string, string> = {
    navigation: 'Navigation',
    view: 'View',
    action: 'Actions',
    map: 'Map/World View',
  };
</script>

{#if state.showHelp}
  <div
    class="overlay"
    onclick={closeHelp}
    onkeydown={(e) => e.key === 'Escape' && closeHelp()}
    role="dialog"
    aria-modal="true"
    aria-label="Keyboard shortcuts"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
    <div class="modal" onclick={(e) => e.stopPropagation()} role="document">
      <div class="header">
        <h2>Keyboard Shortcuts</h2>
        <kbd class="close-hint">Esc</kbd>
      </div>

      <div class="content">
        {#each Object.entries(keybindsByCategory) as [category, binds]}
          <div class="category">
            <h3>{categoryLabels[category] ?? category}</h3>
            <div class="bindings">
              {#each binds as bind}
                <div class="binding">
                  <kbd class="key">{bind.key}</kbd>
                  <span class="desc">{bind.description}</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      <div class="footer">
        <span class="hint">Press <kbd>?</kbd> to toggle this help</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #fff;
    border-radius: 12px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    max-width: 480px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  @media (prefers-color-scheme: dark) {
    .modal {
      background: #1a1a24;
      border: 1px solid rgba(255, 255, 255, 0.1);
    }
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  }

  @media (prefers-color-scheme: dark) {
    .header {
      border-bottom-color: rgba(255, 255, 255, 0.08);
    }
  }

  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .close-hint {
    font-size: 11px;
    padding: 3px 6px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 4px;
    color: rgba(0, 0, 0, 0.5);
  }

  @media (prefers-color-scheme: dark) {
    .close-hint {
      background: rgba(255, 255, 255, 0.08);
      color: rgba(255, 255, 255, 0.5);
    }
  }

  .content {
    padding: 20px 24px;
    overflow-y: auto;
  }

  .category {
    margin-bottom: 20px;
  }

  .category:last-child {
    margin-bottom: 0;
  }

  h3 {
    margin: 0 0 12px 0;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: rgba(0, 0, 0, 0.4);
  }

  @media (prefers-color-scheme: dark) {
    h3 {
      color: rgba(255, 255, 255, 0.4);
    }
  }

  .bindings {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .binding {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .key {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 4px;
    min-width: 48px;
    text-align: center;
    color: rgba(0, 0, 0, 0.8);
  }

  @media (prefers-color-scheme: dark) {
    .key {
      background: rgba(255, 255, 255, 0.1);
      color: rgba(255, 255, 255, 0.9);
    }
  }

  .desc {
    font-size: 13px;
    color: rgba(0, 0, 0, 0.7);
  }

  @media (prefers-color-scheme: dark) {
    .desc {
      color: rgba(255, 255, 255, 0.7);
    }
  }

  .footer {
    padding: 16px 24px;
    border-top: 1px solid rgba(0, 0, 0, 0.08);
    text-align: center;
  }

  @media (prefers-color-scheme: dark) {
    .footer {
      border-top-color: rgba(255, 255, 255, 0.08);
    }
  }

  .hint {
    font-size: 12px;
    color: rgba(0, 0, 0, 0.4);
  }

  @media (prefers-color-scheme: dark) {
    .hint {
      color: rgba(255, 255, 255, 0.4);
    }
  }

  .hint kbd {
    font-family: ui-monospace, monospace;
    font-size: 11px;
    padding: 2px 5px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 3px;
  }

  @media (prefers-color-scheme: dark) {
    .hint kbd {
      background: rgba(255, 255, 255, 0.1);
    }
  }
</style>
