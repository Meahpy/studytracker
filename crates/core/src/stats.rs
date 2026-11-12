//! Agregasi statistik: batas hari belajar, pemecahan sesi lintas hari, heatmap, streak.
//!
//! Semua fungsi di sini menerima offset zona waktu sebagai parameter supaya bisa
//! diuji tanpa bergantung pada zona waktu mesin.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone, Utc};

use crate::models::HeatCell;

pub const DEFAULT_DAY_START_HOUR: u32 = 4;

pub fn now_secs() -> i64 {
    Utc::now().timestamp()
}

/// Offset lokal (detik) untuk sebuah timestamp. Jam sistem mundur tidak masalah:
/// nilai ini hanya dipakai untuk memetakan timestamp ke tanggal lokal.
pub fn local_offset_secs(ts: i64) -> i64 {
    match Local.timestamp_opt(ts, 0).single() {
        Some(dt) => dt.offset().local_minus_utc() as i64,
        None => 0,
    }
}

/// Tanggal "hari belajar" untuk sebuah timestamp. Sesi sebelum jam `day_start_hour`
/// dihitung sebagai hari sebelumnya.
pub fn study_day(ts: i64, day_start_hour: u32) -> NaiveDate {
    study_day_with_offset(ts, day_start_hour, local_offset_secs(ts))
}

pub fn study_day_with_offset(ts: i64, day_start_hour: u32, offset_secs: i64) -> NaiveDate {
    // `ts` adalah detik UTC; tambahkan offset supaya yang dihitung adalah jam dinding
    // lokal, baru buang jam sebelum `day_start_hour` ke hari sebelumnya.
    let shifted = ts + offset_secs - i64::from(day_start_hour) * 3600;
    Utc.timestamp_opt(shifted, 0)
        .single()
        .map(|dt| dt.date_naive())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
}

/// Timestamp UTC saat hari belajar `date` dimulai.
pub fn day_start_ts(date: NaiveDate, day_start_hour: u32) -> i64 {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .expect("tengah malam selalu valid")
        + Duration::hours(i64::from(day_start_hour));
    match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt.timestamp(),
        // DST: jam yang hilang -> geser ke kemunculan berikutnya.
        chrono::LocalResult::Ambiguous(dt, _) => dt.timestamp(),
        chrono::LocalResult::None => naive.and_utc().timestamp(),
    }
}

pub fn day_bounds(date: NaiveDate, day_start_hour: u32) -> (i64, i64) {
    (
        day_start_ts(date, day_start_hour),
        day_start_ts(date + Duration::days(1), day_start_hour),
    )
}

/// Awal minggu (Senin) untuk sebuah hari belajar.
pub fn week_start(date: NaiveDate) -> NaiveDate {
    let offset = i64::from(date.weekday().num_days_from_monday());
    date - Duration::days(offset)
}

/// Pecahkan satu sesi ke hari-hari yang dilaluinya, dengan durasi efektif
/// dibagi proporsional terhadap irisan waktu dindingnya.
pub fn split_session(
    started_at: i64,
    ended_at: i64,
    paused_total_sec: i64,
    offset_secs: i64,
    day_start_hour: u32,
    out: &mut BTreeMap<NaiveDate, i64>,
) {
    if ended_at <= started_at {
        return;
    }
    let wall = ended_at - started_at;
    let effective = (wall - paused_total_sec.max(0)).max(0);
    if effective == 0 {
        return;
    }

    let first_day = study_day_with_offset(started_at, day_start_hour, offset_secs);
    let last_day = study_day_with_offset(ended_at - 1, day_start_hour, offset_secs);

    let mut day = first_day;
    let mut remaining = effective;
    let mut last: Option<NaiveDate> = None;
    while day <= last_day {
        // Batas hari dihitung dalam waktu lokal relatif terhadap offset sesi ini,
        // supaya perbandingan tetap konsisten walau offset berubah di tengah sesi.
        let (start_local, end_local) = local_day_bounds(day, offset_secs, day_start_hour);
        let overlap = (ended_at.min(end_local) - started_at.max(start_local)).max(0);
        let share = if wall == 0 { 0 } else { (effective as i128 * overlap as i128 / wall as i128) as i64 };
        *out.entry(day).or_insert(0) += share;
        remaining -= share;
        last = Some(day);
        if day == last_day {
            break;
        }
        day += Duration::days(1);
    }
    // Sisa pembulatan integer masuk ke hari terakhir supaya total tidak hilang.
    if let Some(day) = last {
        let entry = out.entry(day).or_insert(0);
        *entry += remaining.max(0);
    }
}

