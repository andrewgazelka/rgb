<script lang="ts">
  import { page } from '$app/stores';

  const navItems = [
    { href: '/', label: 'Dashboard', key: 'g d' },
    { href: '/world', label: 'World', key: 'g w' },
    { href: '/map', label: 'Map', key: 'g m' },
  ];

  function isActive(href: string, pathname: string): boolean {
    if (href === '/') return pathname === '/';
    return pathname.startsWith(href);
  }
</script>

<nav class="navbar">
  <a href="/" class="brand">RGB</a>

  <div class="nav-links">
    {#each navItems as item}
      <a
        href={item.href}
        class="nav-link"
        class:active={isActive(item.href, $page.url.pathname)}
      >
        {item.label}
        <kbd class="keyhint">{item.key}</kbd>
      </a>
    {/each}
  </div>

  <div class="right">
    <kbd class="help-hint" title="Keyboard shortcuts">?</kbd>
    <div class="status">
      <span class="dot"></span>
      Connected
    </div>
  </div>
</nav>

<style>
  .navbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    height: 48px;
    position: sticky;
    top: 0;
    z-index: 100;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  }

  @media (prefers-color-scheme: dark) {
    .navbar {
      background: rgba(10, 10, 15, 0.8);
      border-bottom-color: rgba(255, 255, 255, 0.06);
    }
  }

  .brand {
    font-weight: 600;
    font-size: 14px;
    color: inherit;
    text-decoration: none;
    letter-spacing: -0.01em;
  }

  .nav-links {
    display: flex;
    gap: 2px;
  }

  .nav-link {
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.5);
    text-decoration: none;
    border-radius: 6px;
    transition: color 0.1s, background 0.1s;
  }

  .nav-link:hover {
    color: rgba(0, 0, 0, 0.8);
    background: rgba(0, 0, 0, 0.04);
  }

  .nav-link.active {
    color: rgba(0, 0, 0, 0.9);
    background: rgba(0, 0, 0, 0.06);
  }

  @media (prefers-color-scheme: dark) {
    .nav-link {
      color: rgba(255, 255, 255, 0.5);
    }
    .nav-link:hover {
      color: rgba(255, 255, 255, 0.8);
      background: rgba(255, 255, 255, 0.06);
    }
    .nav-link.active {
      color: rgba(255, 255, 255, 0.9);
      background: rgba(255, 255, 255, 0.08);
    }
  }

  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: rgba(0, 0, 0, 0.4);
  }

  @media (prefers-color-scheme: dark) {
    .status {
      color: rgba(255, 255, 255, 0.4);
    }
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #34c759;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .keyhint {
    font-family: ui-monospace, monospace;
    font-size: 10px;
    padding: 2px 4px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 3px;
    color: rgba(0, 0, 0, 0.4);
    margin-left: 6px;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .nav-link:hover .keyhint {
    opacity: 1;
  }

  @media (prefers-color-scheme: dark) {
    .keyhint {
      background: rgba(255, 255, 255, 0.08);
      color: rgba(255, 255, 255, 0.4);
    }
  }

  .help-hint {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.04);
    border-radius: 4px;
    color: rgba(0, 0, 0, 0.4);
    cursor: help;
    transition: background 0.1s, color 0.1s;
  }

  .help-hint:hover {
    background: rgba(0, 0, 0, 0.08);
    color: rgba(0, 0, 0, 0.6);
  }

  @media (prefers-color-scheme: dark) {
    .help-hint {
      background: rgba(255, 255, 255, 0.06);
      color: rgba(255, 255, 255, 0.4);
    }
    .help-hint:hover {
      background: rgba(255, 255, 255, 0.1);
      color: rgba(255, 255, 255, 0.6);
    }
  }
</style>
