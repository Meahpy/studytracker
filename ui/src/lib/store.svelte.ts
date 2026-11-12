import { api, isPreview, listenSessionChanged, windowLabel } from './api';
import { m } from './i18n';
import { addDays, dayKey } from './format';
import type {
  AppInfo,
  DaySummary,
  ExamView,
  Goals,
  SessionView,
  Settings,
  Subject,
  TimerSnapshot
} from './types';

export type View = 'timer' | 'history' | 'stats' | 'targets' | 'subjects' | 'settings';

export type DialogState =
  | { kind: 'session'; session: SessionView | null; date: string }
  | { kind: 'subject'; subject: Subject | null }
  | { kind: 'exam'; exam: ExamView | null }
  | {
      kind: 'confirm';
      title: string;
      body: string;
      confirmLabel: string;
      danger: boolean;
      run: () => void | Promise<void>;
    }
  | { kind: 'orphan'; sessionId: number; subjectName: string; startedAt: number }
  | { kind: 'idle'; minutes: number; sessionId: number };

export interface ToastState {
  text: string;
  actions: Array<{ label: string; run: () => void | Promise<void> }>;
}

const emptyTimer: TimerSnapshot = {
  phase: 'idle',
  sessionId: null,
  subjectId: null,
  subjectName: null,
  subjectColor: null,
  startedAt: null,
  pausedAt: null,
  pausedTotalSec: 0,
  elapsedSec: 0
};

const defaultSettings: Settings = {
  dayStartHour: 4,
  idleThresholdMin: 0,
  countSleepTime: false,
  closeToTray: true,
  autostart: false,
  theme: 'system',
  uiScale: 100,
  globalShortcut: 'Ctrl+Alt+S',
  rememberWindow: true,
  language: 'id',
  miniAlwaysOnTop: false
};

const emptyToday: DaySummary = { date: '', totalSec: 0, sessions: [] };

function mondayOf(key: string): string {
  const date = new Date(Number(key.slice(0, 4)), Number(key.slice(5, 7)) - 1, Number(key.slice(8, 10)));
  date.setDate(date.getDate() - ((date.getDay() + 6) % 7));
  return dayKey(date);
}

class AppStore {
  view = $state<View>('timer');
  isMini = $state(windowLabel() === 'mini');
  preview = isPreview;

  timer = $state<TimerSnapshot>({ ...emptyTimer });
  subjects = $state<Subject[]>([]);
  allSubjects = $state<Subject[]>([]);
  today = $state<DaySummary>({ ...emptyToday });
  weekSec = $state(0);
  goals = $state<Goals>({ dailySec: 4 * 3600, weeklySec: 20 * 3600 });
  exams = $state<ExamView[]>([]);
  settings = $state<Settings>({ ...defaultSettings });
  info = $state<AppInfo | null>(null);

  ready = $state(false);
  busy = $state(false);
  error = $state<string | null>(null);
  dialog = $state<DialogState | null>(null);
  toast = $state<ToastState | null>(null);

  /** Naik setiap data dimuat ulang, supaya layar bisa memuat ulang turunannya. */
  revision = $state(0);

  nowMs = $state(Date.now());
  syncedAtMs = Date.now();

  private toastTimer: number | null = null;
  private initialised = false;
  private targetNotified = '';
  private idleDismissedAt = 0;

  /** Angka yang berdetak hanya tampilan; sumbernya tetap timestamp dari backend. */
  get elapsedSec(): number {
    if (this.timer.phase !== 'running') return this.timer.elapsedSec;
    const drift = (this.nowMs - this.syncedAtMs) / 1000;
    return this.timer.elapsedSec + Math.max(0, drift);
  }

  get progress(): number {
    if (this.goals.dailySec <= 0) return 0;
    return Math.min(1, this.today.totalSec / this.goals.dailySec);
  }

  get remainingToTarget(): number {
    return Math.max(0, this.goals.dailySec - this.today.totalSec);
  }

  get targetReached(): boolean {
    return this.goals.dailySec > 0 && this.today.totalSec >= this.goals.dailySec;
  }

