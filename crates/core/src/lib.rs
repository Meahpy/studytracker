//! `studytracker-core` — seluruh logika bisnis StudyTracker.
//!
//! Crate ini tidak tahu apa-apa soal Tauri, jendela, atau HTML. Semua fungsi di
//! sini bisa diuji dengan `cargo test` dan dengan SQLite in-memory.

pub mod db;
pub mod error;
pub mod export;
pub mod models;
pub mod stats;
pub mod timer;

pub use db::{Settings, Store, DB_FILE_NAME, SCHEMA_VERSION};
pub use error::{CoreError, Result};
pub use models::*;
pub use stats::now_secs;

/// Versi aplikasi yang tampil di Pengaturan.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
