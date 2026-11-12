<script lang="ts">
  import { onMount } from 'svelte';
  import { m } from './lib/i18n';
  import { store } from './lib/store.svelte';
  import type { View } from './lib/store.svelte';
  import Dialogs from './lib/components/Dialogs.svelte';
  import Icon from './lib/components/Icon.svelte';
  import Toast from './lib/components/Toast.svelte';
  import TodayPanel from './lib/components/TodayPanel.svelte';
  import History from './routes/History.svelte';
  import Mini from './routes/Mini.svelte';
  import Settings from './routes/Settings.svelte';
  import Stats from './routes/Stats.svelte';
  import Subjects from './routes/Subjects.svelte';
  import Targets from './routes/Targets.svelte';
  import Timer from './routes/Timer.svelte';

  const primary: Array<{ id: View; label: string; icon: string }> = [
    { id: 'timer', label: m.nav.timer, icon: 'timer' },
    { id: 'history', label: m.nav.history, icon: 'history' },
    { id: 'stats', label: m.nav.stats, icon: 'stats' },
    { id: 'targets', label: m.nav.targets, icon: 'target' },
    { id: 'subjects', label: m.nav.subjects, icon: 'subjects' }
  ];

  const tabs = primary.slice(0, 4);
  const order: View[] = ['timer', 'history', 'stats', 'targets', 'subjects', 'settings'];

  let moreOpen = $state(false);

  function go(view: View) {
    store.view = view;
    moreOpen = false;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      if (store.dialog) store.closeDialog();
      else moreOpen = false;
      return;
    }

    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
      event.preventDefault();
      if (store.timer.phase !== 'idle') void store.stop();
      return;
    }

    if ((event.ctrlKey || event.metaKey) && /^[1-6]$/.test(event.key)) {
      event.preventDefault();
      const view = order[Number(event.key) - 1];
      if (view) go(view);
      return;
    }

    if (event.key === ' ' && !event.ctrlKey && !event.metaKey) {
      const target = event.target as HTMLElement | null;
      const typing =
        !!target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'SELECT' ||
          target.tagName === 'TEXTAREA' ||
          target.isContentEditable);
      if (typing || store.dialog) return;
      event.preventDefault();
      void store.toggle();
    }
  }

  onMount(() => {
    void store.init();
    const tick = window.setInterval(() => {
      store.nowMs = Date.now();
    }, 250);
    window.addEventListener('keydown', onKeydown);
    return () => {
      window.clearInterval(tick);
      window.removeEventListener('keydown', onKeydown);
    };
  });

  $effect(() => {
    const mode = store.settings.theme;
    const query = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = () => {
      const dark = mode === 'dark' || (mode === 'system' && query.matches);
      document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    };
    apply();
    if (mode !== 'system') return;
    query.addEventListener('change', apply);
    return () => query.removeEventListener('change', apply);
  });

  $effect(() => {
    document.documentElement.style.setProperty(
      '--ui-scale',
      String(store.settings.uiScale / 100)
    );
  });
</script>

