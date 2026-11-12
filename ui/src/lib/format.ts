import { m } from './i18n';

const pad2 = (n: number) => String(Math.trunc(n)).padStart(2, '0');

/** `01:24:09`. Angka tabular, tidak pernah negatif. */
export function fmtClock(totalSec: number): string {
  const sec = Math.max(0, Math.floor(totalSec));
  const hours = Math.floor(sec / 3600);
  const minutes = Math.floor((sec % 3600) / 60);
  return `${pad2(hours)}:${pad2(minutes)}:${pad2(sec % 60)}`;
}

/** `3 jam 20 menit`, `1 jam`, `45 detik`, `0 menit`. */
export function fmtLong(totalSec: number): string {
  const sec = Math.max(0, Math.round(totalSec));
  if (sec === 0) return '0 menit';
  if (sec < 60) return `${sec} detik`;
  let hours = Math.floor(sec / 3600);
  let minutes = Math.round((sec % 3600) / 60);
  if (minutes === 60) {
    hours += 1;
    minutes = 0;
  }
  if (hours === 0) return `${minutes} menit`;
  if (minutes === 0) return `${hours} jam`;
  return `${hours} jam ${minutes} menit`;
}

/** `3j 20m`, untuk tempat sempit. */
export function fmtShort(totalSec: number): string {
  const sec = Math.max(0, Math.round(totalSec));
  const hours = Math.floor(sec / 3600);
  const minutes = Math.round((sec % 3600) / 60);
  if (hours === 0) return `${minutes}${m.time.minuteShort}`;
  if (minutes === 0) return `${hours}${m.time.hourShort}`;
  return `${hours}${m.time.hourShort} ${minutes}${m.time.minuteShort}`;
}

export function fmtTime(unixSec: number): string {
  const date = new Date(unixSec * 1000);
  return m.clock(date.getHours(), date.getMinutes());
}

export function fmtRange(startUnix: number, endUnix: number): string {
  return `${fmtTime(startUnix)}–${fmtTime(endUnix)}`;
}

/** `2026-11-10` menjadi `Selasa, 10 November`. */
export function fmtDayTitle(key: string): string {
  const date = parseDayKey(key);
  const weekday = m.dayNames[(date.getDay() + 6) % 7];
  return `${weekday}, ${date.getDate()} ${m.monthNames[date.getMonth()]}`;
}

export function fmtDateShort(key: string): string {
  const date = parseDayKey(key);
  return `${date.getDate()} ${m.monthNames[date.getMonth()].slice(0, 3)}`;
}

export function fmtDaysLeft(days: number): string {
  if (days === 0) return m.targets.today;
  if (days > 0) return m.targets.daysLeft(days);
  return m.targets.overdue(Math.abs(days));
}

export function fmtPercent(part: number, whole: number): number {
  if (whole <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round((part / whole) * 100)));
}

/** Kunci tanggal lokal `YYYY-MM-DD`. */
export function dayKey(date: Date): string {
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`;
}

export function parseDayKey(key: string): Date {
  const [year, month, day] = key.split('-').map(Number);
  return new Date(year, (month ?? 1) - 1, day ?? 1);
}

export function addDays(key: string, days: number): string {
  const date = parseDayKey(key);
  date.setDate(date.getDate() + days);
  return dayKey(date);
}

/** `09:15` pada input jam menjadi timestamp lokal hari ini. */
export function combineDayAndTime(key: string, time: string): number {
  const date = parseDayKey(key);
  const [hours, minutes] = time.split(':').map(Number);
  date.setHours(hours ?? 0, minutes ?? 0, 0, 0);
  return Math.floor(date.getTime() / 1000);
}

export function timeValue(unixSec: number): string {
  const date = new Date(unixSec * 1000);
  return `${pad2(date.getHours())}:${pad2(date.getMinutes())}`;
}
