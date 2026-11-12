//! StudyTracker desktop shell.
//!
//! Lapisan ini sengaja tipis: command, tray, pintasan global, dan jendela.
//! Semua logika bisnis ada di `studytracker-core`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod idle;
mod state;
mod tray;

use std::sync::Mutex;

use studytracker_core::{db, Settings};
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

use crate::state::AppState;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "studytracker=info,studytracker_core=info".into()
            }),
        )
        .init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            commands::show_main(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Err(error) = commands::toggle_timer(app) {
                            tracing::warn!(target: "studytracker", %error, "pintasan global gagal");
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = db::open(&data_dir)?;
            let settings = store.settings()?;

            app.manage(AppState {
                store: Mutex::new(store),
                data_dir,
            });

            tray::create(app.handle())?;
            tray::refresh(app.handle());

            if let Err(error) = commands::register_shortcut(app.handle(), &settings.global_shortcut) {
                tracing::warn!(target: "studytracker", %error, "pintasan global tidak aktif");
            }

            tracing::info!(
                target: "studytracker",
                version = studytracker_core::VERSION,
                "StudyTracker siap"
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_timer_state,
            commands::start_session,
            commands::pause_session,
            commands::resume_session,
            commands::stop_session,
            commands::undo_stop,
            commands::discard_session,
            commands::trim_session,
            commands::finish_orphan_session,
            commands::list_sessions,
            commands::day_summary,
            commands::add_manual_session,
            commands::update_session,
            commands::delete_session,
            commands::has_overlap,
            commands::day_totals,
            commands::list_subjects,
            commands::upsert_subject,
            commands::archive_subject,
            commands::delete_subject,
            commands::subject_totals,
            commands::stats_range,
            commands::subject_breakdown,
            commands::heatmap,
            commands::streaks,
            commands::today_overview,
            commands::get_goals,
            commands::set_goal,
            commands::list_exams,
            commands::upsert_exam,
            commands::delete_exam,
            commands::export_data,
            commands::import_data,
            commands::open_data_dir,
            commands::backup_now,
            commands::get_settings,
            commands::set_setting,
            commands::open_mini,
            commands::close_mini,
            commands::set_mini_always_on_top,
            commands::focus_main,
            commands::idle_seconds,
            commands::notify,
            commands::app_info,
        ])
        .build(tauri::generate_context!())
        .expect("StudyTracker gagal dimuat");

    app.run(|app_handle, event| {
        if let RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } = &event
        {
            if label == "main" && close_to_tray(app_handle) {
                api.prevent_close();
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        }
    });
}

fn close_to_tray(app: &tauri::AppHandle) -> bool {
    let settings: Option<Settings> = app
        .state::<AppState>()
        .lock()
        .ok()
        .and_then(|store| store.settings().ok());
    settings.map(|s| s.close_to_tray).unwrap_or(true)
}