{#if store.isMini}
  <Mini />
{:else}
  <div class="shell">
    <nav class="rail" aria-label={m.nav.primary}>
      <div class="brand">
        <span class="mark" aria-hidden="true"></span>
        <span class="brand-name">{m.app.name}</span>
      </div>

      <div class="rail-items">
        {#each primary as item (item.id)}
          <button
            type="button"
            class="rail-item"
            class:active={store.view === item.id}
            aria-current={store.view === item.id ? 'page' : undefined}
            title={item.label}
            onclick={() => go(item.id)}
          >
            <Icon name={item.icon} size={20} />
            <span class="rail-label">{item.label}</span>
          </button>
        {/each}
      </div>

      <div class="rail-items bottom">
        <button
          type="button"
          class="rail-item"
          class:active={store.view === 'settings'}
          aria-current={store.view === 'settings' ? 'page' : undefined}
          title={m.nav.settings}
          onclick={() => go('settings')}
        >
          <Icon name="settings" size={20} />
          <span class="rail-label">{m.nav.settings}</span>
        </button>
      </div>
    </nav>

    <div class="scroll">
      <div class="content" class:has-aside={store.view === 'timer'}>
        <main id="content">
          {#if store.view === 'timer'}
            <Timer />
          {:else if store.view === 'history'}
            <History />
          {:else if store.view === 'stats'}
            <Stats />
          {:else if store.view === 'targets'}
            <Targets />
          {:else if store.view === 'subjects'}
            <Subjects />
          {:else}
            <Settings />
          {/if}
        </main>

        {#if store.view === 'timer'}
          <aside>
            <TodayPanel compact />
          </aside>
        {/if}
      </div>
    </div>

    <nav class="tabs" aria-label={m.nav.primary}>
      {#each tabs as item (item.id)}
        <button
          type="button"
          class="tab"
          class:active={store.view === item.id}
          aria-current={store.view === item.id ? 'page' : undefined}
          onclick={() => go(item.id)}
        >
          <Icon name={item.icon} size={20} />
          <span>{item.label}</span>
        </button>
      {/each}

      <div class="tab-more">
        <button
          type="button"
          class="tab"
          class:active={store.view === 'subjects' || store.view === 'settings'}
          aria-expanded={moreOpen}
          onclick={() => (moreOpen = !moreOpen)}
        >
          <Icon name="more" size={20} />
          <span>{m.nav.more}</span>
        </button>

        {#if moreOpen}
          <div class="sheet" role="menu">
            <button type="button" role="menuitem" onclick={() => go('subjects')}>
              <Icon name="subjects" size={18} />
              {m.nav.subjects}
            </button>
            <button type="button" role="menuitem" onclick={() => go('settings')}>
              <Icon name="settings" size={18} />
              {m.nav.settings}
            </button>
          </div>
        {/if}
      </div>
    </nav>
  </div>

  <Dialogs />
  <Toast />
{/if}

<style>
  .shell {
    height: 100%;
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    background: var(--blush);
  }

  .rail {
    display: flex;
    flex-direction: column;
    gap: var(--sp-6);
    padding: var(--sp-6) var(--sp-3);
    background: var(--petal);
    min-height: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 0 var(--sp-2);
  }

  .mark {
    width: 18px;
    height: 18px;
    border-radius: 6px;
    background: var(--raspberry);
  }

  .brand-name {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 15px;
    letter-spacing: -0.01em;
    white-space: nowrap;
  }

  .rail-items {
    display: grid;
    gap: var(--sp-1);
  }

  .rail-items.bottom {
    margin-top: auto;
  }

  .rail-item {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    height: 44px;
    padding: 0 var(--sp-3);
    border-radius: var(--r-pill);
    color: var(--plum);
    font-size: 15px;
    font-weight: 500;
    text-align: left;
  }

  .rail-item:hover {
    background: var(--petal-strong);
  }

  .rail-item.active {
    background: var(--petal-strong);
    font-weight: 600;
  }

  .rail-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .scroll {
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    scrollbar-gutter: stable;
  }

  .content {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--sp-6);
    align-items: start;
    padding: var(--sp-8);
    max-width: 1320px;
    margin: 0 auto;
  }

  .content.has-aside {
    grid-template-columns: minmax(0, 1fr) 320px;
  }

  main {
    min-width: 0;
  }

  aside {
    position: sticky;
    top: var(--sp-8);
    min-width: 0;
  }

  .tabs {
    display: none;
  }

  .tab-more {
    position: relative;
  }

  .sheet {
    position: absolute;
    right: 0;
    bottom: calc(100% + 8px);
    display: grid;
    gap: var(--sp-1);
    padding: var(--sp-2);
    min-width: 180px;
    background: var(--blush);
    border: 1.5px solid var(--line);
    border-radius: var(--r-md);
    z-index: 20;
  }

  .sheet button {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    height: 44px;
    padding: 0 var(--sp-3);
    border-radius: var(--r-sm);
    font-size: 14px;
    font-weight: 500;
    text-align: left;
  }

  .sheet button:hover {
    background: var(--petal-strong);
  }

  @media (max-width: 1099px) {
    .shell {
      grid-template-columns: 72px minmax(0, 1fr);
    }

    .rail {
      padding: var(--sp-4) var(--sp-2);
      align-items: center;
    }

    .brand {
      padding: 0;
    }

    .brand-name,
    .rail-label {
      display: none;
    }

    .rail-item {
      justify-content: center;
      width: 48px;
      padding: 0;
    }

    .rail-items {
      width: 100%;
      justify-items: center;
    }

    .content,
    .content.has-aside {
      grid-template-columns: minmax(0, 1fr);
      padding: var(--sp-6);
    }

    aside {
      position: static;
    }
  }

  @media (max-width: 759px) {
    .shell {
      grid-template-columns: minmax(0, 1fr);
      grid-template-rows: minmax(0, 1fr) auto;
    }

    .rail {
      display: none;
    }

    .content {
      padding: var(--sp-4);
      gap: var(--sp-4);
    }

    .tabs {
      display: grid;
      grid-template-columns: repeat(5, minmax(0, 1fr));
      border-top: 1.5px solid var(--line);
      background: var(--petal);
      padding: var(--sp-1) var(--sp-2);
    }

    .tab {
      display: grid;
      justify-items: center;
      gap: 2px;
      padding: var(--sp-2) 0;
      border-radius: var(--r-sm);
      font-size: 11px;
      color: var(--mauve);
    }

    .tab.active {
      color: var(--raspberry);
      font-weight: 600;
    }

    .tab-more {
      display: grid;
    }
  }
</style>
