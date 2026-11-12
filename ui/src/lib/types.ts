export type ColorToken =
  | 'rose'
  | 'butter'
  | 'tangerine'
  | 'lime'
  | 'mint'
  | 'aqua'
  | 'sky'
  | 'lavender';

export interface Subject {
  id: number;
  name: string;
  color: ColorToken;
  archived: boolean;
  createdAt: number;
}

export type SessionSource = 'timer' | 'manual';

export interface Session {
  id: number;
  subjectId: number;
  startedAt: number;
  endedAt: number | null;
  pausedAt: number | null;
  pausedTotalSec: number;
  durationSec: number | null;
  note: string | null;
  source: SessionSource;
}

export interface SessionView extends Session {
  subjectName: string;
  subjectColor: ColorToken;
  effectiveDurationSec: number;
}

export type TimerPhase = 'idle' | 'running' | 'paused';

export interface TimerSnapshot {
  phase: TimerPhase;
  sessionId: number | null;
  subjectId: number | null;
  subjectName: string | null;
  subjectColor: ColorToken | null;
  startedAt: number | null;
  pausedAt: number | null;
  pausedTotalSec: number;
  elapsedSec: number;
}

export interface Goals {
  dailySec: number;
  weeklySec: number;
}

export interface Exam {
  id: number;
  title: string;
  examDate: string;
}

export interface ExamView extends Exam {
  daysLeft: number;
}

export interface BucketPart {
  subjectId: number;
  subjectName: string;
  color: ColorToken;
  secs: number;
}

export interface Bucket {
  label: string;
  date: string;
  totalSec: number;
  parts: BucketPart[];
}

export interface RangeStats {
  from: string;
  to: string;
  totalSec: number;
  previousTotalSec: number;
  sessionCount: number;
  bestDaySec: number;
  bestDayDate: string | null;
  buckets: Bucket[];
}

export interface SubjectTotal {
  subjectId: number;
  subjectName: string;
  color: ColorToken;
  secs: number;
  sessionCount: number;
}

export interface HeatCell {
  date: string;
  totalSec: number;
  level: 0 | 1 | 2 | 3 | 4;
}

export interface Heatmap {
  weeks: HeatCell[][];
  maxSec: number;
}

export interface Streaks {
  currentDays: number;
  longestDays: number;
  bestDaySec: number;
  bestDayDate: string | null;
  bestWeekSec: number;
  bestWeekStart: string | null;
}

export interface DaySummary {
  date: string;
  totalSec: number;
  sessions: SessionView[];
}

export interface DayTotal {
  date: string;
  totalSec: number;
}

export interface ManualSessionInput {
  subjectId: number;
  startedAt: number;
  endedAt: number;
  note: string | null;
}

export interface Settings {
  dayStartHour: number;
  idleThresholdMin: number;
  countSleepTime: boolean;
  closeToTray: boolean;
  autostart: boolean;
  theme: 'light' | 'dark' | 'system';
  uiScale: number;
  globalShortcut: string;
  rememberWindow: boolean;
  language: 'id';
  miniAlwaysOnTop: boolean;
}

export interface AppInfo {
  version: string;
  dataDir: string;
  databasePath: string;
  platform: string;
}

export interface ImportReport {
  subjectsAdded: number;
  sessionsAdded: number;
  examsAdded: number;
  goalsUpdated: number;
}

export const COLORS: ColorToken[] = [
  'rose',
  'butter',
  'tangerine',
  'lime',
  'mint',
  'aqua',
  'sky',
  'lavender'
];
