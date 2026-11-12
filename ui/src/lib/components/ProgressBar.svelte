<script lang="ts">
  import { m } from '../i18n';
  import { fmtPercent } from '../format';

  interface Props {
    value: number;
    label?: string;
    tone?: 'raspberry' | 'ok';
    height?: number;
  }

  let { value, label = '', tone = 'raspberry', height = 12 }: Props = $props();
  const percent = $derived(Math.round(Math.min(1, Math.max(0, value)) * 100));
</script>

<div
  class="bar"
  style:height="{height}px"
  role="progressbar"
  aria-valuenow={percent}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-label={label || m.a11y.progress(percent)}
>
  <div class="fill" class:ok={tone === 'ok'} style:width="{percent}%"></div>
</div>

<style>
  .bar {
    width: 100%;
    background: var(--petal-strong);
    border-radius: var(--r-pill);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: var(--raspberry);
    border-radius: var(--r-pill);
    transition: width var(--dur-color) var(--ease), background-color var(--dur-color) var(--ease);
  }

  .fill.ok {
    background: var(--ok);
  }
</style>
