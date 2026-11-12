<script lang="ts">
  import { store } from '../store.svelte';
  import { m } from '../i18n';
  import Icon from './Icon.svelte';

  const locked = $derived(store.timer.phase !== 'idle');
  const current = $derived(store.timer.subjectId);

  function select(id: number) {
    if (locked) return;
    if (current === id) {
      void store.toggle();
      return;
    }
    void store.start(id);
  }
</script>

<div class="picker" role="group" aria-label={m.timer.title}>
  {#each store.subjects as subject (subject.id)}
    <button
      type="button"
      class="chip"
      class:selected={current === subject.id}
      style:background="var(--s-{subject.color})"
      aria-pressed={current === subject.id}
      disabled={locked && current !== subject.id}
      onclick={() => select(subject.id)}
    >
      {subject.name}
    </button>
  {/each}

  <button type="button" class="chip new" onclick={() => store.openSubject(null)}>
    <Icon name="plus" size={16} />
    {m.timer.newSubject}
  </button>
</div>

{#if locked}
  <p class="hint">{m.timer.lockedHint}</p>
{/if}

<style>
  .picker {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 36px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-pill);
    font-size: 14px;
    font-weight: 600;
    color: var(--plum);
    border: 2px solid transparent;
    transition: border-color var(--dur-color) var(--ease);
  }

  .chip:hover:not(:disabled) {
    border-color: var(--line);
  }

  .chip.selected {
    border-color: var(--plum);
  }

  .chip:disabled {
    opacity: 0.45;
  }

  .chip.new {
    background: transparent;
    border: 1.5px dashed var(--line);
    color: var(--plum);
  }

  .chip.new:hover {
    background: var(--petal);
    border-color: var(--plum);
  }

  .hint {
    margin-top: var(--sp-2);
    font-size: 12px;
    color: var(--mauve);
  }
</style>
