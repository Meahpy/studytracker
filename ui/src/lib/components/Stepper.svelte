<script lang="ts">
  import { m } from '../i18n';

  interface Props {
    hours: number;
    minutes: number;
    idPrefix: string;
    maxHours?: number;
    onchange: (hours: number, minutes: number) => void;
  }

  let { hours, minutes, idPrefix, maxHours = 16, onchange }: Props = $props();

  function clampHours(next: number) {
    return Math.min(maxHours, Math.max(0, next));
  }

  function clampMinutes(next: number) {
    if (next >= 60) return 55;
    if (next < 0) return 0;
    return next;
  }

  function bumpHours(delta: number) {
    onchange(clampHours(hours + delta), minutes);
  }

  function bumpMinutes(delta: number) {
    const next = hours * 60 + minutes + delta * 5;
    const total = Math.min(maxHours * 60, Math.max(0, next));
    onchange(Math.floor(total / 60), total % 60);
  }
</script>

<div class="stepper">
  <div class="unit">
    <span class="label" id="{idPrefix}-hours">{m.targets.hours}</span>
    <div class="controls">
      <button
        type="button"
        class="step"
        aria-label="Kurangi {m.targets.hours}"
        onclick={() => bumpHours(-1)}
        disabled={hours <= 0}
      >–</button>
      <span class="value tabular" aria-live="off">{hours}</span>
      <button
        type="button"
        class="step"
        aria-label="Tambah {m.targets.hours}"
        onclick={() => bumpHours(1)}
        disabled={hours >= maxHours}
      >+</button>
    </div>
  </div>

  <div class="unit">
    <span class="label" id="{idPrefix}-minutes">{m.targets.minutes}</span>
    <div class="controls">
      <button
        type="button"
        class="step"
        aria-label="Kurangi {m.targets.minutes}"
        onclick={() => bumpMinutes(-1)}
        disabled={hours === 0 && minutes <= 0}
      >–</button>
      <span class="value tabular" aria-live="off">{String(minutes).padStart(2, '0')}</span>
      <button
        type="button"
        class="step"
        aria-label="Tambah {m.targets.minutes}"
        onclick={() => bumpMinutes(1)}
        disabled={hours >= maxHours}
      >+</button>
    </div>
  </div>
</div>

<style>
  .stepper {
    display: flex;
    gap: var(--sp-6);
    flex-wrap: wrap;
  }

  .unit {
    display: grid;
    gap: var(--sp-2);
  }

  .label {
    font-size: 12px;
    color: var(--mauve);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    background: var(--paper);
    border: 1.5px solid var(--line);
    border-radius: var(--r-md);
    padding: var(--sp-1);
  }

  .step {
    width: 44px;
    height: 40px;
    border-radius: var(--r-sm);
    font-size: 20px;
    font-weight: 600;
    color: var(--plum);
  }

  .step:hover:not(:disabled) {
    background: var(--petal-strong);
  }

  .step:disabled {
    color: var(--line);
  }

  .value {
    min-width: 48px;
    text-align: center;
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
  }
</style>
