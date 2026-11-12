//! Test integrasi: alur nyata lewat Store + SQLite in-memory.
//! Logika timer, batas hari, dan CRUD diuji di sini, bukan lewat UI.

use chrono::{Duration, Local, TimeZone};
use studytracker_core::db::Settings;
use studytracker_core::export::{self, ExportFormat};
use studytracker_core::models::*;
use studytracker_core::stats;
use studytracker_core::{CoreError, Store};

fn store() -> Store {
    Store::open_in_memory().expect("database in-memory")
}

fn seed_subject(store: &mut Store, name: &str, color: &str) -> i64 {
    store.upsert_subject(None, name, color).expect("mata pelajaran").id
}

/// Timestamp untuk jam lokal tertentu pada hari ini.
fn at(hour: u32, minute: u32) -> i64 {
    let date = Local::now().date_naive();
    Local
        .from_local_datetime(&date.and_hms_opt(hour, minute, 0).unwrap())
        .single()
        .expect("waktu lokal valid")
        .timestamp()
}

#[test]
fn start_pause_resume_stop_counts_only_running_time() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let t0 = 1_700_000_000;

    let snapshot = store.start_session(subject, t0).unwrap();
    assert_eq!(snapshot.phase, TimerPhase::Running);
    assert_eq!(snapshot.elapsed_sec, 0);

    store.pause_session(t0 + 600).unwrap(); // jeda di menit ke-10
    store.resume_session(t0 + 900).unwrap(); // lanjut 5 menit kemudian
    let session = store.stop_session(t0 + 1_200).unwrap(); // berhenti 5 menit kemudian

    // 1200 detik berjalan, 300 detik dijeda.
    assert_eq!(session.duration_sec, Some(900));
    assert_eq!(session.ended_at, Some(t0 + 1_200));
    assert!(store.active_session().unwrap().is_none());
}

#[test]
fn only_one_session_can_run_at_a_time() {
    let mut store = store();
    let math = seed_subject(&mut store, "Matematika", "rose");
    let physics = seed_subject(&mut store, "Fisika", "sky");

    store.start_session(math, 1_000).unwrap();
    let second = store.start_session(physics, 1_100);
    assert!(matches!(second, Err(CoreError::SessionAlreadyRunning)));
}

#[test]
fn stop_while_paused_ends_at_the_pause_moment() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Bahasa", "mint");
    let t0 = 1_700_000_000;

    store.start_session(subject, t0).unwrap();
    store.pause_session(t0 + 300).unwrap();
    let session = store.stop_session(t0 + 5_000).unwrap();

    assert_eq!(session.ended_at, Some(t0 + 300));
    assert_eq!(session.duration_sec, Some(300));
}

#[test]
fn orphan_session_can_be_finished_or_discarded() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Kimia", "lavender");
    let t0 = 1_700_000_000;

    store.start_session(subject, t0).unwrap();
    let session = store.finish_orphan_session(t0 + 1_800).unwrap();
    assert_eq!(session.duration_sec, Some(1_800));

    store.start_session(subject, t0 + 2_000).unwrap();
    let backwards = store.finish_orphan_session(t0 + 1_000);
    assert!(matches!(backwards, Err(CoreError::InvalidTimeRange)));

    let active = store.active_session().unwrap().unwrap();
    store.discard_session(active.id).unwrap();
    assert!(store.active_session().unwrap().is_none());
}

#[test]
fn subject_names_are_unique_and_archived_subjects_leave_the_picker() {
    let mut store = store();
    let id = seed_subject(&mut store, "Matematika", "rose");
    assert!(matches!(
        store.upsert_subject(None, "matematika", "sky"),
        Err(CoreError::DuplicateSubjectName(_))
    ));

    store.archive_subject(id, true).unwrap();
    assert!(store.list_subjects(false).unwrap().is_empty());
    assert_eq!(store.list_subjects(true).unwrap().len(), 1);
}

#[test]
fn subject_with_history_cannot_be_deleted() {
    let mut store = store();
    let id = seed_subject(&mut store, "Biologi", "lime");
    store.start_session(id, 1_000).unwrap();
    store.stop_session(1_600).unwrap();

    assert!(matches!(store.delete_subject(id), Err(CoreError::SubjectHasSessions)));

    let empty = seed_subject(&mut store, "Geografi", "aqua");
    assert!(store.delete_subject(empty).is_ok());
}

