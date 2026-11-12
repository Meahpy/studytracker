<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    options: Option[];
    value: string;
    label: string;
    onchange: (value: string) => void;
  }

  let { options, value, label, onchange }: Props = $props();
</script>

<div class="segmented" role="group" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      class="segment"
      class:active={option.value === value}
      aria-pressed={option.value === value}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    gap: var(--sp-1);
    padding: var(--sp-1);
    background: var(--petal-strong);
    border-radius: var(--r-pill);
  }

  .segment {
    height: 36px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-pill);
    font-size: 14px;
    font-weight: 600;
    color: var(--plum);
    transition: background-color var(--dur-color) var(--ease), color var(--dur-color) var(--ease);
  }

  .segment:hover:not(.active) {
    background: var(--petal);
  }

  .segment.active {
    background: var(--raspberry);
    color: var(--paper);
  }
</style>
