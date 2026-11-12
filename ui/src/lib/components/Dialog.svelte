<script lang="ts">
  import { m } from '../i18n';
  import Icon from './Icon.svelte';

  interface Props {
    title: string;
    onclose: () => void;
    children: import('svelte').Snippet;
  }

  let { title, onclose, children }: Props = $props();

  let panel: HTMLDivElement | undefined = $state();

  $effect(() => {
    panel?.focus();
  });
</script>

<div
  class="scrim"
  role="presentation"
  onclick={(event) => event.target === event.currentTarget && onclose()}
>
  <div
    class="panel"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    bind:this={panel}
  >
    <header>
      <h2>{title}</h2>
      <button type="button" class="close" aria-label={m.dialog.close} onclick={onclose}>
        <Icon name="x" size={18} />
      </button>
    </header>

    {@render children()}
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, var(--plum) 40%, transparent);
    display: grid;
    place-items: center;
    padding: var(--sp-4);
    z-index: 40;
  }

  .panel {
    width: min(480px, 100%);
    max-height: calc(100vh - 32px);
    overflow: auto;
    background: var(--blush);
    border: 1.5px solid var(--line);
    border-radius: var(--r-xl);
    padding: var(--sp-6);
    display: grid;
    gap: var(--sp-4);
    animation: open var(--dur-dialog) var(--ease);
  }

  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }

  header {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: var(--sp-4);
  }

  h2 {
    font-size: 20px;
  }

  .close {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border-radius: var(--r-sm);
    color: var(--mauve);
    margin: -10px -10px 0 0;
  }

  .close:hover {
    background: var(--petal-strong);
    color: var(--plum);
  }
</style>
