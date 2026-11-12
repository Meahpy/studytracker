<script lang="ts">
  import { m } from '../i18n';
  import { fmtLong } from '../format';
  import type { HeatCell } from '../types';

  interface Props {
    weeks: HeatCell[][];
  }

  let { weeks }: Props = $props();
</script>

<div class="heatmap" role="img" aria-label={m.stats.heatmap}>
  <div class="columns">
    {#each weeks as column, index (index)}
      <div class="column">
        {#each column as cell (cell.date)}
          <span
            class="cell"
            class:zero={cell.level === 0}
            style:background={cell.level > 0 ? `var(--heat-${cell.level})` : undefined}
            title="{cell.date}: {fmtLong(cell.totalSec)}"
          ></span>
        {/each}
      </div>
    {/each}
  </div>

  <div class="legend">
    <span>{m.stats.legend.less}</span>
    {#each [0, 1, 2, 3, 4] as level (level)}
      <span class="cell" class:zero={level === 0} style:background={level > 0 ? `var(--heat-${level})` : undefined}></span>
    {/each}
    <span>{m.stats.legend.more}</span>
  </div>
</div>

<style>
  .heatmap {
    display: grid;
    gap: var(--sp-3);
    padding: var(--sp-4);
    border-radius: var(--r-lg);
    background: var(--petal);
  }

  .columns {
    display: flex;
    gap: 3px;
  }

  .column {
    display: grid;
    gap: 3px;
  }

  .cell {
    display: block;
    width: 12px;
    height: 12px;
    border-radius: 4px;
  }

  /* Hari tanpa sesi: pakai heat-0 di tema terang, warna panel di tema gelap
     supaya grid kosong tidak jadi bidang paling terang di layar. */
  .cell.zero {
    background: var(--heat-0);
  }

  :global([data-theme='dark']) .cell.zero {
    background: var(--petal-strong);
  }

  .legend {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    font-size: 12px;
    color: var(--mauve);
  }

  .legend span:first-child {
    margin-right: var(--sp-1);
  }

  .legend span:last-child {
    margin-left: var(--sp-1);
  }
</style>
