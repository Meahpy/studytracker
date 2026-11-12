<script lang="ts">
  import { m } from '../lib/i18n';
  import Icon from '../lib/components/Icon.svelte';
  import SubjectPicker from '../lib/components/SubjectPicker.svelte';
  import TimerPanel from '../lib/components/TimerPanel.svelte';
  import { store } from '../lib/store.svelte';
</script>

<div class="view">
  <header class="head">
    <h1>{m.timer.title}</h1>
    <button type="button" class="mini" onclick={() => store.openMini()}>
      <Icon name="shrink" size={16} />
      {m.timer.mini}
    </button>
  </header>

  <SubjectPicker />
  <TimerPanel />
</div>

<style>
  .view {
    display: grid;
    gap: var(--sp-4);
    align-content: start;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-3);
  }

  h1 {
    font-size: 22px;
  }

  .mini {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 36px;
    padding: 0 var(--sp-3);
    border-radius: var(--r-pill);
    font-size: 13px;
    font-weight: 600;
    color: var(--mauve);
  }

  .mini:hover {
    background: var(--petal-strong);
    color: var(--plum);
  }
</style>
