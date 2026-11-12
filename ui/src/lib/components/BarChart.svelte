<script lang="ts">
  import { m } from '../i18n';
  import { fmtLong } from '../format';
  import type { Bucket } from '../types';

  interface Props {
    buckets: Bucket[];
    targetSec: number;
  }

  let { buckets, targetSec }: Props = $props();

  const max = $derived(
    Math.max(targetSec, ...buckets.map((bucket) => bucket.totalSec), 1)
  );

  const targetPercent = $derived(targetSec > 0 ? targetSec / max : 0);
  const single = $derived(buckets.length === 1);
</script>

<div class="chart" style:--target-ratio={targetPercent}>
  <div class="plot" class:single>
    <div class="grid-lines" aria-hidden="true">
      <span></span><span></span><span></span>
    </div>

    {#if targetSec > 0}
      <div class="target-line" aria-hidden="true">
        <span class="target-label">{m.stats.targetLine}</span>
      </div>
    {/if}

    {#each buckets as bucket (bucket.date)}
      <button
        type="button"
        class="col"
        aria-label="{bucket.label}: {fmtLong(bucket.totalSec)}"
      >
        <div class="bars">
          {#each bucket.parts as part, index (part.subjectId)}
            <span
              class="seg"
              class:top={index === bucket.parts.length - 1}
              style:height="{(part.secs / max) * 100}%"
              style:background="var(--s-{part.color})"
            ></span>
          {/each}
        </div>
        <span class="tick">{bucket.label}</span>

        <div class="tip" role="tooltip">
          <strong>{fmtLong(bucket.totalSec)}</strong>
          {#each bucket.parts as part (part.subjectId)}
            <span class="tip-row">
              <span class="dot" style:background="var(--s-{part.color})"></span>
              {part.subjectName}
              <span class="tip-value tabular">{fmtLong(part.secs)}</span>
            </span>
          {/each}
          {#if bucket.parts.length === 0}
            <span class="tip-row">{m.stats.noData}</span>
          {/if}
        </div>
      </button>
    {/each}
  </div>
</div>

<style>
  .chart {
    padding: var(--sp-4) var(--sp-4) 0;
    background: var(--petal);
    border-radius: var(--r-lg);
  }

  .plot {
    position: relative;
    display: flex;
    align-items: end;
    gap: var(--sp-2);
    height: 200px;
    padding-bottom: 28px;
  }

  .plot.single {
    justify-content: center;
  }

  .plot.single .col {
    max-width: 160px;
  }

  .grid-lines {
    position: absolute;
    inset: 0 0 28px 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    pointer-events: none;
  }

  .grid-lines span {
    display: block;
    height: 1px;
    background: var(--line);
  }

  .target-line {
    position: absolute;
    left: 0;
    right: 0;
    bottom: calc(28px + (100% - 28px) * var(--target-ratio));
    border-top: 1.5px dashed var(--raspberry);
    pointer-events: none;
  }

  .target-label {
    position: absolute;
    right: 0;
    top: -18px;
    font-size: 12px;
    color: var(--raspberry);
  }

  .col {
    position: relative;
    flex: 1;
    min-width: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: end;
    align-items: center;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    cursor: default;
  }

  .col:focus-visible {
    outline: 2px solid var(--plum);
    outline-offset: 2px;
    border-radius: var(--r-sm);
  }

  .bars {
    display: flex;
    flex-direction: column-reverse;
    justify-content: flex-start;
    width: 100%;
    height: calc(100% - 28px);
  }

  .seg {
    display: block;
    flex: none;
    width: 100%;
  }

  .seg.top {
    border-radius: 6px 6px 0 0;
  }

  .tick {
    height: 28px;
    display: grid;
    place-items: center;
    font-size: 12px;
    color: var(--mauve);
  }

  .tip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    display: grid;
    gap: var(--sp-1);
    min-width: 160px;
    padding: var(--sp-3);
    background: var(--paper);
    border: 1.5px solid var(--line);
    border-radius: var(--r-md);
    font-size: 12px;
    opacity: 0;
    visibility: hidden;
    z-index: 10;
    color: var(--plum);
    transition: opacity var(--dur-dialog) var(--ease);
  }

  .col:hover .tip,
  .col:focus .tip,
  .col:focus-within .tip {
    opacity: 1;
    visibility: visible;
  }

  .tip strong {
    font-size: 14px;
  }

  .tip-row {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    white-space: nowrap;
  }

  .tip-value {
    margin-left: auto;
    color: var(--mauve);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
</style>
