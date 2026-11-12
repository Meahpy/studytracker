<script lang="ts">
  import { api } from '../lib/api';
  import { m } from '../lib/i18n';
  import { fmtLong } from '../lib/format';
  import { store } from '../lib/store.svelte';
  import type { SubjectTotal } from '../lib/types';
  import Icon from '../lib/components/Icon.svelte';

  let totals = $state<SubjectTotal[]>([]);

  $effect(() => {
    void store.revision;
    let cancelled = false;
    api
      .subjectTotals()
      .then((rows) => {
        if (!cancelled) totals = rows;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  });

  const rows = $derived.by(() => {
    return totals
      .map((row) => ({
        ...row,
        archived: store.allSubjects.find((subject) => subject.id === row.subjectId)?.archived ?? false
      }))
      .sort((a, b) => {
        if (a.archived !== b.archived) return a.archived ? 1 : -1;
        return a.subjectName.localeCompare(b.subjectName);
      });
  });
</script>

<div class="view">
  <header class="head">
    <h1>{m.subjects.title}</h1>
    <button type="button" class="primary" onclick={() => store.openSubject(null)}>
      <Icon name="plus" size={16} />
      {m.subjects.add}
    </button>
  </header>

  {#if rows.length === 0}
    <p class="empty">{m.subjects.empty}</p>
  {:else}
    <ul class="rows">
      {#each rows as row (row.subjectId)}
        <li>
          <button
            type="button"
            class="row"
            class:archived={row.archived}
            onclick={() => {
              const subject = store.allSubjects.find((item) => item.id === row.subjectId);
              if (subject) store.openSubject(subject);
            }}
          >
            <span class="dot" style:background="var(--s-{row.color})"></span>
            <span class="name">
              {row.subjectName}
              {#if row.archived}
                <span class="tag">{m.subjects.archivedTag}</span>
              {/if}
            </span>
            <span class="total tabular">{row.secs > 0 ? fmtLong(row.secs) : ''}</span>
            <Icon name="pencil" size={16} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
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
    flex-wrap: wrap;
    gap: var(--sp-3);
  }

  h1 {
    font-size: 22px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0 var(--sp-2);
    display: grid;
    border-radius: var(--r-lg);
    background: var(--petal);
    max-width: 820px;
  }

  .rows li + li {
    border-top: 1.5px solid var(--line);
  }

  .row {
    display: grid;
    grid-template-columns: 12px minmax(0, 1fr) auto 20px;
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

  .row:hover {
    background: var(--petal-strong);
  }

  .row.archived {
    color: var(--mauve);
  }

  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .name {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-weight: 500;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag {
    font-size: 11px;
    font-weight: 600;
    padding: 2px var(--sp-2);
    border-radius: var(--r-pill);
    background: var(--petal-strong);
    color: var(--mauve);
    white-space: nowrap;
  }

  .total {
    font-size: 14px;
    color: var(--mauve);
    white-space: nowrap;
  }

  .empty {
    font-size: 14px;
    color: var(--mauve);
  }

  .primary {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    height: 40px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-pill);
    background: var(--raspberry);
    color: var(--on-raspberry);
    font-size: 14px;
    font-weight: 600;
  }

  .primary:hover {
    background: var(--raspberry-press);
  }
</style>
