<script lang="ts">
  import { store } from '../store.svelte';
  import { m } from '../i18n';
  import { fmtLong } from '../format';
  import ProgressBar from './ProgressBar.svelte';
  import SessionRow from './SessionRow.svelte';
  import { fmtPercent } from '../format';

  let { compact = false }: { compact?: boolean } = $props();

  const percent = $derived(fmtPercent(store.today.totalSec, store.goals.dailySec));
</script>

<section class="today" class:compact>
  <header>
    <h2>{m.today.heading}</h2>
    <span class="week">{m.today.week(fmtLong(store.weekSec))}</span>
  </header>

  <p class="total" class:quiet={store.today.totalSec === 0}>
    {m.today.total(fmtLong(store.today.totalSec))}
  </p>

  <div class="goal">
    <ProgressBar
      value={store.progress}
      tone={store.targetReached ? 'ok' : 'raspberry'}
      label={m.a11y.progress(percent)}
    />
    <span class="goal-label tabular">
      {m.today.goalProgress(percent, fmtLong(store.goals.dailySec))}
    </span>
  </div>

  <div class="sessions">
    {#if store.today.sessions.length === 0}
      {#if store.today.totalSec > 0}
        <p class="empty">{m.today.empty}</p>
      {/if}
    {:else}
      <span class="list-title">{m.today.sessions}</span>
      {#each store.today.sessions as session (session.id)}
        <SessionRow {session} onedit={(item) => store.openSession(store.today.date, item)} />
      {/each}
    {/if}
  </div>
</section>

<style>
  .today {
    container-type: inline-size;
    display: grid;
    gap: var(--sp-4);
    padding: var(--sp-6);
    border-radius: var(--r-lg);
    background: var(--petal);
    align-content: start;
  }

  .today.compact {
    padding: var(--sp-4);
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--sp-3);
  }

  h2 {
    font-size: 16px;
    font-weight: 600;
  }

  .week {
    font-size: 12px;
    color: var(--mauve);
  }

  .total {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 600;
    line-height: 1.15;
  }

  .total.quiet {
    font-family: var(--font-body);
    font-size: 14px;
    font-weight: 400;
    color: var(--mauve);
  }

  .goal {
    display: grid;
    gap: var(--sp-2);
  }

  .goal-label {
    font-size: 12px;
    color: var(--mauve);
  }

  .sessions {
    display: grid;
    gap: var(--sp-1);
  }

  .list-title {
    font-size: 12px;
    color: var(--mauve);
    margin-bottom: var(--sp-1);
  }

  .empty {
    font-size: 14px;
    color: var(--mauve);
  }
</style>
