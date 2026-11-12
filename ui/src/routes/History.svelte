<script lang="ts">
  import { api } from '../lib/api';
  import { m } from '../lib/i18n';
  import { dayKey, fmtDayTitle, fmtLong } from '../lib/format';
  import { store } from '../lib/store.svelte';
  import type { DaySummary } from '../lib/types';
  import Calendar from '../lib/components/Calendar.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import SessionRow from '../lib/components/SessionRow.svelte';

  const pad = (n: number) => String(n).padStart(2, '0');
  const today = dayKey(new Date());

  let month = $state(today.slice(0, 7));
  let selected = $state(today);
  let totals = $state<Record<string, number>>({});
  let summary = $state<DaySummary | null>(null);

  $effect(() => {
    const current = month;
    const year = Number(current.slice(0, 4));
    const index = Number(current.slice(5, 7));
    const lastDay = new Date(year, index, 0).getDate();
    const from = `${current}-01`;
    const to = `${current}-${pad(lastDay)}`;
    let cancelled = false;
    api
      .dayTotals(from, to)
      .then((rows) => {
        if (cancelled) return;
        const map: Record<string, number> = {};
        for (const row of rows) map[row.date] = row.totalSec;
        totals = map;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    const date = selected;
    const revision = store.revision;
    void revision;
    let cancelled = false;
    api
      .daySummary(date)
      .then((result) => {
        if (!cancelled) summary = result;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  });

  function shiftMonth(months: number) {
    const year = Number(month.slice(0, 4));
    const index = Number(month.slice(5, 7)) - 1 + months;
    const anchor = new Date(year, index, 1);
    month = `${anchor.getFullYear()}-${pad(anchor.getMonth() + 1)}`;
    if (selected.slice(0, 7) !== month) {
      const lastDay = new Date(anchor.getFullYear(), anchor.getMonth() + 1, 0).getDate();
      selected = `${month}-${pad(Math.min(Number(selected.slice(8, 10)), lastDay))}`;
    }
  }

  const title = $derived(
    summary
      ? m.history.totalOn(fmtDayTitle(selected), fmtLong(summary.totalSec))
      : fmtDayTitle(selected)
  );
</script>

<div class="view">
  <header class="head">
    <h1>{m.history.title}</h1>
    <button type="button" class="primary" onclick={() => store.openSession(selected, null)}>
      <Icon name="plus" size={16} />
      {m.history.addManual}
    </button>
  </header>

  <div class="layout">
    <Calendar
      {month}
      {selected}
      {totals}
      dailyTarget={store.goals.dailySec}
      onselect={(date) => (selected = date)}
      onshift={shiftMonth}
    />

    <section class="day" aria-live="polite">
      <h2>{title}</h2>
      {#if !summary || summary.sessions.length === 0}
        <p class="empty">{m.history.empty}</p>
      {:else}
        <div class="rows">
          {#each summary.sessions as session (session.id)}
            <SessionRow {session} onedit={(item) => store.openSession(selected, item)} />
          {/each}
        </div>
      {/if}
    </section>
  </div>
</div>

<style>
  .view {
    display: grid;
    gap: var(--sp-4);
    align-content: start;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--sp-3);
  }

  h1 {
    font-size: 22px;
  }

  .layout {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
    gap: var(--sp-4);
    align-items: start;
  }

  .day {
    container-type: inline-size;
    display: grid;
    gap: var(--sp-3);
    padding: var(--sp-4);
    border-radius: var(--r-lg);
    background: var(--petal);
    align-content: start;
    min-height: 180px;
  }

  h2 {
    font-size: 16px;
  }

  .rows {
    display: grid;
    gap: var(--sp-1);
  }

  .empty {
    font-size: 14px;
    color: var(--mauve);
  }

  .primary {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 40px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-pill);
    background: var(--raspberry);
    color: var(--paper);
    font-size: 14px;
    font-weight: 600;
  }

  .primary:hover {
    background: var(--raspberry-press);
  }
</style>
