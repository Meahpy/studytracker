/**
 * Backend pratinjau untuk `npm run dev` di browser, dipakai hanya di luar Tauri.
 * Tujuannya menilai tampilan dan alur, bukan menggantikan `crates/core`.
 * Data disimpan di localStorage; jalankan `npm run tauri dev` untuk database asli.
 */

import type {
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

const KEY = 'studytracker-preview-v1';

const listeners = new Set<() => void>();

export function subscribe(cb: () => void): () => void {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
}

function emit() {
  for (const listener of listeners) listener();
}

interface Data {
  subjects: Subject[];
  sessions: Session[];
  goals: Goals;
  exams: Exam[];
  settings: Settings;
  seq: number;
}

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

const now = () => Math.floor(Date.now() / 1000);
const pad = (n: number) => String(n).padStart(2, '0');

function keyOf(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function dayStartTs(key: string, hour: number): number {
  const [y, mo, d] = key.split('-').map(Number);
  return Math.floor(new Date(y, mo - 1, d, hour, 0, 0).getTime() / 1000);
}

function studyDay(ts: number, hour: number): string {
  return keyOf(new Date((ts - hour * 3600) * 1000));
}

function shiftKey(key: string, days: number): string {
  const [y, mo, d] = key.split('-').map(Number);
  const date = new Date(y, mo - 1, d);
  date.setDate(date.getDate() + days);
  return keyOf(date);
}

function seed(): Data {
  const current = now();
  const today = studyDay(current, 4);
  const subjects: Subject[] = [
    { id: 1, name: 'Matematika', color: 'rose', archived: false, createdAt: current - 86400 * 40 },
    { id: 2, name: 'Fisika', color: 'sky', archived: false, createdAt: current - 86400 * 38 },
    { id: 3, name: 'Bahasa Inggris', color: 'mint', archived: false, createdAt: current - 86400 * 30 }
  ];

  const sessions: Session[] = [];
  let id = 1;
  const pattern = [0, 2, 3, 1, 4, 2, 0, 3, 2, 1, 4, 3, 2, 0, 1, 3, 4, 2, 1, 3];
  for (let back = 20; back >= 1; back -= 1) {
    const day = shiftKey(today, -back);
    const blocks = pattern[back % pattern.length] ?? 0;
    for (let block = 0; block < blocks; block += 1) {
      const subject = subjects[(back + block) % subjects.length];
      const startHour = 8 + block * 3;
      const startedAt = dayStartTs(day, 0) + startHour * 3600 + (block % 2) * 600;
      const duration = 2400 + ((back * 300 + block * 420) % 3000);
      sessions.push({
        id: id++,
        subjectId: subject.id,
        startedAt,
        endedAt: startedAt + duration,
        pausedAt: null,
        pausedTotalSec: 0,
        durationSec: duration,
        note: block === 0 ? 'Bab 3, latihan soal' : null,
        source: 'timer'
      });
    }
  }

  return {
    subjects,
    sessions,
    goals: { dailySec: 4 * 3600, weeklySec: 20 * 3600 },
    exams: [
      { id: 1, title: 'Ujian tengah semester', examDate: shiftKey(today, 23) },
      { id: 2, title: 'Olimpiade fisika', examDate: shiftKey(today, 45) }
    ],
    settings: { ...defaultSettings },
    seq: id
  };
}

function load(): Data {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return JSON.parse(raw) as Data;
  } catch {
    // localStorage penuh atau rusak: mulai dari data contoh.
  }
  const data = seed();
  save(data);
  return data;
}

function save(data: Data) {
  try {
    localStorage.setItem(KEY, JSON.stringify(data));
  } catch {
    // Pratinjau tetap jalan walau tidak bisa menyimpan.
  }
}

function subjectOf(data: Data, id: number): Subject {
  const subject = data.subjects.find((s) => s.id === id);
  if (!subject) throw new Error('Mata pelajaran tidak ditemukan.');
  return subject;
}

function view(data: Data, session: Session): SessionView {
  const subject = subjectOf(data, session.subjectId);
  return {
    ...session,
    subjectName: subject.name,
    subjectColor: subject.color,
    effectiveDurationSec:
      session.durationSec ??
      Math.max(0, now() - session.startedAt - session.pausedTotalSec)
  };
}

function active(data: Data): Session | undefined {
  return data.sessions.find((s) => s.endedAt === null);
}

function snapshot(data: Data): TimerSnapshot {
  const session = active(data);
  if (!session) {
    return {
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
  }
  const subject = subjectOf(data, session.subjectId);
  const end = session.pausedAt ?? now();
  return {
    phase: session.pausedAt ? 'paused' : 'running',
    sessionId: session.id,
    subjectId: subject.id,
    subjectName: subject.name,
    subjectColor: subject.color,
    startedAt: session.startedAt,
    pausedAt: session.pausedAt,
    pausedTotalSec: session.pausedTotalSec,
    elapsedSec: Math.max(0, end - session.startedAt - session.pausedTotalSec)
  };
}

/** Bagi durasi efektif tiap sesi ke hari belajar yang dilaluinya. */
function aggregate(data: Data, fromTs: number, toTs: number): Map<string, Map<number, number>> {
  const hour = data.settings.dayStartHour;
  const out = new Map<string, Map<number, number>>();

  for (const session of data.sessions) {
    const end = session.endedAt ?? now();
    if (!(session.startedAt < toTs && end > fromTs)) continue;
    const wall = end - session.startedAt;
    const effective = Math.max(0, wall - session.pausedTotalSec);
    if (effective <= 0) continue;

    const first = studyDay(session.startedAt, hour);
    const last = studyDay(end - 1, hour);
    const parts: Array<{ day: string; overlap: number }> = [];
    let cursor = first;
    for (let guard = 0; guard < 400; guard += 1) {
      const start = dayStartTs(cursor, hour);
      const next = dayStartTs(shiftKey(cursor, 1), hour);
      parts.push({
        day: cursor,
        overlap: Math.max(0, Math.min(end, next) - Math.max(session.startedAt, start))
      });
      if (cursor === last) break;
      cursor = shiftKey(cursor, 1);
    }

    let remaining = effective;
    parts.forEach((part, index) => {
      const share =
        index === parts.length - 1
          ? remaining
          : Math.round((effective * part.overlap) / wall);
      remaining -= share;
      if (share === 0) return;
      const dayMap = out.get(part.day) ?? new Map<number, number>();
      dayMap.set(session.subjectId, (dayMap.get(session.subjectId) ?? 0) + share);
      out.set(part.day, dayMap);
    });
  }
  return out;
}

function daySeries(data: Data, from: string, to: string): Map<string, number> {
  const hour = data.settings.dayStartHour;
  const map = aggregate(data, dayStartTs(from, hour), dayStartTs(shiftKey(to, 1), hour));
  const series = new Map<string, number>();
  for (const [day, parts] of map) {
    let total = 0;
    for (const secs of parts.values()) total += secs;
    series.set(day, total);
  }
  return series;
}

function heatLevel(secs: number, target: number): 0 | 1 | 2 | 3 | 4 {
  if (secs <= 0) return 0;
  if (target <= 0) return 4;
  const ratio = secs / target;
  if (ratio >= 1) return 4;
  if (ratio >= 0.5) return 3;
  if (ratio >= 0.25) return 2;
  return 1;
}

function daySummary(data: Data, date: string): DaySummary {
  const hour = data.settings.dayStartHour;
  const from = dayStartTs(date, hour);
  const to = dayStartTs(shiftKey(date, 1), hour);
  const sessions = data.sessions
    .filter((s) => {
      const end = s.endedAt ?? now();
      return s.startedAt < to && end > from;
    })
    .sort((a, b) => a.startedAt - b.startedAt)
    .map((s) => view(data, s));
  return {
    date,
    totalSec: sessions.reduce((sum, s) => sum + s.effectiveDurationSec, 0),
    sessions
  };
}

function breakdown(data: Data, from: string, to: string): SubjectTotal[] {
  const hour = data.settings.dayStartHour;
  const map = aggregate(data, dayStartTs(from, hour), dayStartTs(shiftKey(to, 1), hour));
  const totals = new Map<number, number>();
  for (const parts of map.values()) {
    for (const [subjectId, secs] of parts) {
      totals.set(subjectId, (totals.get(subjectId) ?? 0) + secs);
    }
  }
  return data.subjects
    .filter((subject) => (totals.get(subject.id) ?? 0) > 0)
    .map((subject) => ({
      subjectId: subject.id,
      subjectName: subject.name,
      color: subject.color,
      secs: totals.get(subject.id) ?? 0,
      sessionCount: data.sessions.filter((s) => s.subjectId === subject.id).length
    }))
    .sort((a, b) => b.secs - a.secs);
}

function statsRange(data: Data, from: string, to: string, granularity: string): RangeStats {
  const hour = data.settings.dayStartHour;
  const map = aggregate(data, dayStartTs(from, hour), dayStartTs(shiftKey(to, 1), hour));
  const dayNames = ['Sen', 'Sel', 'Rab', 'Kam', 'Jum', 'Sab', 'Min'];
  const buckets: RangeStats['buckets'] = [];
  let cursor = from;
  for (let guard = 0; guard < 400; guard += 1) {
    const parts = map.get(cursor) ?? new Map<number, number>();
    const bucketParts = [...parts.entries()]
      .map(([subjectId, secs]) => {
        const subject = subjectOf(data, subjectId);
        return {
          subjectId,
          subjectName: subject.name,
          color: subject.color,
          secs
        };
      })
      .filter((part) => part.secs > 0)
      .sort((a, b) => b.secs - a.secs);
    const date = new Date(dayStartTs(cursor, 12) * 1000);
    buckets.push({
      label:
        granularity === 'day'
          ? (dayNames[(date.getDay() + 6) % 7] ?? '')
          : String(date.getDate()),
      date: cursor,
      totalSec: bucketParts.reduce((sum, part) => sum + part.secs, 0),
      parts: bucketParts
    });
    if (cursor === to) break;
    cursor = shiftKey(cursor, 1);
  }

  const span = buckets.length;
  const previousTo = shiftKey(from, -1);
  const previousFrom = shiftKey(previousTo, -(span - 1));
  const previous = daySeries(data, previousFrom, previousTo);
  const totalSec = buckets.reduce((sum, bucket) => sum + bucket.totalSec, 0);
  const best = buckets.reduce(
    (top, bucket) => (bucket.totalSec > top.totalSec ? bucket : top),
    buckets[0]
  );

  return {
    from,
    to,
    totalSec,
    previousTotalSec: [...previous.values()].reduce((sum, secs) => sum + secs, 0),
    sessionCount: data.sessions.filter((s) => {
      const end = s.endedAt ?? now();
      return s.startedAt < dayStartTs(shiftKey(to, 1), hour) && end > dayStartTs(from, hour);
    }).length,
    bestDaySec: best?.totalSec ?? 0,
    bestDayDate: best?.date ?? null,
    buckets
  };
}

function heatmap(data: Data, weeks: number): Heatmap {
  const today = studyDay(now(), data.settings.dayStartHour);
  const [y, mo, d] = today.split('-').map(Number);
  const monday = new Date(y, mo - 1, d);
  monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7));
  const firstMonday = new Date(monday);
  firstMonday.setDate(firstMonday.getDate() - (weeks - 1) * 7);

  const series = daySeries(data, keyOf(firstMonday), today);
  const target = data.goals.dailySec;
  const columns: Heatmap['weeks'] = [];
  let maxSec = 0;
  for (let w = 0; w < weeks; w += 1) {
    const column: Heatmap['weeks'][number] = [];
    for (let d2 = 0; d2 < 7; d2 += 1) {
      const date = new Date(firstMonday);
      date.setDate(date.getDate() + w * 7 + d2);
      const key = keyOf(date);
      const totalSec = series.get(key) ?? 0;
      maxSec = Math.max(maxSec, totalSec);
      column.push({ date: key, totalSec, level: heatLevel(totalSec, target) });
    }
    columns.push(column);
  }
  return { weeks: columns, maxSec };
}