  get activeSubject(): Subject | undefined {
    return this.subjects.find((subject) => subject.id === this.timer.subjectId);
  }

  async init() {
    await this.refreshAll();
    const [info] = await Promise.all([api.appInfo().catch(() => null)]);
    this.info = info;
    this.ready = true;

    if (this.timer.phase !== 'idle' && this.timer.sessionId) {
      this.dialog = {
        kind: 'orphan',
        sessionId: this.timer.sessionId,
        subjectName: this.timer.subjectName ?? '',
        startedAt: this.timer.startedAt ?? Date.now() / 1000
      };
    }

    await listenSessionChanged(() => {
      void this.refreshAll();
    });

    this.initialised = true;
    this.startIdleWatch();
  }

  async refreshAll() {
    try {
      const today = dayKey(new Date());
      const [timer, subjects, allSubjects, overview, goals, exams, week, settings] =
        await Promise.all([
          api.getTimerState(),
          api.listSubjects(false),
          api.listSubjects(true),
          api.todayOverview(),
          api.getGoals(),
          api.listExams(),
          api.statsRange(mondayOf(today), today, 'week'),
          api.getSettings()
        ]);
      this.timer = timer;
      this.syncedAtMs = Date.now();
      this.subjects = subjects;
      this.allSubjects = allSubjects;
      this.today = overview;
      this.goals = goals;
      this.exams = exams;
      this.weekSec = week.totalSec;
      this.settings = settings;
      this.revision += 1;
      this.checkTargetReached();
    } catch (error) {
      this.report(error);
    }
  }

  async refreshToday() {
    try {
      this.today = await api.todayOverview();
      this.checkTargetReached();
    } catch (error) {
      this.report(error);
    }
  }

  // ------------------------------------------------------------------- timer

  async start(subjectId: number) {
    await this.guard(async () => {
      this.timer = await api.startSession(subjectId);
      this.syncedAtMs = Date.now();
      await this.refreshToday();
    });
  }

  async pause() {
    await this.guard(async () => {
      this.timer = await api.pauseSession();
      this.syncedAtMs = Date.now();
    });
  }

  async resume() {
    await this.guard(async () => {
      this.timer = await api.resumeSession();
      this.syncedAtMs = Date.now();
    });
  }

  async toggle() {
    if (this.timer.phase === 'running') await this.pause();
    else if (this.timer.phase === 'paused') await this.resume();
    else {
      const subject = this.activeSubject ?? this.subjects[0];
      if (subject) await this.start(subject.id);
      else this.view = 'subjects';
    }
  }

  async stop() {
    await this.guard(async () => {
      const saved = await api.stopSession();
      this.timer = await api.getTimerState();
      this.syncedAtMs = Date.now();
      await this.refreshToday();
      this.showToast({
        text: m.toast.sessionSaved,
        actions: [
          { label: m.toast.addNote, run: () => this.openSession(this.today.date || dayKey(new Date()), saved) },
          {
            label: m.toast.undo,
            run: async () => {
              await api.undoStop(saved.id);
              await this.refreshAll();
              this.showToast({ text: m.toast.undone, actions: [] });
            }
          }
        ]
      });
    });
  }

  async finishOrphan(endedAt: number) {
    await this.guard(async () => {
      try {
        await api.finishOrphanSession(endedAt);
      } catch (error) {
        // Sesi bisa sudah ditutup dari tray atau pintasan global sebelum dialog
        // dijawab. Kalau timer sudah diam, dialog tidak perlu bertahan.
        await this.refreshAll();
        if (this.timer.phase !== 'idle') throw error;
      }
      this.dialog = null;
      await this.refreshAll();
    });
  }

  async discardOrphan(sessionId: number) {
    await this.guard(async () => {
      try {
        await api.discardSession(sessionId);
      } catch (error) {
        await this.refreshAll();
        if (this.timer.phase !== 'idle') throw error;
      }
      this.dialog = null;
      await this.refreshAll();
    });
  }

