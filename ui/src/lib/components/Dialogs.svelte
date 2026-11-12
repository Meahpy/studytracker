<script lang="ts">
  import { store } from '../store.svelte';
  import { api } from '../api';
  import { m } from '../i18n';
  import {
    combineDayAndTime,
    dayKey,
    fmtRange,
    fmtTime,
    timeValue
  } from '../format';
  import { COLORS } from '../types';
  import type { ColorToken } from '../types';
  import Dialog from './Dialog.svelte';
  import Icon from './Icon.svelte';

  // ------------------------------------------------------------------ sesi

  let subjectId = $state(0);
  let startTime = $state('09:00');
  let endTime = $state('10:00');
  let note = $state('');
  let formError = $state('');
  let initialisedFor = $state('');

  $effect(() => {
    const dialog = store.dialog;
    if (dialog?.kind !== 'session') return;
    const key = `${dialog.session?.id ?? 'new'}-${dialog.date}`;
    if (initialisedFor === key) return;
    initialisedFor = key;
    formError = '';
    subjectId = dialog.session?.subjectId ?? store.subjects[0]?.id ?? 0;
    startTime = dialog.session ? timeValue(dialog.session.startedAt) : '09:00';
    endTime = dialog.session?.endedAt ? timeValue(dialog.session.endedAt) : '10:00';
    note = dialog.session?.note ?? '';
  });

  const sessionDialog = $derived(store.dialog?.kind === 'session' ? store.dialog : null);

  async function persist(date: string) {
    const startedAt = combineDayAndTime(date, startTime);
    const endedAt = combineDayAndTime(date, endTime);
    if (endedAt <= startedAt) {
      formError = m.session.invalidRange;
      return;
    }
    await store.guard(async () => {
      const input = { subjectId, startedAt, endedAt, note: note.trim() || null };
      if (sessionDialog?.session) {
        await api.updateSession(sessionDialog.session.id, input);
      } else {
        await api.addManualSession(input);
      }
      store.closeDialog();
      await store.refreshAll();
      store.showToast({ text: m.session.saved, actions: [] });
    });
  }

  async function saveSession() {
    if (!sessionDialog) return;
    if (!subjectId) {
      formError = m.timer.noSubjects;
      return;
    }
    const startedAt = combineDayAndTime(sessionDialog.date, startTime);
    const endedAt = combineDayAndTime(sessionDialog.date, endTime);
    if (endedAt <= startedAt) {
      formError = m.session.invalidRange;
      return;
    }
    const overlap = await api
      .hasOverlap(startedAt, endedAt, sessionDialog.session?.id ?? null)
      .catch(() => false);
    if (overlap) {
      store.confirm({
        title: m.session.editTitle,
        body: m.session.overlapWarning,
        confirmLabel: m.dialog.confirm,
        run: () => persist(sessionDialog.date)
      });
      return;
    }
    await persist(sessionDialog.date);
  }

  function deleteSession() {
    const current = sessionDialog?.session;
    if (!current) return;
    store.confirm({
      title: m.session.delete,
      body: m.session.deleteConfirm(
        current.subjectName,
        fmtTime(current.startedAt),
        current.endedAt ? fmtTime(current.endedAt) : ''
      ),
      confirmLabel: m.session.delete,
      danger: true,
      run: async () => {
        await api.deleteSession(current.id);
        store.closeDialog();
        await store.refreshAll();
        store.showToast({ text: m.session.deleted, actions: [] });
      }
    });
  }

  // -------------------------------------------------------- mata pelajaran

  let subjectName = $state('');
  let subjectColor = $state<ColorToken>('rose');

  $effect(() => {
    const dialog = store.dialog;
    if (dialog?.kind !== 'subject') return;
    subjectName = dialog.subject?.name ?? '';
    subjectColor = dialog.subject?.color ?? 'rose';
    formError = '';
  });

  async function saveSubject() {
    if (store.dialog?.kind !== 'subject') return;
    const editing = store.dialog.subject;
    if (!subjectName.trim()) {
      formError = m.subjects.nameRequired;
      return;
    }
    await store.guard(async () => {
      await api.upsertSubject(editing?.id ?? null, subjectName, subjectColor);
      store.closeDialog();
      await store.refreshAll();
    });
  }

  function archiveSubject() {
    if (store.dialog?.kind !== 'subject') return;
    const subject = store.dialog.subject;
    if (!subject) return;
    void store.guard(async () => {
      await api.archiveSubject(subject.id, !subject.archived);
      store.closeDialog();
      await store.refreshAll();
    });
  }

  function deleteSubject() {
    if (store.dialog?.kind !== 'subject') return;
    const subject = store.dialog.subject;
    if (!subject) return;
    store.confirm({
      title: m.subjects.delete,
      body: m.subjects.onlyNoSessions,
      confirmLabel: m.subjects.delete,
      danger: true,
      run: async () => {
        await api.deleteSubject(subject.id);
        store.closeDialog();
        await store.refreshAll();
      }
    });
  }

  // ------------------------------------------------------------- D-day

  let examTitle = $state('');
  let examDate = $state('');

  $effect(() => {
    const dialog = store.dialog;
    if (dialog?.kind !== 'exam') return;
    examTitle = dialog.exam?.title ?? '';
    examDate = dialog.exam?.examDate ?? dayKey(new Date());
    formError = '';
  });

  async function saveExam() {
    if (store.dialog?.kind !== 'exam') return;
    const editing = store.dialog.exam;
    if (!examTitle.trim() || !examDate) {
      formError = m.targets.invalid;
      return;
    }
    await store.guard(async () => {
      await api.upsertExam(editing?.id ?? null, examTitle, examDate);
      store.closeDialog();
      await store.refreshAll();
    });
  }

  function deleteExam() {
    if (store.dialog?.kind !== 'exam') return;
    const exam = store.dialog.exam;
    if (!exam) return;
    store.confirm({
      title: m.targets.delete,
      body: exam.title,
      confirmLabel: m.targets.delete,
      danger: true,
      run: async () => {
        await api.deleteExam(exam.id);
        store.closeDialog();
        await store.refreshAll();
      }
    });
  }

  // --------------------------------------------------------- sesi tertinggal

  let orphanTime = $state('');
  let orphanError = $state('');

  $effect(() => {
    const dialog = store.dialog;
    if (dialog?.kind !== 'orphan') return;
    // Bulatkan ke atas: input jam hanya sampai menit, jadi sesi yang baru mulai
    // beberapa detik lalu bisa mendapat jam selesai yang lebih awal dari mulai.
    const start = Math.floor(dialog.startedAt);
    const now = Math.floor(Date.now() / 1000);
    orphanTime = timeValue(Math.max(now, start + 60));
    orphanError = '';
  });

  async function saveOrphan() {
    const dialog = store.dialog;
    if (dialog?.kind !== 'orphan') return;
    const endedAt = combineDayAndTime(dayKey(new Date()), orphanTime);
    if (endedAt <= dialog.startedAt) {
      orphanError = m.session.invalidRange;
      return;
    }
    await store.finishOrphan(endedAt);
  }
