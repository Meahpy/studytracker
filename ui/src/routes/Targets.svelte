<script lang="ts">
  import { m } from '../lib/i18n';
  import { fmtDaysLeft, fmtDayTitle } from '../lib/format';
  import { store } from '../lib/store.svelte';
  import Icon from '../lib/components/Icon.svelte';
  import Stepper from '../lib/components/Stepper.svelte';

  function split(secs: number) {
    const total = Math.round(secs / 60);
    return { hours: Math.floor(total / 60), minutes: total % 60 };
  }

  const daily = $derived(split(store.goals.dailySec));
  const weekly = $derived(split(store.goals.weeklySec));

  function save(scope: 'daily' | 'weekly', hours: number, minutes: number) {
    void store.setGoal(scope, hours * 3600 + minutes * 60);
  }
</script>

<div class="view">
  <header class="head">
    <h1>{m.targets.title}</h1>
    <button type="button" class="primary" onclick={() => store.openExam(null)}>
      <Icon name="plus" size={16} />
      {m.targets.addExam}
    </button>
  </header>

  <section class="block">
    <div class="goal">
      <div class="goal-text">
        <h2>{m.targets.daily}</h2>
        <p>{m.targets.dailyHint}</p>
      </div>
      <Stepper
        hours={daily.hours}
        minutes={daily.minutes}
        idPrefix="daily"
        onchange={(h, min) => save('daily', h, min)}
      />
    </div>

    <div class="goal">
      <div class="goal-text">
        <h2>{m.targets.weekly}</h2>
        <p>{m.targets.weeklyHint}</p>
      </div>
      <Stepper
        hours={weekly.hours}
        minutes={weekly.minutes}
        idPrefix="weekly"
        maxHours={80}
        onchange={(h, min) => save('weekly', h, min)}
      />
    </div>
  </section>

  <section class="block">
    <h2>{m.targets.dday}</h2>
    {#if store.exams.length === 0}
      <p class="empty">{m.targets.empty}</p>
    {:else}
      <ul class="exams">
        {#each store.exams as exam (exam.id)}
          <li>
            <button type="button" class="exam" onclick={() => store.openExam(exam)}>
              <span class="exam-text">
                <span class="exam-title">{exam.title}</span>
                <span class="exam-date">{fmtDayTitle(exam.examDate)}</span>
              </span>
              <span
                class="countdown"
                class:soon={exam.daysLeft >= 0 && exam.daysLeft <= 7}
                class:overdue={exam.daysLeft < 0}
              >
                {fmtDaysLeft(exam.daysLeft)}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .view {
    display: grid;
    gap: var(--sp-6);
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

  .block {
    display: grid;
    gap: var(--sp-3);
  }

  h2 {
    font-size: 16px;
  }

  .goal {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    padding: var(--sp-4);
    border-radius: var(--r-lg);
    background: var(--petal);
  }

  .goal-text {
    display: grid;
    gap: var(--sp-1);
  }

  .goal-text p {
    font-size: 13px;
    color: var(--mauve);
    max-width: 42ch;
  }

  .exams {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: var(--sp-2);
  }

  .exam {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    width: 100%;
    min-height: 64px;
    padding: var(--sp-3) var(--sp-4);
    border-radius: var(--r-md);
    background: var(--petal);
    text-align: left;
    color: var(--plum);
    transition: background-color var(--dur-color) var(--ease);
  }

  .exam:hover {
    background: var(--petal-strong);
  }

  .exam-text {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .exam-title {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .exam-date {
    font-size: 12px;
    color: var(--mauve);
  }

  .countdown {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
  }

  .countdown.soon {
    color: var(--warn);
  }

  .countdown.overdue {
    color: var(--danger);
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