fn local_day_bounds(date: NaiveDate, offset_secs: i64, day_start_hour: u32) -> (i64, i64) {
    let midnight_local = date
        .and_hms_opt(0, 0, 0)
        .expect("tengah malam selalu valid")
        .and_utc()
        .timestamp()
        + i64::from(day_start_hour) * 3600;
    let start = midnight_local - offset_secs;
    (start, start + 86_400)
}

/// Skala heatmap 0..=4 menurut persen dari target harian.
pub fn heat_level(secs: i64, daily_target_sec: i64) -> u8 {
    if secs <= 0 {
        return 0;
    }
    if daily_target_sec <= 0 {
        return 4;
    }
    let pct = secs as f64 / daily_target_sec as f64;
    if pct >= 1.0 {
        4
    } else if pct >= 0.5 {
        3
    } else if pct >= 0.25 {
        2
    } else {
        1
    }
}

/// Streak hari berturut. Hari ini yang masih kosong belum memutus streak,
/// jadi perhitungan dimulai dari kemarin bila hari ini kosong.
pub fn current_streak(active_days: &BTreeSet<NaiveDate>, today: NaiveDate) -> i64 {
    let mut cursor = if active_days.contains(&today) {
        today
    } else {
        today - Duration::days(1)
    };
    let mut count = 0;
    while active_days.contains(&cursor) {
        count += 1;
        cursor -= Duration::days(1);
    }
    count
}

pub fn longest_streak(active_days: &BTreeSet<NaiveDate>) -> i64 {
    let mut best = 0;
    let mut run = 0;
    let mut previous: Option<NaiveDate> = None;
    for day in active_days {
        run = match previous {
            Some(prev) if *day == prev + Duration::days(1) => run + 1,
            Some(_) => 1,
            None => 1,
        };
        best = best.max(run);
        previous = Some(*day);
    }
    best
}

