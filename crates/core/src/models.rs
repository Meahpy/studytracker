//! Tipe data yang dipakai bersama oleh core, Tauri, dan frontend.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Warna mata pelajaran. Delapan pilihan pastel, teks di atasnya selalu gelap.
pub const SUBJECT_COLORS: [&str; 8] = [
    "rose", "butter", "tangerine", "lime", "mint", "aqua", "sky", "lavender",
];

pub fn is_valid_color(color: &str) -> bool {
    SUBJECT_COLORS.contains(&color)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: i64,
    pub name: String,
    /// Nama token warna: `rose|butter|tangerine|lime|mint|aqua|sky|lavender`.
    pub color: String,
    pub archived: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionSource {
    Timer,
    Manual,
}

impl SessionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionSource::Timer => "timer",
            SessionSource::Manual => "manual",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "manual" => SessionSource::Manual,
            _ => SessionSource::Timer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub subject_id: i64,
    pub started_at: i64,
    /// `None` berarti sesi masih berjalan atau sedang dijeda.
    pub ended_at: Option<i64>,
    pub paused_at: Option<i64>,
    pub paused_total_sec: i64,
    pub duration_sec: Option<i64>,
    pub note: Option<String>,
    pub source: SessionSource,
}

/// Sesi + nama/warna mata pelajaran, supaya UI tidak perlu join sendiri.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    #[serde(flatten)]
    pub session: Session,
    pub subject_name: String,
    pub subject_color: String,
    /// Durasi yang dipakai UI: tersimpan bila selesai, berjalan bila belum.
    pub effective_duration_sec: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerPhase {
    Idle,
    Running,
    Paused,
}

/// Keadaan timer yang dikirim ke frontend. Frontend hanya menampilkan angka,
/// tidak menghitung durasi maupun statistik.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub phase: TimerPhase,
    pub session_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub subject_color: Option<String>,
    pub started_at: Option<i64>,
    pub paused_at: Option<i64>,
    pub paused_total_sec: i64,
    pub elapsed_sec: i64,
}

impl TimerSnapshot {
    pub fn idle() -> Self {
        Self {
            phase: TimerPhase::Idle,
            session_id: None,
            subject_id: None,
            subject_name: None,
            subject_color: None,
            started_at: None,
            paused_at: None,
            paused_total_sec: 0,
            elapsed_sec: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalScope {
    Daily,
    Weekly,
}

impl GoalScope {
    pub fn as_str(self) -> &'static str {
        match self {
            GoalScope::Daily => "daily",
            GoalScope::Weekly => "weekly",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "daily" => Some(GoalScope::Daily),
            "weekly" => Some(GoalScope::Weekly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goals {
    pub daily_sec: i64,
    pub weekly_sec: i64,
}

impl Default for Goals {
    fn default() -> Self {
        // 4 jam sehari, 20 jam seminggu: cukup ambisius, tidak menghukum.
        Self { daily_sec: 4 * 3600, weekly_sec: 20 * 3600 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    pub id: i64,
    pub title: String,
    /// Tanggal lokal `YYYY-MM-DD`.
    pub exam_date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamView {
    #[serde(flatten)]
    pub exam: Exam,
    /// Selisih hari terhadap hari ini. Negatif berarti sudah terlewat.
    pub days_left: i64,
}

impl Exam {
    pub fn parsed_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.exam_date, "%Y-%m-%d").ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Granularity {
    Day,
    Week,
    Month,
}

impl Granularity {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "week" => Granularity::Week,
            "month" => Granularity::Month,
            _ => Granularity::Day,
        }
    }
}

/// Satu batang/kelompok pada grafik statistik.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    /// Label pendek yang sudah siap tampil, mis. `Sen` atau `10 Nov`.
    pub label: String,
    /// Tanggal lokal pertama dalam bucket, `YYYY-MM-DD`.
    pub date: String,
    pub total_sec: i64,
    /// Pecahan per mata pelajaran untuk grafik bertumpuk.
    pub parts: Vec<BucketPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketPart {
    pub subject_id: i64,
    pub subject_name: String,
    pub color: String,
    pub secs: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeStats {
    pub from: String,
    pub to: String,
    pub total_sec: i64,
    pub previous_total_sec: i64,
    pub session_count: i64,
    pub best_day_sec: i64,
    pub best_day_date: Option<String>,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectTotal {
    pub subject_id: i64,
    pub subject_name: String,
    pub color: String,
    pub secs: i64,
    pub session_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatCell {
    pub date: String,
    pub total_sec: i64,
    /// 0 sampai 4, mengikuti skala `--heat-0` sampai `--heat-4`.
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Heatmap {
    /// Kolom demi kolom, tiap kolom tujuh hari mulai Senin.
    pub weeks: Vec<Vec<HeatCell>>,
    pub max_sec: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Streaks {
    pub current_days: i64,
    pub longest_days: i64,
    pub best_day_sec: i64,
    pub best_day_date: Option<String>,
    pub best_week_sec: i64,
    pub best_week_start: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySummary {
    pub date: String,
    pub total_sec: i64,
    pub sessions: Vec<SessionView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualSessionInput {
    pub subject_id: i64,
    pub started_at: i64,
    pub ended_at: i64,
    pub note: Option<String>,
}
