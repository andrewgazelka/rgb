<script lang="ts">
  import { page } from '$app/stores';

  const navItems = [
    { href: '/', label: 'Dashboard' },
    { href: '/world', label: 'World' },
    { href: '/map', label: 'Map' },
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
      </a>
    {/each}
  </div>

  <div class="status">
    <span class="dot"></span>
    Connected
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
</style>
