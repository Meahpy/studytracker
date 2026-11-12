<script lang="ts">
  import { api } from '../lib/api';
  import { m } from '../lib/i18n';
  import { dayKey, fmtDateShort, fmtLong, parseDayKey } from '../lib/format';
  import { store } from '../lib/store.svelte';
  import type { Heatmap, RangeStats, Streaks, SubjectTotal } from '../lib/types';
  import BarChart from '../lib/components/BarChart.svelte';
  import HeatmapChart from '../lib/components/Heatmap.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Segmented from '../lib/components/Segmented.svelte';

  type Gran = 'day' | 'week' | 'month';

  const pad = (n: number) => String(n).padStart(2, '0');

  let gran = $state<Gran>('day');
  let offset = $state(0);
  let stats = $state<RangeStats | null>(null);
  let breakdown = $state<SubjectTotal[]>([]);
  let heat = $state<Heatmap | null>(null);
  let streaks = $state<Streaks | null>(null);

  const options = $derived([
    { value: 'day', label: m.stats.day },
    { value: 'week', label: m.stats.week },
    { value: 'month', label: m.stats.month }
  ]);

  const range = $derived.by(() => {
    const base = parseDayKey(dayKey(new Date()));
    if (gran === 'week') {
      const end = new Date(base);
      end.setDate(end.getDate() - ((end.getDay() + 6) % 7) - offset * 7 + 6);
      const start = new Date(end);
      start.setDate(start.getDate() - 6);
      return { from: dayKey(start), to: dayKey(end) };
    }
    if (gran === 'month') {
      const anchor = new Date(base.getFullYear(), base.getMonth() - offset, 1);
      const end = new Date(anchor.getFullYear(), anchor.getMonth() + 1, 0);
      return { from: dayKey(anchor), to: dayKey(end) };
    }
    const end = new Date(base);
    end.setDate(end.getDate() - offset * 7);
    const start = new Date(end);
    start.setDate(start.getDate() - 6);
    return { from: dayKey(start), to: dayKey(end) };
  });

  $effect(() => {
    const { from, to } = range;
    void store.revision;
    let cancelled = false;
    Promise.all([api.statsRange(from, to, 'day'), api.subjectBreakdown(from, to)])
      .then(([next, parts]) => {
        if (cancelled) return;
        stats = next;
        breakdown = parts;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    void store.revision;
    let cancelled = false;
    Promise.all([api.heatmap(26), api.streaks()])
      .then(([nextHeat, nextStreaks]) => {
        if (cancelled) return;
        heat = nextHeat;
        streaks = nextStreaks;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  });

  function changeGran(value: string) {
    gran = value as Gran;
    offset = 0;
  }

  const summaryText = $derived.by(() => {
    if (!stats || stats.totalSec === 0) return '';
    const delta = stats.totalSec - stats.previousTotalSec;
    if (delta === 0) return m.stats.summarySame(fmtLong(stats.totalSec));
    return m.stats.summary(fmtLong(stats.totalSec), fmtLong(Math.abs(delta)), delta > 0);
  });

  const breakdownMax = $derived(
    Math.max(1, ...breakdown.map((row) => row.secs))
  );

  const periodLabel = $derived(`${fmtDateShort(range.from)} – ${fmtDateShort(range.to)}`);
  const isEmpty = $derived(stats !== null && stats.totalSec === 0);
</script>

<div class="view">
  <header class="head">
    <h1>{m.stats.title}</h1>
    <Segmented {options} value={gran} label={m.stats.title} onchange={changeGran} />
  </header>

  <div class="period">
    <button
      type="button"
      class="nav"
      aria-label={m.stats.previous}
      onclick={() => (offset += 1)}
    >
      <Icon name="chevron-left" size={18} />
    </button>
    <span class="label tabular" aria-live="polite">{periodLabel}</span>
    <button
      type="button"
      class="nav"
      aria-label={m.stats.next}
      disabled={offset === 0}
      onclick={() => (offset = Math.max(0, offset - 1))}
    >
      <Icon name="chevron-right" size={18} />
    </button>
  </div>

  {#if isEmpty}
    <p class="empty">{m.stats.empty}</p>
  {:else if stats}
    <p class="summary">{summaryText}</p>

    <BarChart buckets={stats.buckets} targetSec={store.goals.dailySec} />

    <section class="block">
      <h2>{m.stats.breakdown}</h2>
      {#if breakdown.length === 0}
        <p class="empty">{m.stats.noData}</p>
      {:else}
        <ul class="parts">
          {#each breakdown as part (part.subjectId)}
            <li>
              <span class="dot" style:background="var(--s-{part.color})"></span>
              <span class="name">{part.subjectName}</span>
              <span class="duration tabular">{fmtLong(part.secs)}</span>
              <span class="track">
                <span
                  class="fill"
                  style:width="{(part.secs / breakdownMax) * 100}%"
                  style:background="var(--s-{part.color})"
                ></span>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/if}

  <div class="lower">
    {#if heat}
      <section class="block">
        <h2>{m.stats.heatmap}</h2>
        <HeatmapChart weeks={heat.weeks} />
      </section>
    {/if}

    {#if streaks}
      <section class="block">
        <h2>{m.stats.streaks}</h2>
        <dl class="streaks">
          <div>
            <dt>{m.stats.currentStreak}</dt>
            <dd>{m.stats.days(streaks.currentDays)}</dd>
          </div>
          <div>
            <dt>{m.stats.longestStreak}</dt>
            <dd>{m.stats.days(streaks.longestDays)}</dd>
          </div>
          <div>
            <dt>{m.stats.bestDay}</dt>
            <dd>{fmtLong(streaks.bestDaySec)}</dd>
          </div>
          <div>
            <dt>{m.stats.bestWeek}</dt>
            <dd>{fmtLong(streaks.bestWeekSec)}</dd>
          </div>
        </dl>
      </section>
    {/if}
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

  .period {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .period .label {
    font-size: 14px;
    color: var(--mauve);
  }

  .nav {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: var(--r-sm);
    color: var(--plum);
  }

  .nav:hover:not(:disabled) {
    background: var(--petal-strong);
  }

  .nav:disabled {
    color: var(--line);
  }

  .summary {
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    line-height: 1.4;
    max-width: 44ch;
  }

  .empty {
    font-size: 14px;
    color: var(--mauve);
  }

  .block {
    display: grid;
    gap: var(--sp-3);
  }

  .lower {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
    gap: var(--sp-6);
    align-items: start;
  }

  h2 {
    font-size: 16px;
  }

  .parts {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--sp-2);
  }

  .parts li {
    display: grid;
    grid-template-columns: 12px minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--sp-2) var(--sp-3);
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .name {
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .duration {
    font-size: 14px;
    font-weight: 600;
  }

  .track {
    grid-column: 2 / -1;
    height: 8px;
    border-radius: var(--r-pill);
    background: var(--petal-strong);
    overflow: hidden;
  }

  .fill {
    display: block;
    height: 100%;
    border-radius: var(--r-pill);
  }

  .streaks {
    margin: 0;
    display: grid;
  }

  .streaks div {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--sp-3);
    padding: var(--sp-3) 0;
    border-top: 1.5px solid var(--line);
  }

  .streaks div:last-child {
    border-bottom: 1.5px solid var(--line);
  }

  .streaks dt {
    font-size: 14px;
    color: var(--mauve);
  }

  .streaks dd {
    margin: 0;
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
  }
</style>
