<script lang="ts">
  import { m } from '../i18n';
  import { fmtClock, fmtLong, fmtPercent, fmtRange } from '../format';
  import type { SessionView } from '../types';
  import Icon from './Icon.svelte';

  interface Props {
    session: SessionView;
    onedit?: (session: SessionView) => void;
  }

  let { session, onedit }: Props = $props();

  const duration = $derived(fmtClock(session.effectiveDurationSec));
  const live = $derived(!session.endedAt);
  const paused = $derived(!session.endedAt && session.pausedAt !== null);
</script>

<button type="button" class="row" onclick={() => onedit?.(session)} disabled={!onedit}>
  <span class="dot" style:background="var(--s-{session.subjectColor})" aria-hidden="true"></span>
  <span class="text">
    <span class="name">{session.subjectName}</span>
    {#if session.note}
      <span class="note">{session.note}</span>
    {/if}
  </span>
  <span class="times tabular">
    {#if session.endedAt}
      {fmtRange(session.startedAt, session.endedAt)}
    {:else}
      {fmtRange(session.startedAt, session.startedAt + session.effectiveDurationSec)}
    {/if}
  </span>
  <span class="duration tabular">
    {duration}
    {#if live}
      <span class="live">{paused ? m.timer.paused : m.timer.running}</span>
    {/if}
  </span>
</button>

<style>
  .row {
    display: grid;
    grid-template-columns: 12px minmax(0, 1fr) auto auto;
    align-items: center;
    gap: var(--sp-3);
    width: 100%;
    min-height: 56px;
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-md);
    text-align: left;
    color: var(--plum);
    transition: background-color var(--dur-color) var(--ease);
  }

  .row:hover:not(:disabled) {
    background: var(--petal-strong);
  }

  .row:disabled {
    cursor: default;
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .text {
    display: grid;
    min-width: 0;
  }

  .name {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .note {
    font-size: 12px;
    color: var(--mauve);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .times {
    font-size: 14px;
    color: var(--mauve);
    white-space: nowrap;
  }

  .duration {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
  }

  .live {
    margin-left: var(--sp-2);
    font-size: 12px;
    font-weight: 500;
    color: var(--mauve);
  }

  @container (max-width: 380px) {
    .row {
      grid-template-columns: 12px minmax(0, 1fr) auto;
    }

    .times {
      display: none;
    }
  }
</style>