function streaks(data: Data): Streaks {
  const today = studyDay(now(), data.settings.dayStartHour);
  const first = data.sessions.reduce(
    (min, session) => Math.min(min, session.startedAt),
    now()
  );
  const series = daySeries(data, studyDay(first, data.settings.dayStartHour), today);
  const activeDays = [...series.entries()]
    .filter(([, secs]) => secs > 0)
    .map(([day]) => day)
    .sort();

  let current = 0;
  let cursor = series.get(today) ? today : shiftKey(today, -1);
  while (activeDays.includes(cursor)) {
    current += 1;
    cursor = shiftKey(cursor, -1);
  }

  let longest = 0;
  let run = 0;
  let previous: string | null = null;
  for (const day of activeDays) {
    run = previous && shiftKey(previous, 1) === day ? run + 1 : 1;
    longest = Math.max(longest, run);
    previous = day;
  }

  const weeks = new Map<string, number>();
  for (const [day, secs] of series) {
    const [y, mo, d] = day.split('-').map(Number);
    const date = new Date(y, mo - 1, d);
    date.setDate(date.getDate() - ((date.getDay() + 6) % 7));
    const week = keyOf(date);
    weeks.set(week, (weeks.get(week) ?? 0) + secs);
  }
  const bestDay = [...series.entries()].sort((a, b) => b[1] - a[1])[0];
  const bestWeek = [...weeks.entries()].sort((a, b) => b[1] - a[1])[0];

  return {
    currentDays: current,
    longestDays: longest,
    bestDaySec: bestDay?.[1] ?? 0,
    bestDayDate: bestDay?.[0] ?? null,
    bestWeekSec: bestWeek?.[1] ?? 0,
    bestWeekStart: bestWeek?.[0] ?? null
  };
}