  async trimIdle(minutes: number, sessionId: number) {
    await this.guard(async () => {
      this.timer = await api.trimSession(sessionId, minutes * 60);
      this.syncedAtMs = Date.now();
    });
  }

  async openMini() {
    if (this.preview) {
      this.showToast({ text: m.settings.notSupported, actions: [] }, 5000);
      return;
    }
    await this.guard(async () => {
      await api.openMini();
    });
  }

  async backToMain() {
    if (this.preview) return;
    await this.guard(async () => {
      await api.focusMain();
    });
  }

  async setMiniOnTop(flag: boolean) {
    if (this.preview) return;
    await this.guard(async () => {
      await api.setMiniAlwaysOnTop(flag);
      this.settings = { ...this.settings, miniAlwaysOnTop: flag };
    });
  }

  async setGoal(scope: 'daily' | 'weekly', targetSec: number) {
    await this.guard(async () => {
      this.goals = await api.setGoal(scope, Math.max(300, Math.round(targetSec)));
    });
  }

  // -------------------------------------------------------------------- sesi

  openSession(date: string, session: SessionView | null = null) {
    this.dialog = { kind: 'session', session, date };
  }

  openSubject(subject: Subject | null = null) {
    this.dialog = { kind: 'subject', subject };
  }

  openExam(exam: ExamView | null = null) {
    this.dialog = { kind: 'exam', exam };
  }

  confirm(options: {
    title: string;
    body: string;
    confirmLabel: string;
    danger?: boolean;
    run: () => void | Promise<void>;
  }) {
    this.dialog = {
      kind: 'confirm',
      title: options.title,
      body: options.body,
      confirmLabel: options.confirmLabel,
      danger: options.danger ?? false,
      run: options.run
    };
  }

  closeDialog() {
    this.dialog = null;
  }

  // ------------------------------------------------------------------ toast

  showToast(toast: ToastState, ms = 8000) {
    this.toast = toast;
    if (this.toastTimer !== null) window.clearTimeout(this.toastTimer);
    this.toastTimer = window.setTimeout(() => {
      this.toast = null;
      this.toastTimer = null;
    }, ms);
  }

  dismissToast() {
    this.toast = null;
    if (this.toastTimer !== null) window.clearTimeout(this.toastTimer);
    this.toastTimer = null;
  }

  // ------------------------------------------------------------- pengaturan

  async updateSetting(key: keyof Settings, value: string | number | boolean) {
    await this.guard(async () => {
      this.settings = await api.setSetting(key, String(value));
    });
  }
  // ---------------------------------------------------------------- internal

  async guard(work: () => Promise<void>) {
    if (this.busy) return;
    this.busy = true;
    try {
      await work();
      this.error = null;
    } catch (error) {
      this.report(error);
    } finally {
      this.busy = false;
    }
  }

  report(error: unknown) {
    const text = error instanceof Error ? error.message : String(error);
    this.error = text;
    this.showToast({ text, actions: [] }, 6000);
  }

  private checkTargetReached() {
    const today = dayKey(new Date());
    if (!this.targetReached || this.targetNotified === today) return;
    this.targetNotified = today;
    this.showToast({ text: m.toast.targetReached, actions: [] });
    void api
      .notify(m.app.name, m.toast.targetReached)
      .catch(() => undefined);
  }

  private startIdleWatch() {
    if (this.settings.idleThresholdMin <= 0) return;
    window.setInterval(async () => {
      if (this.timer.phase !== 'running' || this.dialog) return;
      const threshold = this.settings.idleThresholdMin * 60;
      try {
        const idle = await api.idleSeconds();
        if (idle < threshold) return;
        if (Date.now() - this.idleDismissedAt < 60_000) return;
        this.idleDismissedAt = Date.now();
        this.dialog = {
          kind: 'idle',
          minutes: this.settings.idleThresholdMin,
          sessionId: this.timer.sessionId ?? 0
        };
      } catch {
        // Deteksi idle tidak tersedia di platform ini.
      }
    }, 30_000);
  }
}

export const store = new AppStore();
export { addDays };
