<script lang="ts">
  import { api, inTauri } from '../lib/api';
  import { m } from '../lib/i18n';
  import { store } from '../lib/store.svelte';
  import Segmented from '../lib/components/Segmented.svelte';
  import Toggle from '../lib/components/Toggle.svelte';
  import Icon from '../lib/components/Icon.svelte';

  const themeOptions = $derived([
    { value: 'light', label: m.settings.themeLight },
    { value: 'dark', label: m.settings.themeDark },
    { value: 'system', label: m.settings.themeSystem }
  ]);

  const scaleOptions = ['90', '100', '110', '125'].map((value) => ({
    value,
    label: `${value}%`
  }));

  const idleOptions = $derived([
    { value: '0', label: m.settings.idleOff },
    { value: '5', label: m.settings.idleMinutes(5) },
    { value: '10', label: m.settings.idleMinutes(10) },
    { value: '15', label: m.settings.idleMinutes(15) }
  ]);

  const dayStartOptions = Array.from({ length: 13 }, (_, hour) => ({
    value: String(hour),
    label: m.clock(hour, 0)
  }));

  let shortcut = $state('');
  let savedShortcut = '';

  $effect(() => {
    const value = store.settings.globalShortcut;
    if (value !== savedShortcut) {
      savedShortcut = value;
      shortcut = value;
    }
  });

  async function commitShortcut() {
    const value = shortcut.trim();
    if (!value || value === store.settings.globalShortcut) return;
    await store.updateSetting('globalShortcut', value);
  }

  async function exportAs(format: 'json' | 'csv') {
    await store.guard(async () => {
      if (!inTauri) {
        store.showToast({ text: m.settings.notSupported, actions: [] }, 5000);
        return;
      }
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        defaultPath: `studytracker.${format}`,
        filters: [{ name: 'StudyTracker', extensions: [format] }]
      });
      if (!path) return;
      const written = await api.exportData(format, path);
      store.showToast({ text: m.toast.exported(written), actions: [] });
    });
  }

  async function importFile() {
    await store.guard(async () => {
      if (!inTauri) {
        store.showToast({ text: m.settings.notSupported, actions: [] }, 5000);
        return;
      }
      const { open } = await import('@tauri-apps/plugin-dialog');
      const path = await open({
        multiple: false,
        filters: [{ name: 'StudyTracker', extensions: ['json', 'csv'] }]
      });
      if (!path || Array.isArray(path)) return;
      const report = await api.importData(path);
      await store.refreshAll();
      store.showToast({ text: m.toast.imported(report.sessionsAdded), actions: [] });
    });
  }

  async function openDataDir() {
    await store.guard(async () => {
      if (!inTauri) {
        store.showToast({ text: m.settings.notSupported, actions: [] }, 5000);
        return;
      }
      await api.openDataDir();
    });
  }

  async function backup() {
    await store.guard(async () => {
      if (!inTauri) {
        store.showToast({ text: m.settings.notSupported, actions: [] }, 5000);
        return;
      }
      const path = await api.backupNow();
      store.showToast({ text: m.toast.backedUp(path), actions: [] });
    });
  }
</script>