function validateSettings(key: string, value: string): string {
  const ok =
    key === 'uiScale'
      ? ['90', '100', '110', '125'].includes(value)
      : key === 'theme'
        ? ['light', 'dark', 'system'].includes(value)
        : key === 'idleThresholdMin'
          ? ['0', '5', '10', '15'].includes(value)
          : ['closeToTray', 'autostart', 'rememberWindow', 'miniAlwaysOnTop', 'countSleepTime'].includes(
              key
            )
            ? ['true', 'false'].includes(value)
            : key === 'dayStartHour'
              ? Number(value) >= 0 && Number(value) <= 12
              : key === 'globalShortcut'
                ? value.trim().length > 0
                : false;
  if (!ok) throw new Error('Nilai pengaturan tidak dikenal.');
  return value;
}

export function createMockBackend() {
  return async (name: string, args: Record<string, unknown> = {}): Promise<unknown> => {
    const data = load();
    const num = (key: string) => Number(args[key] ?? 0);
    const str = (key: string) => String(args[key] ?? '');

    switch (name) {
      case 'get_timer_state':
        return snapshot(data);

      case 'start_session': {
        if (active(data)) throw new Error('Masih ada sesi yang berjalan.');
        const subject = subjectOf(data, num('subjectId'));
        if (subject.archived) throw new Error('Mata pelajaran ini sudah diarsipkan.');
        data.sessions.push({
          id: data.seq++,
          subjectId: subject.id,
          startedAt: now(),
          endedAt: null,
          pausedAt: null,
          pausedTotalSec: 0,
          durationSec: null,
          note: null,
          source: 'timer'
        });
        save(data);
        emit();
        return snapshot(data);
      }

      case 'pause_session':
      case 'resume_session': {
        const session = active(data);
        if (!session) throw new Error('Belum ada sesi yang berjalan.');
        if (name === 'pause_session') {
          if (session.pausedAt) throw new Error('Timer sedang jeda.');
          session.pausedAt = now();
        } else {
          if (!session.pausedAt) throw new Error('Timer tidak sedang jeda.');
          session.pausedTotalSec += Math.max(0, now() - session.pausedAt);
          session.pausedAt = null;
        }
        save(data);
        emit();
        return snapshot(data);
      }

      case 'stop_session': {
        const session = active(data);
        if (!session) throw new Error('Belum ada sesi yang berjalan.');
        const endedAt = session.pausedAt ?? now();
        session.endedAt = endedAt;
        session.pausedAt = null;
        session.durationSec = Math.max(0, endedAt - session.startedAt - session.pausedTotalSec);
        save(data);
        emit();
        return view(data, session);
      }

      case 'undo_stop': {
        const index = data.sessions.findIndex((s) => s.id === num('sessionId'));
        if (index < 0) throw new Error('Sesi tidak ditemukan.');
        data.sessions.splice(index, 1);
        save(data);
        emit();
        return null;
      }

      case 'discard_session': {
        data.sessions = data.sessions.filter(
          (s) => !(s.id === num('sessionId') && s.endedAt === null)
        );
        save(data);
        emit();
        return null;
      }

      case 'trim_session': {
        const session = active(data);
        if (!session || session.id !== num('sessionId')) {
          throw new Error('Sesi tidak ditemukan.');
        }
        const end = session.pausedAt ?? now();
        const running = Math.max(0, end - session.startedAt - session.pausedTotalSec);
        session.pausedTotalSec += Math.min(Math.max(0, num('secs')), running);
        save(data);
        emit();
        return snapshot(data);
      }

      case 'finish_orphan_session': {
        const session = active(data);
        if (!session) throw new Error('Belum ada sesi yang berjalan.');
        const endedAt = num('endedAt');
        if (endedAt < session.startedAt) throw new Error('Waktu selesai lebih awal dari waktu mulai.');
        session.endedAt = endedAt;
        session.pausedAt = null;
        session.durationSec = Math.max(0, endedAt - session.startedAt - session.pausedTotalSec);
        save(data);
        emit();
        return session;
      }

      case 'list_sessions': {
        const hour = data.settings.dayStartHour;
        const from = dayStartTs(str('from'), hour);
        const to = dayStartTs(shiftKey(str('to'), 1), hour);
        return data.sessions
          .filter((s) => {
            const end = s.endedAt ?? now();
            return s.startedAt < to && end > from;
          })
          .sort((a, b) => a.startedAt - b.startedAt)
          .map((s) => view(data, s));
      }

      case 'day_summary':
        return daySummary(data, str('date'));

      case 'today_overview':
        return daySummary(data, studyDay(now(), data.settings.dayStartHour));

      case 'day_totals': {
        const series = daySeries(data, str('from'), str('to'));
        const rows: DayTotal[] = [];
        for (const [date, totalSec] of series) rows.push({ date, totalSec });
        return rows.sort((a, b) => a.date.localeCompare(b.date));
      }

      case 'add_manual_session': {
        const input = args.input as ManualSessionInput;
        subjectOf(data, input.subjectId);
        if (input.endedAt < input.startedAt) throw new Error('Waktu selesai lebih awal dari waktu mulai.');
        const session: Session = {
          id: data.seq++,
          subjectId: input.subjectId,
          startedAt: input.startedAt,
          endedAt: input.endedAt,
          pausedAt: null,
          pausedTotalSec: 0,
          durationSec: input.endedAt - input.startedAt,
          note: input.note,
          source: 'manual'
        };
        data.sessions.push(session);
        save(data);
        emit();
        return session;
      }

      case 'update_session': {
        const id = num('sessionId');
        const input = args.input as ManualSessionInput;
        const session = data.sessions.find((s) => s.id === id);
        if (!session) throw new Error('Sesi tidak ditemukan.');
        if (input.endedAt < input.startedAt) throw new Error('Waktu selesai lebih awal dari waktu mulai.');
        Object.assign(session, {
          subjectId: input.subjectId,
          startedAt: input.startedAt,
          endedAt: input.endedAt,
          pausedAt: null,
          pausedTotalSec: 0,
          durationSec: input.endedAt - input.startedAt,
          note: input.note
        });
        save(data);
        emit();
        return session;
      }

      case 'delete_session': {
        data.sessions = data.sessions.filter((s) => s.id !== num('sessionId'));
        save(data);
        emit();
        return null;
      }

      case 'has_overlap': {
        const startedAt = num('startedAt');
        const endedAt = num('endedAt');
        const excludeId = args.excludeId as number | null;
        return data.sessions.some((s) => {
          if (excludeId !== null && s.id === excludeId) return false;
          const end = s.endedAt ?? now();
          return s.startedAt < endedAt && end > startedAt;
        });
      }

      case 'list_subjects':
        return data.subjects
          .filter((s) => args.includeArchived === true || !s.archived)
          .sort((a, b) => a.name.localeCompare(b.name));

      case 'upsert_subject': {
        const id = args.id as number | null;
        const subjectName = str('name').trim();
        const color = str('color');
        if (!subjectName) throw new Error('Nama mata pelajaran tidak boleh kosong.');
        const duplicate = data.subjects.some(
          (s) => s.id !== id && s.name.toLowerCase() === subjectName.toLowerCase()
        );
        if (duplicate) throw new Error(`Mata pelajaran "${subjectName}" sudah ada.`);
        if (id === null) {
          const subject: Subject = {
            id: data.seq++,
            name: subjectName,
            color: color as Subject['color'],
            archived: false,
            createdAt: now()
          };
          data.subjects.push(subject);
          save(data);
          emit();
          return subject;
        }
        const subject = subjectOf(data, id);
        subject.name = subjectName;
        subject.color = color as Subject['color'];
        save(data);
        emit();
        return subject;
      }

      case 'archive_subject': {
        const subject = subjectOf(data, num('subjectId'));
        subject.archived = args.archived === true;
        save(data);
        emit();
        return subject;
      }

      case 'delete_subject': {
        const id = num('subjectId');
        if (data.sessions.some((s) => s.subjectId === id)) {
          throw new Error('Mata pelajaran ini punya riwayat sesi, jadi tidak bisa dihapus.');
        }
        data.subjects = data.subjects.filter((s) => s.id !== id);
        save(data);
        emit();
        return null;
      }

      case 'subject_totals': {
        const totals = new Map<number, { secs: number; count: number }>();
        for (const session of data.sessions) {
          const entry = totals.get(session.subjectId) ?? { secs: 0, count: 0 };
          entry.secs += session.durationSec ?? 0;
          if (session.endedAt !== null) entry.count += 1;
          totals.set(session.subjectId, entry);
        }
        return data.subjects
          .map((subject) => ({
            subjectId: subject.id,
            subjectName: subject.name,
            color: subject.color,
            secs: totals.get(subject.id)?.secs ?? 0,
            sessionCount: totals.get(subject.id)?.count ?? 0
          }))
          .sort((a, b) => a.subjectName.localeCompare(b.subjectName));
      }

      case 'stats_range':
        return statsRange(data, str('from'), str('to'), str('granularity'));

      case 'subject_breakdown':
        return breakdown(data, str('from'), str('to'));

      case 'heatmap':
        return heatmap(data, Number(args.weeks ?? 26));

      case 'streaks':
        return streaks(data);

      case 'get_goals':
        return data.goals;

      case 'set_goal': {
        const target = num('targetSec');
        if (target <= 0) throw new Error('Target harus lebih dari nol.');
        if (str('scope') === 'weekly') data.goals.weeklySec = target;
        else data.goals.dailySec = target;
        save(data);
        emit();
        return data.goals;
      }

      case 'list_exams': {
        const today = studyDay(now(), data.settings.dayStartHour);
        return data.exams
          .slice()
          .sort((a, b) => a.examDate.localeCompare(b.examDate))
          .map(
            (exam): ExamView => ({
              ...exam,
              daysLeft: Math.round(
                (dayStartTs(exam.examDate, 12) - dayStartTs(today, 12)) / 86400
              )
            })
          );
      }

      case 'upsert_exam': {
        const id = args.id as number | null;
        const title = str('title').trim();
        const examDate = str('examDate');
        if (!title) throw new Error('Judul D-day tidak boleh kosong.');
        if (!/^\d{4}-\d{2}-\d{2}$/.test(examDate)) throw new Error('Tanggal tidak dikenali.');
        if (id === null) {
          const exam: Exam = { id: data.seq++, title, examDate };
          data.exams.push(exam);
          save(data);
          emit();
          return exam;
        }
        const exam = data.exams.find((e) => e.id === id);
        if (!exam) throw new Error('D-day tidak ditemukan.');
        exam.title = title;
        exam.examDate = examDate;
        save(data);
        emit();
        return exam;
      }

      case 'delete_exam':
        data.exams = data.exams.filter((e) => e.id !== num('examId'));
        save(data);
        emit();
        return null;

      case 'export_data': {
        const stamp = new Date().toISOString().slice(0, 16).replace(/[:T]/g, '-');
        return `pratinjau/studytracker-${stamp}.${str('format') === 'csv' ? 'csv' : 'json'}`;
      }

      case 'import_data':
        return { subjectsAdded: 0, sessionsAdded: 0, examsAdded: 0, goalsUpdated: 0 } as ImportReport;

      case 'open_data_dir':
        return 'localStorage';

      case 'backup_now':
        return 'pratinjau/cadangan.db';

      case 'get_settings':
        return data.settings;

      case 'set_setting': {
        const key = str('key').replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
        const value = validateSettings(key, str('value'));
        const map: Record<string, keyof Settings> = {
          dayStartHour: 'dayStartHour',
          idleThresholdMin: 'idleThresholdMin',
          countSleepTime: 'countSleepTime',
          closeToTray: 'closeToTray',
          autostart: 'autostart',
          theme: 'theme',
          uiScale: 'uiScale',
          globalShortcut: 'globalShortcut',
          rememberWindow: 'rememberWindow',
          miniAlwaysOnTop: 'miniAlwaysOnTop'
        };
        const field = map[key];
        if (!field) throw new Error('Nilai pengaturan tidak dikenal.');
        const target = data.settings as unknown as Record<string, unknown>;
        target[field] =
          value === 'true' ? true : value === 'false' ? false : Number.isNaN(Number(value)) ? value : Number(value);
        save(data);
        emit();
        return data.settings;
      }

      case 'app_info':
        return {
          version: '0.1.0',
          dataDir: 'localStorage',
          databasePath: 'localStorage: studytracker-preview-v1',
          platform: 'pratinjau browser'
        };

      case 'idle_seconds':
        return 0;

      case 'notify':
      case 'open_mini':
      case 'close_mini':
      case 'set_mini_always_on_top':
      case 'focus_main':
        return null;

      default:
        throw new Error(`Command ${name} belum ada di backend pratinjau.`);
    }
  };
}