#[test]
fn day_start_hour_moves_early_morning_sessions_to_yesterday() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let hour = store.settings().unwrap().day_start_hour;
    assert_eq!(hour, 4);

    let today = store.today();
    let yesterday = today - Duration::days(1);
    let (yesterday_start, _) = stats::day_bounds(yesterday, hour);

    // 02.00 pagi ini menurut jam dinding = masih hari belajar kemarin.
    let early = yesterday_start + 86_400 + 2 * 3_600;
    assert_eq!(stats::study_day(early, hour), yesterday);

    store
        .add_manual_session(&ManualSessionInput {
            subject_id: subject,
            started_at: early,
            ended_at: early + 1_800,
            note: None,
        })
        .unwrap();

    assert_eq!(store.day_summary(yesterday).unwrap().total_sec, 1_800);
    assert_eq!(store.day_summary(today).unwrap().total_sec, 0);
}

#[test]
fn manual_sessions_report_overlaps_but_are_still_allowed() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Fisika", "sky");
    let start = at(9, 0);

    store
        .add_manual_session(&ManualSessionInput {
            subject_id: subject,
            started_at: start,
            ended_at: start + 3_600,
            note: Some("Bab 4".into()),
        })
        .unwrap();

    assert!(store.has_overlap(start + 1_800, start + 5_400, None).unwrap());

    let second = store
        .add_manual_session(&ManualSessionInput {
            subject_id: subject,
            started_at: start + 1_800,
            ended_at: start + 5_400,
            note: None,
        })
        .unwrap();
    assert!(store.has_overlap(start + 1_800, start + 1_900, Some(second.id)).unwrap());
}

#[test]
fn manual_session_rejects_backwards_clock() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Fisika", "sky");
    let result = store.add_manual_session(&ManualSessionInput {
        subject_id: subject,
        started_at: 2_000,
        ended_at: 1_000,
        note: None,
    });
    assert!(matches!(result, Err(CoreError::InvalidTimeRange)));
}

#[test]
fn goals_default_to_four_hours_and_reject_zero() {
    let mut store = store();
    let goals = store.goals().unwrap();
    assert_eq!(goals.daily_sec, 4 * 3600);
    assert_eq!(goals.weekly_sec, 20 * 3600);

    assert!(matches!(
        store.set_goal(GoalScope::Daily, 0),
        Err(CoreError::InvalidGoal)
    ));
    let updated = store.set_goal(GoalScope::Daily, 5 * 3600).unwrap();
    assert_eq!(updated.daily_sec, 5 * 3600);
}

#[test]
fn exams_are_sorted_and_counted_down() {
    let mut store = store();
    let today = store.today();
    let soon = (today + Duration::days(3)).format("%Y-%m-%d").to_string();
    let later = (today + Duration::days(20)).format("%Y-%m-%d").to_string();

    store.upsert_exam(None, "UTS Fisika", &later).unwrap();
    store.upsert_exam(None, "UTS Matematika", &soon).unwrap();

    let exams = store.list_exams().unwrap();
    assert_eq!(exams[0].exam.title, "UTS Matematika");
    assert_eq!(exams[0].days_left, 3);
    assert_eq!(exams[1].days_left, 20);

    assert!(store.upsert_exam(None, "Salah", "12-11-2026").is_err());
}

#[test]
fn settings_are_validated_before_saved() {
    let mut store = store();
    assert_eq!(store.settings().unwrap(), Settings::default());

    let saved = store.set_setting("ui_scale", "110").unwrap();
    assert_eq!(saved.ui_scale, 110);

    assert!(store.set_setting("ui_scale", "300").is_err());
    assert!(store.set_setting("tema_kucing", "dark").is_err());
    // Nilai lama tetap utuh setelah penolakan.
    assert_eq!(store.settings().unwrap().ui_scale, 110);
}

#[test]
fn stats_range_sums_against_the_previous_period() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let today = store.today();

    let (from, _) = stats::day_bounds(today - Duration::days(1), 4);
    for day in 0..2 {
        let start = from + day * 86_400 + 10 * 3_600;
        store
            .add_manual_session(&ManualSessionInput {
                subject_id: subject,
                started_at: start,
                ended_at: start + 3_600,
                note: None,
            })
            .unwrap();
    }

    let range = store
        .stats_range(today - Duration::days(1), today, Granularity::Day)
        .unwrap();
    assert_eq!(range.total_sec, 7_200);
    assert_eq!(range.buckets.len(), 2);
    assert_eq!(range.session_count, 2);

    let breakdown = store
        .subject_breakdown(today - Duration::days(1), today)
        .unwrap();
    assert_eq!(breakdown.len(), 1);
    assert_eq!(breakdown[0].secs, 7_200);
}

