/**
 * Satu-satunya pintu ke backend. Di dalam Tauri memanggil `invoke`;
 * di browser memakai backend pratinjau (dev-only) supaya UI bisa dinilai.
 */

import type {
  AppInfo,
  DaySummary,
  DayTotal,
  Exam,
  ExamView,
  Goals,
  Heatmap,
  ImportReport,
  ManualSessionInput,
  RangeStats,
  Session,
  SessionView,
  Settings,
  Streaks,
  Subject,
  SubjectTotal,
  TimerSnapshot
} from './types';

type Cmd = (name: string, args?: Record<string, unknown>) => Promise<unknown>;

export const inTauri =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
export const isPreview = !inTauri;

let backendPromise: Promise<Cmd> | null = null;

function backend(): Promise<Cmd> {
  if (!backendPromise) {
    backendPromise = inTauri
      ? import('@tauri-apps/api/core').then(({ invoke }) => {
          const call: Cmd = (name, args) => invoke(name, args);
          return call;
        })
      : import('./devmock').then(({ createMockBackend }) => createMockBackend());
  }
  return backendPromise;
}

async function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  const run = await backend();
  return (await run(name, args)) as T;
}

// ------------------------------------------------------------------ pengaturan

/** Backend menyimpan kunci dalam snake_case; UI memakai camelCase. */
const settingKey = (key: string) => key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);

export const api = {
  // timer
  getTimerState: () => cmd<TimerSnapshot>('get_timer_state'),
  startSession: (subjectId: number) => cmd<TimerSnapshot>('start_session', { subjectId }),
  pauseSession: () => cmd<TimerSnapshot>('pause_session'),
  resumeSession: () => cmd<TimerSnapshot>('resume_session'),
  stopSession: () => cmd<SessionView>('stop_session'),
  undoStop: (sessionId: number) => cmd<void>('undo_stop', { sessionId }),
  discardSession: (sessionId: number) => cmd<void>('discard_session', { sessionId }),
  trimSession: (sessionId: number, secs: number) =>
    cmd<TimerSnapshot>('trim_session', { sessionId, secs }),
  finishOrphanSession: (endedAt: number) =>
    cmd<Session>('finish_orphan_session', { endedAt }),

  // sesi
  listSessions: (from: string, to: string) =>
    cmd<SessionView[]>('list_sessions', { from, to }),
  daySummary: (date: string) => cmd<DaySummary>('day_summary', { date }),
  todayOverview: () => cmd<DaySummary>('today_overview'),
  addManualSession: (input: ManualSessionInput) =>
    cmd<Session>('add_manual_session', { input }),
  updateSession: (sessionId: number, input: ManualSessionInput) =>
    cmd<Session>('update_session', { sessionId, input }),
  deleteSession: (sessionId: number) => cmd<void>('delete_session', { sessionId }),
  hasOverlap: (startedAt: number, endedAt: number, excludeId: number | null) =>
    cmd<boolean>('has_overlap', { startedAt, endedAt, excludeId }),
  dayTotals: (from: string, to: string) =>
    cmd<DayTotal[]>('day_totals', { from, to }),

  // mata pelajaran
  listSubjects: (includeArchived = false) =>
    cmd<Subject[]>('list_subjects', { includeArchived }),
  upsertSubject: (id: number | null, name: string, color: string) =>
    cmd<Subject>('upsert_subject', { id, name, color }),
  archiveSubject: (subjectId: number, archived: boolean) =>
    cmd<Subject>('archive_subject', { subjectId, archived }),
  deleteSubject: (subjectId: number) => cmd<void>('delete_subject', { subjectId }),
  subjectTotals: () => cmd<SubjectTotal[]>('subject_totals'),

  // statistik
  statsRange: (from: string, to: string, granularity: string) =>
    cmd<RangeStats>('stats_range', { from, to, granularity }),
  subjectBreakdown: (from: string, to: string) =>
    cmd<SubjectTotal[]>('subject_breakdown', { from, to }),
  heatmap: (weeks = 26) => cmd<Heatmap>('heatmap', { weeks }),
  streaks: () => cmd<Streaks>('streaks'),

  // target dan D-day
  getGoals: () => cmd<Goals>('get_goals'),
  setGoal: (scope: 'daily' | 'weekly', targetSec: number) =>
    cmd<Goals>('set_goal', { scope, targetSec }),
  listExams: () => cmd<ExamView[]>('list_exams'),
  upsertExam: (id: number | null, title: string, examDate: string) =>
    cmd<Exam>('upsert_exam', { id, title, examDate }),
  deleteExam: (examId: number) => cmd<void>('delete_exam', { examId }),

  // data
  exportData: (format: 'json' | 'csv', path?: string | null) =>
    cmd<string>('export_data', { format, path: path ?? null }),
  importData: (path: string) => cmd<ImportReport>('import_data', { path }),
  openDataDir: () => cmd<string>('open_data_dir'),
  backupNow: () => cmd<string>('backup_now'),

  // pengaturan
  getSettings: () => cmd<Settings>('get_settings'),
  setSetting: (key: string, value: string) =>
    cmd<Settings>('set_setting', { key: settingKey(key), value }),

  // jendela dan sistem
  openMini: () => cmd<void>('open_mini'),
  closeMini: () => cmd<void>('close_mini'),
  setMiniAlwaysOnTop: (flag: boolean) =>
    cmd<void>('set_mini_always_on_top', { flag }),
  focusMain: () => cmd<void>('focus_main'),
  idleSeconds: () => cmd<number>('idle_seconds'),
  notify: (title: string, body: string) => cmd<void>('notify', { title, body }),
  appInfo: () => cmd<AppInfo>('app_info')
};

/** Dengarkan `session-changed` dari backend dan dari jendela lain. */
export async function listenSessionChanged(cb: () => void): Promise<() => void> {
  if (inTauri) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen('session-changed', () => cb());
    return () => unlisten();
  }
  const { subscribe } = await import('./devmock');
  return subscribe(cb);
}

/** Label jendela saat ini: `main` atau `mini`. */
export function windowLabel(): string {
  if (!inTauri) return 'main';
  const internals = (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ as
    | { metadata?: { currentWindow?: { label?: string } } }
    | undefined;
  return internals?.metadata?.currentWindow?.label ?? 'main';
}