</script>

{#if store.dialog?.kind === 'session' && sessionDialog}
  <Dialog
    title={sessionDialog.session ? m.session.editTitle : m.session.addTitle}
    onclose={() => store.closeDialog()}
  >
    <label class="field">
      <span>{m.session.subject}</span>
      <select bind:value={subjectId}>
        {#each store.allSubjects as subject (subject.id)}
          <option value={subject.id}>{subject.name}</option>
        {/each}
      </select>
    </label>

    <div class="row">
      <label class="field">
        <span>{m.session.start}</span>
        <input type="time" bind:value={startTime} />
      </label>
      <label class="field">
        <span>{m.session.end}</span>
        <input type="time" bind:value={endTime} />
      </label>
    </div>

    <label class="field">
      <span>{m.session.note}</span>
      <input type="text" bind:value={note} placeholder={m.session.notePlaceholder} />
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="actions">
      {#if sessionDialog.session}
        <button type="button" class="danger" onclick={deleteSession}>{m.session.delete}</button>
      {/if}
      <button type="button" class="secondary" onclick={() => store.closeDialog()}>
        {m.dialog.cancel}
      </button>
      <button type="button" class="primary" onclick={saveSession}>
        {sessionDialog.session ? m.session.save : m.session.saveNew}
      </button>
    </div>
  </Dialog>
{/if}

{#if store.dialog?.kind === 'subject'}
  <Dialog title={store.dialog.subject ? m.subjects.edit : m.subjects.add} onclose={() => store.closeDialog()}>
    <label class="field">
      <span>{m.subjects.name}</span>
      <input type="text" bind:value={subjectName} />
    </label>

    <fieldset class="swatches">
      <legend>{m.subjects.color}</legend>
      {#each COLORS as color (color)}
        <button
          type="button"
          class="swatch"
          class:selected={subjectColor === color}
          style:background="var(--s-{color})"
          aria-label={color}
          aria-pressed={subjectColor === color}
          onclick={() => (subjectColor = color)}
        ></button>
      {/each}
    </fieldset>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="actions">
      {#if store.dialog.subject}
        <button type="button" class="danger" onclick={deleteSubject}>{m.subjects.delete}</button>
        <button type="button" class="secondary" onclick={archiveSubject}>
          {store.dialog.subject.archived ? m.subjects.unarchive : m.subjects.archive}
        </button>
      {/if}
      <button type="button" class="secondary" onclick={() => store.closeDialog()}>
        {m.dialog.cancel}
      </button>
      <button type="button" class="primary" onclick={saveSubject}>{m.subjects.save}</button>
    </div>
  </Dialog>
{/if}

{#if store.dialog?.kind === 'exam'}
  <Dialog title={store.dialog.exam ? m.targets.examTitle : m.targets.addExam} onclose={() => store.closeDialog()}>
    <label class="field">
      <span>{m.targets.examTitle}</span>
      <input type="text" bind:value={examTitle} />
    </label>
    <label class="field">
      <span>{m.targets.examDate}</span>
      <input type="date" bind:value={examDate} />
    </label>

    {#if formError}
      <p class="error">{formError}</p>
    {/if}

    <div class="actions">
      {#if store.dialog.exam}
        <button type="button" class="danger" onclick={deleteExam}>{m.targets.delete}</button>
      {/if}
      <button type="button" class="secondary" onclick={() => store.closeDialog()}>
        {m.dialog.cancel}
      </button>
      <button type="button" class="primary" onclick={saveExam}>{m.targets.save}</button>
    </div>
  </Dialog>
{/if}

{#if store.dialog?.kind === 'confirm'}
  <Dialog title={store.dialog.title} onclose={() => store.closeDialog()}>
    <p>{store.dialog.body}</p>
    <div class="actions">
      <button type="button" class="secondary" onclick={() => store.closeDialog()}>
        {m.dialog.cancel}
      </button>
      <button
        type="button"
        class={store.dialog.danger ? 'danger filled' : 'primary'}
        onclick={async () => {
          const run = store.dialog?.kind === 'confirm' ? store.dialog.run : null;
          store.closeDialog();
          if (run) await run();
        }}
      >
        {store.dialog.confirmLabel}
      </button>
    </div>
  </Dialog>
{/if}

{#if store.dialog?.kind === 'orphan'}
  <Dialog title={m.orphan.title} onclose={() => store.closeDialog()}>
    <p>{m.orphan.body(fmtTime(store.dialog.startedAt))}</p>
    <label class="field">
      <span>{m.session.end}</span>
      <input type="time" bind:value={orphanTime} />
    </label>
    {#if orphanError}
      <p class="error">{orphanError}</p>
    {/if}
    <div class="actions">
      <button
        type="button"
        class="danger"
        onclick={() => store.dialog?.kind === 'orphan' && store.discardOrphan(store.dialog.sessionId)}
      >
        {m.orphan.discard}
      </button>
      <button type="button" class="primary" onclick={saveOrphan}>{m.orphan.save}</button>
    </div>
  </Dialog>
{/if}

{#if store.dialog?.kind === 'idle'}
  <Dialog title={m.idle.title} onclose={() => store.closeDialog()}>
    <p>{m.idle.body(store.dialog.minutes)}</p>
    <div class="actions">
      <button
        type="button"
        class="secondary"
        onclick={async () => {
          if (store.dialog?.kind !== 'idle') return;
          const { minutes, sessionId } = store.dialog;
          store.closeDialog();
          await store.trimIdle(minutes, sessionId);
        }}
      >
        {m.idle.drop(store.dialog.minutes)}
      </button>
      <button type="button" class="primary" onclick={() => store.closeDialog()}>
        {m.idle.count}
      </button>
    </div>
  </Dialog>
{/if}

<style>
  .field {
    display: grid;
    gap: var(--sp-2);
    font-size: 14px;
  }

  .field > span {
    color: var(--mauve);
  }

  .field input,
  .field select {
    height: 44px;
    width: 100%;
    padding: 0 var(--sp-3);
    background: var(--paper);
    border: 1.5px solid var(--line);
    border-radius: var(--r-md);
    color: var(--plum);
  }

  .field input:focus-visible,
  .field select:focus-visible {
    outline: 3px solid var(--plum);
    outline-offset: 2px;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    border: 0;
    padding: 0;
    margin: 0;
  }

  .swatches legend {
    font-size: 14px;
    color: var(--mauve);
    margin-bottom: var(--sp-2);
  }

  .swatch {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 2px solid transparent;
  }

  .swatch.selected {
    border-color: var(--plum);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2);
    justify-content: flex-end;
    margin-top: var(--sp-2);
  }

  .primary {
    height: 48px;
    padding: 0 var(--sp-6);
    border-radius: var(--r-pill);
    background: var(--raspberry);
    color: var(--paper);
    font-weight: 600;
  }

  .primary:hover {
    background: var(--raspberry-press);
  }

  .secondary {
    height: 48px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-md);
    background: var(--petal-strong);
    color: var(--plum);
    font-weight: 600;
  }

  .secondary:hover {
    background: var(--line);
  }

  .danger {
    height: 48px;
    padding: 0 var(--sp-4);
    border-radius: var(--r-md);
    border: 1.5px solid var(--danger);
    color: var(--danger);
    font-weight: 600;
    background: transparent;
  }

  .danger.filled {
    background: var(--danger);
    color: var(--paper);
    border-color: var(--danger);
  }

  .error {
    color: var(--danger);
    font-size: 14px;
  }
</style>