#[test]
fn heatmap_and_streaks_read_from_the_same_aggregation() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let today = store.today();

    let (from, _) = stats::day_bounds(today - Duration::days(2), 4);
    for day in 0..3 {
        let start = from + day * 86_400 + 9 * 3_600;
        store
            .add_manual_session(&ManualSessionInput {
                subject_id: subject,
                started_at: start,
                ended_at: start + 4 * 3_600,
                note: None,
            })
            .unwrap();
    }

    let heatmap = store.heatmap(26).unwrap();
    assert_eq!(heatmap.weeks.len(), 26);
    let levels: Vec<u8> = heatmap
        .weeks
        .iter()
        .flat_map(|column| column.iter())
        .filter(|cell| cell.total_sec > 0)
        .map(|cell| cell.level)
        .collect();
    assert_eq!(levels, vec![4, 4, 4]);

    let streaks = store.streaks().unwrap();
    assert_eq!(streaks.current_days, 3);
    assert_eq!(streaks.longest_days, 3);
    assert_eq!(streaks.best_day_sec, 4 * 3_600);
}

#[test]
fn export_and_import_round_trip_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("studi.json");

    let mut source = store();
    let subject = seed_subject(&mut source, "Matematika", "rose");
    source.start_session(subject, 1_700_000_000).unwrap();
    source.stop_session(1_700_003_600).unwrap();
    source.set_goal(GoalScope::Daily, 6 * 3_600).unwrap();
    source
        .upsert_exam(None, "UAS", "2026-12-01")
        .unwrap();

    let exported = export::export(source.conn(), &path, ExportFormat::Json).unwrap();
    assert_eq!(exported, 1);

    let mut target = store();
    let report = export::import(target.conn(), &path).unwrap();
    assert_eq!(report.subjects_added, 1);
    assert_eq!(report.sessions_added, 1);
    assert_eq!(report.exams_added, 1);
    assert_eq!(target.goals().unwrap().daily_sec, 6 * 3_600);

    // Impor kedua tidak menggandakan data.
    let again = export::import(target.conn(), &path).unwrap();
    assert_eq!(again.sessions_added, 0);
    assert_eq!(again.subjects_added, 0);
}

#[test]
fn export_and_import_round_trip_csv() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("studi.csv");

    let mut source = store();
    let subject = seed_subject(&mut source, "Fisika", "sky");
    source
        .add_manual_session(&ManualSessionInput {
            subject_id: subject,
            started_at: at(9, 0),
            ended_at: at(10, 30),
            note: Some("Ulangan, bab 3".into()),
        })
        .unwrap();

    export::export(source.conn(), &path, ExportFormat::Csv).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("id,mata_pelajaran"));
    assert!(text.contains("\"Ulangan, bab 3\""));

    let mut target = store();
    let report = export::import(target.conn(), &path).unwrap();
    assert_eq!(report.sessions_added, 1);

    let today = source.today();
    let imported = target.day_summary(today).unwrap();
    assert_eq!(imported.total_sec, 5_400);
    assert_eq!(imported.sessions[0].subject_name, "Fisika");
}

#[test]
fn import_rejects_files_that_are_not_studytracker_exports() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("asing.json");
    std::fs::write(&path, r#"{"format":"lain","version":1}"#).unwrap();

    let store = store();
    assert!(export::import(store.conn(), &path).is_err());
}

#[test]
fn deleting_a_session_updates_totals() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let start = at(13, 0);
    let session = store
        .add_manual_session(&ManualSessionInput {
            subject_id: subject,
            started_at: start,
            ended_at: start + 3_600,
            note: None,
        })
        .unwrap();

    assert_eq!(store.today_total().unwrap(), 3_600);
    store.delete_session(session.id).unwrap();
    assert_eq!(store.today_total().unwrap(), 0);
}

#[test]
fn undo_stop_removes_the_saved_session() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    store.start_session(subject, 1_000).unwrap();
    let saved = store.stop_session(4_600).unwrap();

    store.undo_stop(saved.id).unwrap();
    assert!(matches!(store.undo_stop(saved.id), Err(CoreError::SessionNotFound)));
    assert_eq!(store.today_total().unwrap(), 0);
}

#[test]
fn session_view_reports_live_duration_while_running() {
    let mut store = store();
    let subject = seed_subject(&mut store, "Matematika", "rose");
    let start = stats::now_secs() - 90;
    store.start_session(subject, start).unwrap();

    let active = store.active_session().unwrap().unwrap();
    assert!(active.ended_at.is_none());

    let day = store.today();
    let (from, to) = stats::day_bounds(day, 4);
    let views = store.list_sessions(from, to).unwrap();
    assert_eq!(views.len(), 1);
    assert!(views[0].effective_duration_sec >= 90);
}
