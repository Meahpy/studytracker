<script lang="ts">
  import { m } from '../i18n';
  import { dayKey, fmtPercent } from '../format';
  import Icon from './Icon.svelte';

  interface Props {
    month: string;
    selected: string;
    totals: Record<string, number>;
    dailyTarget: number;
    onselect: (date: string) => void;
    onshift: (months: number) => void;
  }

  let { month, selected, totals, dailyTarget, onselect, onshift }: Props = $props();

  const year = $derived(Number(month.slice(0, 4)));
  const monthIndex = $derived(Number(month.slice(5, 7)) - 1);

  const cells = $derived.by(() => {
    const first = new Date(year, monthIndex, 1);
    const offset = (first.getDay() + 6) % 7;
    const daysInMonth = new Date(year, monthIndex + 1, 0).getDate();
    const list: Array<{ key: string; day: number; inMonth: boolean }> = [];
    for (let i = 0; i < offset; i += 1) {
      const date = new Date(year, monthIndex, 1 - (offset - i));
      list.push({ key: dayKey(date), day: date.getDate(), inMonth: false });
    }
    for (let day = 1; day <= daysInMonth; day += 1) {
      list.push({ key: dayKey(new Date(year, monthIndex, day)), day, inMonth: true });
    }
    while (list.length % 7 !== 0) {
      const last = new Date(year, monthIndex, daysInMonth + (list.length % 7));
      list.push({ key: dayKey(last), day: last.getDate(), inMonth: false });
    }
    return list;
  });

  function level(key: string): number {
    const secs = totals[key] ?? 0;
    if (secs <= 0) return 0;
    if (dailyTarget <= 0) return 4;
    const ratio = secs / dailyTarget;
    if (ratio >= 1) return 4;
    if (ratio >= 0.5) return 3;
    if (ratio >= 0.25) return 2;
    return 1;
  }
</script>

<div class="calendar">
  <header>
    <button type="button" class="nav" aria-label={m.history.previousMonth} onclick={() => onshift(-1)}>
      <Icon name="chevron-left" size={18} />
    </button>
    <span class="month" aria-live="polite">{m.monthNames[monthIndex]} {year}</span>
    <button type="button" class="nav" aria-label={m.history.nextMonth} onclick={() => onshift(1)}>
      <Icon name="chevron-right" size={18} />
    </button>
  </header>

  <div class="weekdays" aria-hidden="true">
    {#each m.dayShort as label (label)}
      <span>{label}</span>
    {/each}
  </div>

  <div class="grid" role="grid">
    {#each cells as cell (cell.key)}
      {@const lv = cell.inMonth ? level(cell.key) : 0}
      <button
        type="button"
        class="cell"
        class:outside={!cell.inMonth}
        class:selected={cell.key === selected}
        class:filled={lv > 0}
        class:strongest={lv >= 4}
        style:background={lv > 0 ? `var(--heat-${lv})` : 'transparent'}
        aria-pressed={cell.key === selected}
        aria-label="{cell.key}, {fmtPercent(totals[cell.key] ?? 0, dailyTarget)}%"
        onclick={() => onselect(cell.key)}
      >
        {cell.day}
      </button>
    {/each}
  </div>
</div>

<style>
  .calendar {
    display: grid;
    gap: var(--sp-3);
    padding: var(--sp-4);
    border-radius: var(--r-lg);
    background: var(--petal);
    align-content: start;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
  }

  .month {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 16px;
  }

  .nav {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border-radius: var(--r-sm);
    color: var(--plum);
  }

  .nav:hover {
    background: var(--petal-strong);
  }

  .weekdays,
  .grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: var(--sp-1);
  }

  .weekdays span {
    text-align: center;
    font-size: 12px;
    color: var(--mauve);
  }

  .cell {
    aspect-ratio: 1;
    min-height: 34px;
    border-radius: var(--r-sm);
    font-size: 13px;
    font-weight: 500;
    color: var(--plum);
    border: 2px solid transparent;
  }

  .cell:hover {
    border-color: var(--line);
  }

  .cell.outside {
    color: var(--mauve);
    opacity: 0.45;
  }

  .cell.filled {
    color: var(--on-heat);
  }

  .cell.strongest {
    color: var(--on-heat-strong);
  }

  .cell.selected {
    border-color: var(--plum);
  }
</style>
