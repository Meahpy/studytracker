<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (value: boolean) => void;
  }

  let { checked, label, disabled = false, onchange }: Props = $props();
</script>

<button
  type="button"
  class="toggle"
  class:on={checked}
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 44px;
    height: 26px;
    min-width: 44px;
    border-radius: var(--r-pill);
    background: var(--line);
    transition: background-color var(--dur-color) var(--ease);
  }

  .toggle.on {
    background: var(--raspberry);
  }

  .toggle:disabled {
    opacity: 0.5;
  }

  .knob {
    position: absolute;
    top: 4px;
    left: 4px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--paper);
    transition: transform var(--dur-color) var(--ease);
  }

  .toggle.on .knob {
    transform: translateX(18px);
  }
</style>
