import { describe, expect, it } from 'vitest';
import {
  addDays,
  combineDayAndTime,
  dayKey,
  fmtClock,
  fmtDaysLeft,
  fmtDayTitle,
  fmtLong,
  fmtPercent,
  fmtShort,
  fmtTime,
  timeValue
} from './format';

describe('fmtClock', () => {
  it('renders hours, minutes, seconds with padding', () => {
    expect(fmtClock(0)).toBe('00:00:00');
    expect(fmtClock(5049)).toBe('01:24:09');
    expect(fmtClock(360000)).toBe('100:00:00');
  });

  it('never goes negative', () => {
    expect(fmtClock(-90)).toBe('00:00:00');
  });
});

describe('fmtLong', () => {
  it('reads as a sentence fragment in Indonesian', () => {
    expect(fmtLong(0)).toBe('0 menit');
    expect(fmtLong(45)).toBe('45 detik');
    expect(fmtLong(3600)).toBe('1 jam');
    expect(fmtLong(12000)).toBe('3 jam 20 menit');
    expect(fmtLong(7200)).toBe('2 jam');
  });

  it('rounds 59.6 minutes up to the next hour', () => {
    expect(fmtLong(3576)).toBe('1 jam');
  });
});

describe('fmtShort', () => {
  it('uses compact suffixes', () => {
    expect(fmtShort(12000)).toBe('3j 20m');
    expect(fmtShort(300)).toBe('5m');
    expect(fmtShort(7200)).toBe('2j');
  });
});

describe('dates', () => {
  it('formats a day title with the Indonesian weekday', () => {
    expect(fmtDayTitle('2026-11-10')).toBe('Selasa, 10 November');
  });

  it('shifts days without leaving the local calendar', () => {
    expect(addDays('2026-03-01', -1)).toBe('2026-02-28');
    expect(addDays('2026-12-31', 1)).toBe('2027-01-01');
  });

  it('formats local time with a dot separator', () => {
    const ts = combineDayAndTime('2026-11-12', '09:12');
    expect(fmtTime(ts)).toBe('09.12');
    expect(timeValue(ts)).toBe('09:12');
  });

  it('keeps the local day key stable through a round trip', () => {
    const ts = combineDayAndTime('2026-11-12', '23:45');
    expect(dayKey(new Date(ts * 1000))).toBe('2026-11-12');
  });
});

describe('countdown and percentages', () => {
  it('speaks the countdown like a person would', () => {
    expect(fmtDaysLeft(0)).toBe('Hari ini');
    expect(fmtDaysLeft(23)).toBe('23 hari lagi');
    expect(fmtDaysLeft(-2)).toBe('Terlewat 2 hari');
  });

  it('clamps progress percentage', () => {
    expect(fmtPercent(1, 2)).toBe(50);
    expect(fmtPercent(0, 0)).toBe(0);
    expect(fmtPercent(5, 2)).toBe(100);
  });
});