/// Susun kolom heatmap: `weeks` kolom, tiap kolom Senin sampai Minggu.
pub fn build_heatmap(
    totals: &BTreeMap<NaiveDate, i64>,
    today: NaiveDate,
    weeks: u32,
    daily_target_sec: i64,
) -> (Vec<Vec<HeatCell>>, i64) {
    let weeks = weeks.max(1);
    let this_monday = week_start(today);
    let first_monday = this_monday - Duration::days(i64::from(weeks - 1) * 7);
    let mut columns = Vec::with_capacity(weeks as usize);
    let mut max_sec = 0;

    for w in 0..weeks {
        let monday = first_monday + Duration::days(i64::from(w) * 7);
        let mut column = Vec::with_capacity(7);
        for d in 0..7 {
            let date = monday + Duration::days(d);
            let total = totals.get(&date).copied().unwrap_or(0);
            max_sec = max_sec.max(total);
            column.push(HeatCell {
                date: date.format("%Y-%m-%d").to_string(),
                total_sec: total,
                level: heat_level(total, daily_target_sec),
            });
        }
        columns.push(column);
    }
    (columns, max_sec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn session_before_day_start_belongs_to_previous_day() {
        // 01:30 UTC+7 tanggal 11 = 18:30 UTC tanggal 10
        let ts = 1_760_000_000; // 2025-10-09 14:13 UTC, dipakai sebagai basis
        let offset = 7 * 3600;
        let day = study_day_with_offset(ts, 4, offset);
        let clock = Utc.timestamp_opt(ts + offset, 0).single().unwrap();
        let expected = if clock.hour() < 4 {
            clock.date_naive() - Duration::days(1)
        } else {
            clock.date_naive()
        };
        assert_eq!(day, expected);
    }

    #[test]
    fn day_start_hour_shifts_the_boundary() {
        // 03:00 lokal = masih hari sebelumnya saat day_start = 4
        let offset = 0;
        let base = d(2025, 10, 11).and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let three_am = base + 3 * 3600;
        assert_eq!(study_day_with_offset(three_am, 4, offset), d(2025, 10, 10));
        assert_eq!(study_day_with_offset(three_am, 0, offset), d(2025, 10, 11));
        assert_eq!(study_day_with_offset(base + 4 * 3600, 4, offset), d(2025, 10, 11));
    }

    #[test]
    fn session_crossing_midnight_is_split_across_days() {
        let offset = 0;
        let start = d(2025, 10, 10).and_hms_opt(22, 0, 0).unwrap().and_utc().timestamp();
        let end = start + 4 * 3600; // 02:00 besoknya
        let mut totals = BTreeMap::new();
        split_session(start, end, 0, offset, 0, &mut totals);
        assert_eq!(totals.get(&d(2025, 10, 10)), Some(&7_200));
        assert_eq!(totals.get(&d(2025, 10, 11)), Some(&7_200));
        assert_eq!(totals.values().sum::<i64>(), 4 * 3600);
    }

    #[test]
    fn paused_time_is_removed_from_totals() {
        let offset = 0;
        let start = d(2025, 10, 10).and_hms_opt(20, 0, 0).unwrap().and_utc().timestamp();
        let mut totals = BTreeMap::new();
        split_session(start, start + 3_600, 1_800, offset, 0, &mut totals);
        assert_eq!(totals.values().sum::<i64>(), 1_800);
    }

    #[test]
    fn zero_length_session_adds_nothing() {
        let mut totals = BTreeMap::new();
        split_session(1_000, 1_000, 0, 0, 4, &mut totals);
        assert!(totals.is_empty());
    }

    #[test]
    fn heat_levels_follow_target_percentages() {
        assert_eq!(heat_level(0, 3_600), 0);
        assert_eq!(heat_level(1, 3_600), 1);
        assert_eq!(heat_level(900, 3_600), 2); // 25%
        assert_eq!(heat_level(1_800, 3_600), 3); // 50%
        assert_eq!(heat_level(3_600, 3_600), 4); // 100%
        assert_eq!(heat_level(9_999, 0), 4);
    }

    #[test]
    fn streak_ignores_today_until_it_is_filled() {
        let days: BTreeSet<_> = [d(2025, 10, 8), d(2025, 10, 9), d(2025, 10, 10)].into();
        assert_eq!(current_streak(&days, d(2025, 10, 11)), 3);
        assert_eq!(current_streak(&days, d(2025, 10, 10)), 3);
        assert_eq!(current_streak(&days, d(2025, 10, 12)), 0);
    }

    #[test]
    fn longest_streak_finds_the_best_run() {
        let days: BTreeSet<_> = [
            d(2025, 9, 1),
            d(2025, 9, 2),
            d(2025, 9, 5),
            d(2025, 9, 6),
            d(2025, 9, 7),
            d(2025, 9, 9),
        ]
        .into();
        assert_eq!(longest_streak(&days), 3);
        assert_eq!(longest_streak(&BTreeSet::new()), 0);
    }

    #[test]
    fn heatmap_has_seven_rows_per_week_column() {
        let mut totals = BTreeMap::new();
        totals.insert(d(2025, 10, 10), 4 * 3600);
        let (columns, max) = build_heatmap(&totals, d(2025, 10, 10), 26, 4 * 3600);
        assert_eq!(columns.len(), 26);
        assert!(columns.iter().all(|c| c.len() == 7));
        assert_eq!(max, 4 * 3600);
        let last = columns.last().unwrap();
        assert_eq!(last[4].level, 4); // Jumat
    }

    #[test]
    fn week_start_is_monday() {
        assert_eq!(week_start(d(2025, 10, 12)), d(2025, 10, 6));
        assert_eq!(week_start(d(2025, 10, 6)), d(2025, 10, 6));
    }
}
