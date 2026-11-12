//! Jembatan tipis antara frontend dan `studytracker-core`.
//!
//! Tidak ada logika bisnis di sini: setiap command memanggil core, lalu
//! menyiarkan `session-changed` supaya jendela utama dan mini mode tetap sinkron.

use chrono::NaiveDate;
use studytracker_core::db::Settings;
use studytracker_core::export::{self, ExportFormat};
use studytracker_core::models::*;
use studytracker_core::stats;
use studytracker_core::{CoreError, Result as CoreResult, Store};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::idle::{IdleMonitor, SystemIdle};
use crate::state::AppState;
use crate::tray;

const MINI_LABEL: &str = "mini";

fn parse_date(raw: &str) -> CoreResult<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| {
        CoreError::msg(format!(
            "Tanggal \"{raw}\" tidak dikenali. Pakai format tahun-bulan-hari, misalnya 2026-11-12."
        ))
    })
}

/// Siarkan perubahan sesi ke semua jendela dan segarkan ikon tray.
pub fn broadcast(app: &AppHandle) {
    let _ = app.emit("session-changed", ());
    tray::refresh(app);
}

// ---------------------------------------------------------------------- timer

#[tauri::command]
pub fn get_timer_state(state: State<'_, AppState>) -> CoreResult<TimerSnapshot> {
    state.lock()?.timer_snapshot()
}

