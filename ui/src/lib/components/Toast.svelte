<script lang="ts">
  import { m } from '../i18n';
  import Icon from './Icon.svelte';
  import { store } from '../store.svelte';
</script>

{#if store.toast}
  <div class="toast" role="status">
    <span>{store.toast.text}</span>
    {#each store.toast.actions as action (action.label)}
      <button
        type="button"
        class="action"
        onclick={async () => {
          store.dismissToast();
          await action.run();
        }}
      >
        {action.label}
      </button>
    {/each}
    <button type="button" class="dismiss" aria-label={m.dialog.close} onclick={() => store.dismissToast()}>
      <Icon name="x" size={14} />
    </button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: var(--sp-6);
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    max-width: min(560px, calc(100vw - 32px));
    padding: var(--sp-2) var(--sp-2) var(--sp-2) var(--sp-6);
    background: var(--plum);
    color: var(--paper);
    border-radius: var(--r-pill);
    z-index: 50;
    font-size: 14px;
    animation: rise var(--dur-dialog) var(--ease);
  }

  .action {
    color: var(--paper);
    font-weight: 600;
    font-size: 14px;
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-pill);
    white-space: nowrap;
  }

  .action:hover {
    background: color-mix(in srgb, var(--paper) 18%, transparent);
    text-decoration: underline;
  }

  .dismiss {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--r-pill);
    color: var(--petal);
    font-size: 12px;
  }

  .dismiss:hover {
    background: color-mix(in srgb, var(--paper) 18%, transparent);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translate(-50%, 6px);
    }
  }
</style>
