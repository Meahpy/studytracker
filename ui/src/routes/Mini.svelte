<script lang="ts">
  import { m } from '../lib/i18n';
  import { fmtClock } from '../lib/format';
  import { store } from '../lib/store.svelte';
  import Icon from '../lib/components/Icon.svelte';

  const phase = $derived(store.timer.phase);
  const actionLabel = $derived(
    phase === 'running' ? m.timer.pause : phase === 'paused' ? m.timer.resume : m.timer.start
  );
</script>

<div
  class="mini"
  class:running={phase === 'running'}
  class:paused={phase === 'paused'}
  data-tauri-drag-region
>
  <header data-tauri-drag-region>
    <span class="subject" data-tauri-drag-region>
      {store.timer.subjectName ?? m.timer.title}
    </span>
    <div class="tools">
      <button
        type="button"
        class="tool"
        class:active={store.settings.miniAlwaysOnTop}
        aria-label={m.mini.pin}
        aria-pressed={store.settings.miniAlwaysOnTop}
        onclick={() => store.setMiniOnTop(!store.settings.miniAlwaysOnTop)}
      >
        <Icon name="pin" size={14} />
      </button>
      <button type="button" class="tool" aria-label={m.mini.back} onclick={() => store.backToMain()}>
        <Icon name="expand" size={14} />
      </button>
    </div>
  </header>

  <div class="clock tabular">{fmtClock(store.elapsedSec)}</div>

  <button type="button" class="action" disabled={store.busy} onclick={() => store.toggle()}>
    <Icon name={phase === 'running' ? 'pause' : 'play'} size={16} />
    {actionLabel}
  </button>
</div>

<style>
  .mini {
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr auto;
    gap: var(--sp-2);
    padding: var(--sp-3);
    background: var(--petal);
    color: var(--plum);
    border-radius: var(--r-lg);
    transition: background-color var(--dur-color) var(--ease);
  }

  .mini.running {
    background: var(--raspberry);
    color: var(--paper);
  }

  .mini.paused {
    background: var(--paused);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
  }

  .subject {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tools {
    display: flex;
    gap: 2px;
  }

  .tool {
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    border-radius: var(--r-sm);
    color: inherit;
    opacity: 0.7;
  }

  .tool:hover,
  .tool.active {
    opacity: 1;
    background: color-mix(in srgb, currentColor 14%, transparent);
  }

  .clock {
    align-self: center;
    text-align: center;
    font-family: var(--font-display);
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }

  .action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    height: 38px;
    border-radius: var(--r-pill);
    background: var(--raspberry);
    color: var(--paper);
    font-size: 14px;
    font-weight: 600;
  }

  .mini.running .action {
    background: var(--paper);
    color: var(--raspberry);
  }

  .mini.paused .action {
    background: var(--plum);
    color: var(--paused);
  }

  .action:active {
    transform: scale(0.985);
  }
</style>