#[tauri::command]
pub fn start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    subject_id: i64,
) -> CoreResult<TimerSnapshot> {
    let snapshot = state.lock()?.start_session(subject_id, stats::now_secs())?;
    broadcast(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn pause_session(app: AppHandle, state: State<'_, AppState>) -> CoreResult<TimerSnapshot> {
    let snapshot = state.lock()?.pause_session(stats::now_secs())?;
    broadcast(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn resume_session(app: AppHandle, state: State<'_, AppState>) -> CoreResult<TimerSnapshot> {
    let snapshot = state.lock()?.resume_session(stats::now_secs())?;
    broadcast(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn stop_session(app: AppHandle, state: State<'_, AppState>) -> CoreResult<SessionView> {
    let session = state.lock()?.stop_session(stats::now_secs())?;
    broadcast(&app);
    let view = state.lock()?.session_view_full(session.id)?;
    Ok(view)
}

#[tauri::command]
pub fn undo_stop(app: AppHandle, state: State<'_, AppState>, session_id: i64) -> CoreResult<()> {
    state.lock()?.undo_stop(session_id)?;
    broadcast(&app);
    Ok(())
}

#[tauri::command]
pub fn discard_session(app: AppHandle, state: State<'_, AppState>, session_id: i64) -> CoreResult<()> {
    state.lock()?.discard_session(session_id)?;
    broadcast(&app);
    Ok(())
}

#[tauri::command]
pub fn trim_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: i64,
    secs: i64,
) -> CoreResult<TimerSnapshot> {
    let snapshot = state.lock()?.trim_session(session_id, secs)?;
    broadcast(&app);
    Ok(snapshot)
}

#[tauri::command]
pub fn finish_orphan_session(
    app: AppHandle,
    state: State<'_, AppState>,
    ended_at: i64,
) -> CoreResult<Session> {
    let session = state.lock()?.finish_orphan_session(ended_at)?;
    broadcast(&app);
    Ok(session)
}

// ----------------------------------------------------------------------- sesi

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayTotalDto {
    pub date: String,
    pub total_sec: i64,
}

#[tauri::command]
pub fn list_sessions(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> CoreResult<Vec<SessionView>> {
    let hour = state.lock()?.settings()?.day_start_hour;
    let (from_ts, _) = stats::day_bounds(parse_date(&from)?, hour);
    let (_, to_ts) = stats::day_bounds(parse_date(&to)?, hour);
    state.lock()?.list_sessions(from_ts, to_ts)
}

#[tauri::command]
pub fn day_summary(state: State<'_, AppState>, date: String) -> CoreResult<DaySummary> {
    state.lock()?.day_summary(parse_date(&date)?)
}

#[tauri::command]
pub fn add_manual_session(
    app: AppHandle,
    state: State<'_, AppState>,
    input: ManualSessionInput,
) -> CoreResult<Session> {
    let session = state.lock()?.add_manual_session(&input)?;
    broadcast(&app);
    Ok(session)
}

#[tauri::command]
pub fn update_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: i64,
    input: ManualSessionInput,
) -> CoreResult<Session> {
    let session = state.lock()?.update_session(session_id, &input)?;
    broadcast(&app);
    Ok(session)
}

#[tauri::command]
pub fn delete_session(app: AppHandle, state: State<'_, AppState>, session_id: i64) -> CoreResult<()> {
    state.lock()?.delete_session(session_id)?;
    broadcast(&app);
    Ok(())
}

#[tauri::command]
pub fn has_overlap(
    state: State<'_, AppState>,
    started_at: i64,
    ended_at: i64,
    exclude_id: Option<i64>,
) -> CoreResult<bool> {
    state.lock()?.has_overlap(started_at, ended_at, exclude_id)
}

#[tauri::command]
pub fn day_totals(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> CoreResult<Vec<DayTotalDto>> {
    let rows = state
        .lock()?
        .day_totals(parse_date(&from)?, parse_date(&to)?)?;
    Ok(rows
        .into_iter()
        .map(|(date, total_sec)| DayTotalDto {
            date: date.format("%Y-%m-%d").to_string(),
            total_sec,
        })
        .collect())
}

// ------------------------------------------------------------- mata pelajaran

#[tauri::command]
pub fn list_subjects(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> CoreResult<Vec<Subject>> {
    state.lock()?.list_subjects(include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn upsert_subject(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Option<i64>,
    name: String,
    color: String,
) -> CoreResult<Subject> {
    let subject = state.lock()?.upsert_subject(id, &name, &color)?;
    broadcast(&app);
    Ok(subject)
}

#[tauri::command]
pub fn archive_subject(
    app: AppHandle,
    state: State<'_, AppState>,
    subject_id: i64,
    archived: bool,
) -> CoreResult<Subject> {
    let subject = state.lock()?.archive_subject(subject_id, archived)?;
    broadcast(&app);
    Ok(subject)
}

#[tauri::command]
pub fn delete_subject(app: AppHandle, state: State<'_, AppState>, subject_id: i64) -> CoreResult<()> {
    state.lock()?.delete_subject(subject_id)?;
    broadcast(&app);
    Ok(())
}

#[tauri::command]
pub fn subject_totals(state: State<'_, AppState>) -> CoreResult<Vec<SubjectTotal>> {
    state.lock()?.subject_totals()
}

// ----------------------------------------------------------------- statistik

#[tauri::command]
pub fn stats_range(
    state: State<'_, AppState>,
    from: String,
    to: String,
    granularity: String,
) -> CoreResult<RangeStats> {
    state.lock()?.stats_range(
        parse_date(&from)?,
        parse_date(&to)?,
        Granularity::parse(&granularity),
    )
}

#[tauri::command]
pub fn subject_breakdown(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> CoreResult<Vec<SubjectTotal>> {
    state
        .lock()?
        .subject_breakdown(parse_date(&from)?, parse_date(&to)?)
}

#[tauri::command]
pub fn heatmap(state: State<'_, AppState>, weeks: Option<u32>) -> CoreResult<Heatmap> {
    state.lock()?.heatmap(weeks.unwrap_or(26))
}

#[tauri::command]
pub fn streaks(state: State<'_, AppState>) -> CoreResult<Streaks> {
    state.lock()?.streaks()
}

#[tauri::command]
pub fn today_overview(state: State<'_, AppState>) -> CoreResult<DaySummary> {
    let store = state.lock()?;
    store.day_summary(store.today())
}

// ---------------------------------------------------------- target dan D-day

#[tauri::command]
pub fn get_goals(state: State<'_, AppState>) -> CoreResult<Goals> {
    state.lock()?.goals()
}

#[tauri::command]
pub fn set_goal(
    app: AppHandle,
    state: State<'_, AppState>,
    scope: String,
    target_sec: i64,
) -> CoreResult<Goals> {
    let scope = GoalScope::parse(&scope)
        .ok_or_else(|| CoreError::msg("Jenis target tidak dikenal. Pilih harian atau mingguan."))?;
    let goals = state.lock()?.set_goal(scope, target_sec)?;
    broadcast(&app);
    Ok(goals)
}

#[tauri::command]
pub fn list_exams(state: State<'_, AppState>) -> CoreResult<Vec<ExamView>> {
    state.lock()?.list_exams()
}

#[tauri::command]
pub fn upsert_exam(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Option<i64>,
    title: String,
    exam_date: String,
) -> CoreResult<Exam> {
    let exam = state.lock()?.upsert_exam(id, &title, &exam_date)?;
    broadcast(&app);
    Ok(exam)
}

#[tauri::command]
pub fn delete_exam(app: AppHandle, state: State<'_, AppState>, exam_id: i64) -> CoreResult<()> {
    state.lock()?.delete_exam(exam_id)?;
    broadcast(&app);
    Ok(())
}

// ---------------------------------------------------------------------- data

#[tauri::command]
pub fn export_data(
    app: AppHandle,
    state: State<'_, AppState>,
    format: String,
    path: Option<String>,
) -> CoreResult<String> {
    let format = ExportFormat::parse(&format);
    let target = match path {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            let stamp = chrono::Local::now().format("%Y%m%d-%H%M");
            let ext = if format == ExportFormat::Csv { "csv" } else { "json" };
            state
                .data_dir
                .join("ekspor")
                .join(format!("studytracker-{stamp}.{ext}"))
        }
    };
    let store = state.lock()?;
    let count = export::export(store.conn(), &target, format)?;
    tracing::info!(target: "studytracker", sessions = count, path = %target.display(), "ekspor selesai");
    let _ = app.emit("data-exported", target.display().to_string());
    Ok(target.display().to_string())
}

#[tauri::command]
pub fn import_data(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> CoreResult<export::ImportReport> {
    let file = std::path::PathBuf::from(path);
    let store = state.lock()?;
    let report = export::import(store.conn(), &file)?;
    drop(store);
    broadcast(&app);
    Ok(report)
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, AppState>) -> CoreResult<String> {
    let dir = state.data_dir.clone();
    open_in_file_manager(&dir)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub fn backup_now(state: State<'_, AppState>) -> CoreResult<String> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let dir = state.data_dir.join("cadangan");
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(format!("studytracker-{stamp}.db"));
    let store = state.lock()?;
    let source = studytracker_core::db::db_path(&state.data_dir);
    drop(store);
    std::fs::copy(&source, &target)?;
    Ok(target.display().to_string())
}

fn open_in_file_manager(path: &std::path::Path) -> CoreResult<()> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg(path);
        cmd
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut cmd = std::process::Command::new("open");
        cmd.arg(path);
        cmd
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut cmd = std::process::Command::new("xdg-open");
        cmd.arg(path);
        cmd
    };
    command.spawn()?;
    Ok(())
}

// --------------------------------------------------------------- pengaturan

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CoreResult<Settings> {
    state.lock()?.settings()
}

#[tauri::command]
pub fn set_setting(
    app: AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> CoreResult<Settings> {
    let settings = state.lock()?.set_setting(&key, &value)?;
    apply_setting_side_effects(&app, &key, &settings);
    Ok(settings)
}

/// Pengaturan yang punya efek di luar database: autostart dan pintasan global.
fn apply_setting_side_effects(app: &AppHandle, key: &str, settings: &Settings) {
    match key {
        "autostart" => {
            use tauri_plugin_autostart::ManagerExt;
            let manager = app.autolaunch();
            let result = if settings.autostart {
                manager.enable()
            } else {
                manager.disable()
            };
            if let Err(error) = result {
                tracing::warn!(target: "studytracker", %error, "gagal mengubah autostart");
            }
        }
        "global_shortcut" => {
            if let Err(error) = register_shortcut(app, &settings.global_shortcut) {
                tracing::warn!(target: "studytracker", %error, "gagal mendaftarkan pintasan global");
            }
        }
        "close_to_tray" | "remember_window" => {
            // Dipegang jendela; tidak ada aksi di sisi backend.
        }
        _ => {}
    }
}

pub fn register_shortcut(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    let parsed: Shortcut = shortcut
        .parse()
        .map_err(|_| format!("Pintasan \"{shortcut}\" tidak valid. Contoh yang benar: Ctrl+Alt+S."))?;
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();
    manager
        .register(parsed)
        .map_err(|error| format!("Pintasan tidak bisa dipakai: {error}"))
}

// ------------------------------------------------------------------- jendela

/// Buka (atau fokuskan) jendela mini mode. Dipakai layar Timer dan menu tray.
pub fn open_mini_window(app: &AppHandle) -> CoreResult<()> {
    if let Some(window) = app.get_webview_window(MINI_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    let always_on_top = app
        .state::<AppState>()
        .lock()?
        .settings()?
        .mini_always_on_top;
    WebviewWindowBuilder::new(app, MINI_LABEL, WebviewUrl::App("index.html".into()))
        .title("StudyTracker")
        .inner_size(300.0, 160.0)
        .min_inner_size(280.0, 140.0)
        .resizable(true)
        .always_on_top(always_on_top)
        .skip_taskbar(true)
        .build()
        .map_err(|error| CoreError::msg(format!("Mini mode tidak bisa dibuka: {error}")))?;
    Ok(())
}

/// Tampilkan jendela utama, tutup mini mode bila sedang terbuka.
pub fn show_main(app: &AppHandle) {
    if let Some(mini) = app.get_webview_window(MINI_LABEL) {
        let _ = mini.close();
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Berhenti dari menu tray. Sesi langsung tersimpan seperti tombol di layar.
pub fn stop_from_tray(app: &AppHandle) -> CoreResult<()> {
    let state = app.state::<AppState>();
    let mut store = state.lock()?;
    if store.timer_snapshot()?.phase == TimerPhase::Idle {
        return Ok(());
    }
    store.stop_session(stats::now_secs())?;
    drop(store);
    broadcast(app);
    Ok(())
}

// Command jendela dibuat `async` supaya tidak berjalan di main thread: membuat
// WebView2 baru dari main thread saat IPC sedang menunggu jawaban akan deadlock.
#[tauri::command]
pub async fn open_mini(app: AppHandle) -> CoreResult<()> {
    open_mini_window(&app)
}

#[tauri::command]
pub async fn close_mini(app: AppHandle) -> CoreResult<()> {
    if let Some(window) = app.get_webview_window(MINI_LABEL) {
        window
            .close()
            .map_err(|error| CoreError::msg(format!("Mini mode tidak bisa ditutup: {error}")))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_mini_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    flag: bool,
) -> CoreResult<()> {
    state
        .lock()?
        .set_setting("mini_always_on_top", if flag { "true" } else { "false" })?;
    if let Some(window) = app.get_webview_window(MINI_LABEL) {
        let _ = window.set_always_on_top(flag);
    }
    Ok(())
}

#[tauri::command]
pub async fn focus_main(app: AppHandle) -> CoreResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    if let Some(mini) = app.get_webview_window(MINI_LABEL) {
        let _ = mini.close();
    }
    Ok(())
}

// -------------------------------------------------------- sistem dan informasi

#[tauri::command]
pub fn idle_seconds() -> u64 {
    SystemIdle.idle_secs()
}

#[tauri::command]
pub fn notify(app: AppHandle, title: String, body: String) -> CoreResult<()> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| CoreError::msg(format!("Notifikasi tidak bisa ditampilkan: {error}")))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub database_path: String,
    pub platform: String,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: studytracker_core::VERSION.to_string(),
        data_dir: state.data_dir.display().to_string(),
        database_path: studytracker_core::db::db_path(&state.data_dir)
            .display()
            .to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}

/// Dipakai tray dan pintasan global: mulai dari mata pelajaran terakhir,
/// atau jeda/lanjut bila timer sudah jalan.
pub fn toggle_timer(app: &AppHandle) -> CoreResult<()> {
    let state = app.state::<AppState>();
    let snapshot = {
        let mut store = state.lock()?;
        match store.timer_snapshot()?.phase {
            TimerPhase::Idle => {
                let subject = match store.last_used_subject()? {
                    Some(subject) => Some(subject),
                    None => store.list_subjects(false)?.into_iter().next(),
                };
                match subject {
                    Some(subject) => Some(store.start_session(subject.id, stats::now_secs())?),
                    None => None,
                }
            }
            TimerPhase::Running => Some(store.pause_session(stats::now_secs())?),
            TimerPhase::Paused => Some(store.resume_session(stats::now_secs())?),
        }
    };
    if snapshot.is_none() {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    broadcast(app);
    Ok(())
}

/// Store dibungkus supaya command tidak perlu mengimpor `Store` di banyak tempat.
trait StoreExt {
    fn session_view_full(&self, id: i64) -> CoreResult<SessionView>;
}

impl StoreExt for Store {
    fn session_view_full(&self, id: i64) -> CoreResult<SessionView> {
        let now = stats::now_secs();
        let session = self.session_view(id)?;
        let subject = self.subject(session.subject_id)?;
        Ok(SessionView {
            effective_duration_sec: session.duration_sec.unwrap_or_else(|| {
                studytracker_core::timer::elapsed_sec(
                    session.started_at,
                    session.paused_at,
                    session.paused_total_sec,
                    now,
                )
            }),
            session,
            subject_name: subject.name,
            subject_color: subject.color,
        })
    }
}
