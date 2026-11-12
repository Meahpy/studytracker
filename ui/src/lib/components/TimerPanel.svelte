<script lang="ts">
  import { store } from '../store.svelte';
  import { m } from '../i18n';
  import { fmtClock, fmtLong, fmtTime } from '../format';
  import Icon from './Icon.svelte';

  const phase = $derived(store.timer.phase);
  const subject = $derived(store.timer.subjectName ?? '');
  const remaining = $derived(store.remainingToTarget);
</script>

<section
  class="panel"
  class:running={phase === 'running'}
  class:paused={phase === 'paused'}
  aria-label={m.timer.title}
>
  <div class="head">
    {#if subject}
      <span class="subject">{subject}</span>
    {/if}
  </div>

  <div class="clock tabular" aria-live={phase === 'running' ? 'off' : 'polite'}>
    {fmtClock(store.elapsedSec)}
  </div>

  <div class="meta">
    {#if store.timer.startedAt}
      <span>{m.timer.startedAt(fmtTime(store.timer.startedAt))}</span>
    {:else}
      <span>{m.timer.idle}</span>
    {/if}
    {#if store.goals.dailySec > 0}
      <span class="target">
        {remaining > 0
          ? m.timer.targetLeft(fmtLong(remaining))
          : m.timer.targetReached}
      </span>
    {/if}
  </div>

  <div class="actions">
    {#if phase === 'idle'}
      <button
        type="button"
        class="primary"
        disabled={store.busy || store.subjects.length === 0}
        onclick={() => {
          const first = store.activeSubject ?? store.subjects[0];
          if (first) void store.start(first.id);
        }}
      >
        <Icon name="play" size={20} />
        {m.timer.start}
      </button>
    {:else}
      <button type="button" class="primary" disabled={store.busy} onclick={() => store.stop()}>
        <Icon name="stop" size={18} />
        {m.timer.stop}
      </button>
      {#if phase === 'running'}
        <button type="button" class="ghost" disabled={store.busy} onclick={() => store.pause()}>
          <Icon name="pause" size={18} />
          {m.timer.pause}
        </button>
      {:else}
        <button type="button" class="ghost resume" disabled={store.busy} onclick={() => store.resume()}>
          <Icon name="play" size={18} />
          {m.timer.resume}
        </button>
      {/if}
    {/if}
  </div>
</section>

<style>
  .panel {
    container-type: inline-size;
    display: grid;
    justify-items: center;
    align-content: center;
    gap: var(--sp-3);
    padding: var(--sp-8) var(--sp-6) var(--sp-6);
    border-radius: var(--r-xl);
    background: var(--petal);
    transition: background-color var(--dur-color) var(--ease);
  }

  /* Di layar lebar, panel timer jadi panggung: isi di tengah, bukan menempel atas. */
  @media (min-width: 760px) {
    .panel {
      min-height: clamp(320px, 62vh, 560px);
    }
  }

  .panel.running {
    background: var(--raspberry);
    color: var(--paper);
  }

  .panel.paused {
    background: var(--paused);
  }

  .head {
    display: grid;
    justify-items: center;
    gap: var(--sp-1);
    min-height: 24px;
    text-align: center;
  }

  .head .subject {
    font-family: var(--font-display);
    font-size: 16px;
    font-weight: 600;
  }

  .clock {
    font-family: var(--font-display);
    font-weight: 700;
    font-size: clamp(56px, 14cqi, 144px);
    line-height: 1;
    letter-spacing: -0.02em;
    text-align: center;
  }

  .meta {
    display: grid;
    justify-items: center;
    gap: 2px;
    font-size: 14px;
    opacity: 0.85;
    text-align: center;
    min-height: 40px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--sp-3);
    margin-top: var(--sp-2);
  }

  .primary {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 64px;
    padding: 0 var(--sp-8);
    border-radius: var(--r-pill);
    background: var(--raspberry);
    color: var(--paper);
    font-size: 16px;
    font-weight: 600;
    transition: background-color var(--dur-color) var(--ease);
  }

  .primary:hover:not(:disabled) {
    background: var(--raspberry-press);
  }

  .primary:active:not(:disabled) {
    background: var(--raspberry-press);
  }

  .primary:disabled {
    opacity: 0.55;
  }

  /* Di atas panel berjalan, tombol utama berbalik menjadi putih. */
  .panel.running .primary {
    background: var(--paper);
    color: var(--raspberry);
  }

  .panel.running .primary:hover:not(:disabled) {
    background: var(--petal);
  }

  .ghost {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 64px;
    padding: 0 var(--sp-6);
    border-radius: var(--r-pill);
    border: 1.5px solid currentColor;
    color: inherit;
    font-size: 16px;
    font-weight: 600;
  }

  .ghost:hover:not(:disabled) {
    background: color-mix(in srgb, currentColor 12%, transparent);
  }

  .panel.paused .ghost.resume {
    background: var(--plum);
    border-color: var(--plum);
    color: var(--paused);
  }

  .panel.paused .ghost.resume:hover:not(:disabled) {
    background: color-mix(in srgb, var(--plum) 88%, black);
  }

  @container (max-width: 420px) {
    .panel {
      padding: var(--sp-6) var(--sp-4);
    }

    .primary,
    .ghost {
      height: 56px;
      padding: 0 var(--sp-4);
      font-size: 15px;
    }
  }
</style>