<div class="view">
  <h1>{m.settings.title}</h1>

  <section class="block">
    <h2>{m.settings.appearance}</h2>

    <div class="row">
      <span class="label">{m.settings.theme}</span>
      <Segmented
        options={themeOptions}
        value={store.settings.theme}
        label={m.settings.theme}
        onchange={(value) => store.updateSetting('theme', value)}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.scale}</span>
      <Segmented
        options={scaleOptions}
        value={String(store.settings.uiScale)}
        label={m.settings.scale}
        onchange={(value) => store.updateSetting('uiScale', Number(value))}
      />
    </div>
  </section>

  <section class="block">
    <h2>{m.settings.timer}</h2>

    <div class="row">
      <div class="text">
        <span class="label">{m.settings.dayStart}</span>
        <span class="hint">{m.settings.dayStartHint}</span>
      </div>
      <select
        value={String(store.settings.dayStartHour)}
        onchange={(event) =>
          store.updateSetting('dayStartHour', Number(event.currentTarget.value))}
        aria-label={m.settings.dayStart}
      >
        {#each dayStartOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>

    <div class="row">
      <span class="label">{m.settings.idle}</span>
      <Segmented
        options={idleOptions}
        value={String(store.settings.idleThresholdMin)}
        label={m.settings.idle}
        onchange={(value) => store.updateSetting('idleThresholdMin', Number(value))}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.sleep}</span>
      <Toggle
        checked={store.settings.countSleepTime}
        label={m.settings.sleep}
        onchange={(value) => store.updateSetting('countSleepTime', value)}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.globalShortcut}</span>
      <input
        class="shortcut"
        type="text"
        bind:value={shortcut}
        onblur={commitShortcut}
        onkeydown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            event.currentTarget.blur();
          }
        }}
        aria-label={m.settings.globalShortcut}
      />
    </div>
  </section>

  <section class="block">
    <h2>{m.settings.window}</h2>

    <div class="row">
      <span class="label">{m.settings.closeToTray}</span>
      <Toggle
        checked={store.settings.closeToTray}
        label={m.settings.closeToTray}
        onchange={(value) => store.updateSetting('closeToTray', value)}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.autostart}</span>
      <Toggle
        checked={store.settings.autostart}
        label={m.settings.autostart}
        onchange={(value) => store.updateSetting('autostart', value)}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.rememberWindow}</span>
      <Toggle
        checked={store.settings.rememberWindow}
        label={m.settings.rememberWindow}
        onchange={(value) => store.updateSetting('rememberWindow', value)}
      />
    </div>

    <div class="row">
      <span class="label">{m.settings.miniOnTop}</span>
      <Toggle
        checked={store.settings.miniAlwaysOnTop}
        label={m.settings.miniOnTop}
        onchange={(value) => store.setMiniOnTop(value)}
      />
    </div>
  </section>

  <section class="block">
    <h2>{m.settings.data}</h2>

    {#if store.preview}
      <p class="hint">{m.app.previewBanner}</p>
    {/if}

    <div class="actions">
      <button type="button" class="secondary" onclick={() => exportAs('csv')}>
        <Icon name="download" size={16} />
        {m.settings.exportCsv}
      </button>
      <button type="button" class="secondary" onclick={() => exportAs('json')}>
        <Icon name="download" size={16} />
        {m.settings.exportJson}
      </button>
      <button type="button" class="secondary" onclick={importFile}>
        <Icon name="upload" size={16} />
        {m.settings.import}
      </button>
      <button type="button" class="secondary" onclick={openDataDir}>
        <Icon name="folder" size={16} />
        {m.settings.openDir}
      </button>
      <button type="button" class="secondary" onclick={backup}>
        <Icon name="archive" size={16} />
        {m.settings.backup}
      </button>
    </div>
  </section>

  <section class="block">
    <h2>{m.settings.about}</h2>
    <dl class="about">
      <div>
        <dt>{m.settings.title}</dt>
        <dd>{store.info ? m.settings.version(store.info.version) : m.settings.version('0.1.0')}</dd>
      </div>
      <div>
        <dt>{m.settings.database}</dt>
        <dd class="path">{store.info?.databasePath ?? '—'}</dd>
      </div>
    </dl>
  </section>
</div>

<style>
  .view {
    display: grid;
    gap: var(--sp-6);
    align-content: start;
    max-width: 760px;
  }

  h1 {
    font-size: 22px;
  }

  .block {
    display: grid;
    gap: var(--sp-2);
  }

  h2 {
    font-size: 15px;
    color: var(--mauve);
    font-family: var(--font-body);
    font-weight: 600;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--sp-3);
    min-height: 52px;
    padding: var(--sp-2) 0;
    border-bottom: 1px solid var(--petal-strong);
  }

  .row:last-child {
    border-bottom: 0;
  }

  .text {
    display: grid;
    gap: 2px;
  }

  .label {
    font-size: 15px;
  }

  .hint {
    font-size: 12px;
    color: var(--mauve);
    max-width: 46ch;
  }

  select,
  .shortcut {
    height: 40px;
    padding: 0 var(--sp-3);
    background: var(--paper);
    border: 1.5px solid var(--line);
    border-radius: var(--r-md);
    color: var(--plum);
    font-size: 14px;
  }

  .shortcut {
    width: 180px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    padding-top: var(--sp-2);
  }

  .secondary {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 40px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-md);
    background: var(--petal-strong);
    color: var(--plum);
    font-size: 14px;
    font-weight: 600;
  }

  .secondary:hover {
    background: var(--line);
  }

  .about {
    margin: 0;
    display: grid;
    gap: var(--sp-2);
    font-size: 14px;
  }

  .about div {
    display: flex;
    gap: var(--sp-3);
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .about dt {
    color: var(--mauve);
  }

  .about dd {
    margin: 0;
  }

  .path {
    font-family: ui-monospace, 'Cascadia Mono', monospace;
    font-size: 12px;
    overflow-wrap: anywhere;
  }
</style>
