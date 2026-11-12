//! Ikon tray: menampilkan keadaan timer dan menu cepat.

use studytracker_core::models::TimerPhase;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::commands;
use crate::state::AppState;

pub const TRAY_ID: &str = "studytracker-tray";

const ICON_IDLE: &[u8] = include_bytes!("../icons/tray-idle.png");
const ICON_RUNNING: &[u8] = include_bytes!("../icons/tray-running.png");
const ICON_PAUSED: &[u8] = include_bytes!("../icons/tray-paused.png");

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Mulai atau jeda", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Berhenti dan simpan", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Buka StudyTracker", true, None::<&str>)?;
    let mini = MenuItem::with_id(app, "mini", "Mini mode", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
    let separator_a = PredefinedMenuItem::separator(app)?;
    let separator_b = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[&toggle, &stop, &separator_a, &open, &mini, &separator_b, &quit],
    )?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(ICON_IDLE)?)
        .tooltip("StudyTracker")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => {
                if let Err(error) = commands::toggle_timer(app) {
                    tracing::warn!(target: "studytracker::tray", %error, "gagal mengubah timer");
                }
            }
            "stop" => {
                if let Err(error) = commands::stop_from_tray(app) {
                    tracing::warn!(target: "studytracker::tray", %error, "gagal menghentikan sesi");
                }
            }
            "open" => commands::show_main(app),
            "mini" => {
                if let Err(error) = commands::open_mini_window(app) {
                    tracing::warn!(target: "studytracker::tray", %error, "gagal membuka mini mode");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                commands::show_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Segarkan ikon dan tooltip tray menurut keadaan timer.
pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let snapshot = app
        .state::<AppState>()
        .lock()
        .ok()
        .and_then(|store| store.timer_snapshot().ok());

    let (bytes, tooltip) = match snapshot {
        Some(snapshot) => match snapshot.phase {
            TimerPhase::Running => (
                ICON_RUNNING,
                format!(
                    "Berjalan: {}",
                    snapshot.subject_name.unwrap_or_else(|| "belajar".into())
                ),
            ),
            TimerPhase::Paused => (
                ICON_PAUSED,
                format!(
                    "Jeda: {}",
                    snapshot.subject_name.unwrap_or_else(|| "belajar".into())
                ),
            ),
            TimerPhase::Idle => (ICON_IDLE, "StudyTracker".to_string()),
        },
        None => (ICON_IDLE, "StudyTracker".to_string()),
    };

    if let Ok(image) = Image::from_bytes(bytes) {
        let _ = tray.set_icon(Some(image));
    }
    let _ = tray.set_tooltip(Some(&tooltip));
}
