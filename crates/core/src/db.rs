//! Koneksi SQLite, migrasi, dan seluruh query. Satu-satunya tempat yang boleh
//! menulis SQL.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Row};
use rusqlite_migration::{Migrations, M};

use crate::error::{CoreError, Result};
use crate::models::*;
use crate::stats;
use crate::timer;

pub const DB_FILE_NAME: &str = "studytracker.db";

/// Versi skema terakhir. Dipakai untuk memutuskan kapan cadangan otomatis dibuat.
pub const SCHEMA_VERSION: i64 = 1;

const MIGRATION_V1: &str = r#"
CREATE TABLE subjects (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  color       TEXT NOT NULL,
  archived    INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);

CREATE TABLE sessions (
  id               INTEGER PRIMARY KEY,
  subject_id       INTEGER NOT NULL REFERENCES subjects(id),
  started_at       INTEGER NOT NULL,
  ended_at         INTEGER,
  paused_at        INTEGER,
  paused_total_sec INTEGER NOT NULL DEFAULT 0,
  duration_sec     INTEGER,
  note             TEXT,
  source           TEXT NOT NULL DEFAULT 'timer'
);

CREATE UNIQUE INDEX one_active_session ON sessions((ended_at IS NULL)) WHERE ended_at IS NULL;
CREATE INDEX idx_sessions_started ON sessions(started_at);

CREATE TABLE goals (
  scope      TEXT PRIMARY KEY CHECK (scope IN ('daily','weekly')),
  target_sec INTEGER NOT NULL
);

CREATE TABLE exams (
  id        INTEGER PRIMARY KEY,
  title     TEXT NOT NULL,
  exam_date TEXT NOT NULL
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

INSERT INTO goals (scope, target_sec) VALUES ('daily', 14400), ('weekly', 72000);

INSERT INTO settings (key, value) VALUES
  ('day_start_hour', '4'),
  ('idle_threshold_min', '0'),
  ('count_sleep_time', 'false'),
  ('close_to_tray', 'true'),
  ('autostart', 'false'),
  ('theme', 'system'),
  ('ui_scale', '100'),
  ('global_shortcut', 'Ctrl+Alt+S'),
  ('remember_window', 'true'),
  ('language', 'id'),
  ('mini_always_on_top', 'false');
"#;

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(MIGRATION_V1)])
}

/// Buka database, jalankan migrasi, dan cadangkan file lama bila skema berubah.
pub fn open(app_data_dir: &Path) -> Result<Store> {
    std::fs::create_dir_all(app_data_dir)?;
    let path = app_data_dir.join(DB_FILE_NAME);
    backup_if_outdated(&path)?;
    let mut conn = Connection::open(&path)?;
    configure(&conn)?;
    migrations().to_latest(&mut conn)?;
    Ok(Store { conn })
}

/// Cadangkan file database sebelum migrasi yang mengubah skema.
fn backup_if_outdated(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let conn = Connection::open(path)?;
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    drop(conn);
    if version < SCHEMA_VERSION {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup = path.with_file_name(format!("{DB_FILE_NAME}.{stamp}.bak"));
        std::fs::copy(path, &backup)?;
        tracing::info!(target: "studytracker::db", path = %backup.display(), "cadangan dibuat sebelum migrasi");
    }
    Ok(())
}

fn configure(conn: &Connection) -> Result<()> {
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )?;
    Ok(())
}

/// Lokasi file database di dalam folder data aplikasi.
pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(DB_FILE_NAME)
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        configure(&conn)?;
        migrations().to_latest(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    // ---------------------------------------------------------------- pengaturan

    pub fn settings(&self) -> Result<Settings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (key, value) = row?;
            map.insert(key, value);
        }
        Ok(Settings::from_map(&map))
    }

    pub fn day_start_hour(&self) -> u32 {
        self.settings().map(|s| s.day_start_hour).unwrap_or(stats::DEFAULT_DAY_START_HOUR)
    }

    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<Settings> {
        let parsed = Settings::validate(key, value)?;
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, parsed],
        )?;
        self.settings()
    }

    // ------------------------------------------------------------- mata pelajaran

    pub fn list_subjects(&self, include_archived: bool) -> Result<Vec<Subject>> {
        let sql = if include_archived {
            "SELECT id, name, color, archived, created_at FROM subjects ORDER BY archived, name COLLATE NOCASE"
        } else {
            "SELECT id, name, color, archived, created_at FROM subjects WHERE archived = 0 ORDER BY name COLLATE NOCASE"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], map_subject)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn subject(&self, id: i64) -> Result<Subject> {
        self.conn
            .query_row(
                "SELECT id, name, color, archived, created_at FROM subjects WHERE id = ?1",
                params![id],
                map_subject,
            )
            .optional()?
            .ok_or(CoreError::SubjectNotFound)
    }

    pub fn upsert_subject(&mut self, id: Option<i64>, name: &str, color: &str) -> Result<Subject> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::EmptySubjectName);
        }
        if !is_valid_color(color) {
            return Err(CoreError::UnknownColor(color.to_string()));
        }
        let clash: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM subjects WHERE name = ?1 COLLATE NOCASE AND id IS NOT ?2",
                params![name, id],
                |row| row.get(0),
            )
            .optional()?;
        if clash.is_some() {
            return Err(CoreError::DuplicateSubjectName(name.to_string()));
        }

        match id {
            Some(existing) => {
                self.conn.execute(
                    "UPDATE subjects SET name = ?2, color = ?3 WHERE id = ?1",
                    params![existing, name, color],
                )?;
                self.subject(existing)
            }
            None => {
                let now = stats::now_secs();
                self.conn.execute(
                    "INSERT INTO subjects (name, color, archived, created_at) VALUES (?1, ?2, 0, ?3)",
                    params![name, color, now],
                )?;
                self.subject(self.conn.last_insert_rowid())
            }
        }
    }

    pub fn archive_subject(&mut self, id: i64, archived: bool) -> Result<Subject> {
        self.conn.execute(
            "UPDATE subjects SET archived = ?2 WHERE id = ?1",
            params![id, i32::from(archived)],
        )?;
        self.subject(id)
    }

    pub fn delete_subject(&mut self, id: i64) -> Result<()> {
        self.subject(id)?;
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE subject_id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        if count > 0 {
            return Err(CoreError::SubjectHasSessions);
        }
        self.conn
            .execute("DELETE FROM subjects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn subject_totals(&self) -> Result<Vec<SubjectTotal>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.name, s.color,
                    COALESCE(SUM(x.duration_sec), 0) AS total,
                    COUNT(x.id) AS sessions
             FROM subjects s
             LEFT JOIN sessions x ON x.subject_id = s.id AND x.ended_at IS NOT NULL
             GROUP BY s.id
             ORDER BY s.archived, s.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SubjectTotal {
                subject_id: row.get(0)?,
                subject_name: row.get(1)?,
                color: row.get(2)?,
                secs: row.get(3)?,
                session_count: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // --------------------------------------------------------------------- timer

    pub fn active_session(&self) -> Result<Option<Session>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source
                 FROM sessions WHERE ended_at IS NULL LIMIT 1",
                [],
                map_session,
            )
            .optional()?)
    }

    /// Mata pelajaran yang terakhir dicatat. Dipakai pintasan global dan tray
    /// supaya mulai belajar cukup satu tombol.
    pub fn last_used_subject(&self) -> Result<Option<Subject>> {
        let id: Option<i64> = self
            .conn
            .query_row(
                "SELECT subject_id FROM sessions ORDER BY started_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(id.and_then(|id| self.subject(id).ok()))
    }

    pub fn timer_snapshot(&self) -> Result<TimerSnapshot> {
        self.timer_snapshot_at(stats::now_secs())
    }

    /// Snapshot pada waktu tertentu. Dipakai operasi timer supaya `elapsed_sec`
    /// dihitung dari `now` yang dikirim, bukan dari jam nyata.
    pub fn timer_snapshot_at(&self, now: i64) -> Result<TimerSnapshot> {
        let Some(session) = self.active_session()? else {
            return Ok(TimerSnapshot::idle());
        };
        let subject = self.subject(session.subject_id)?;
        let phase = timer::phase_of(&session);
        let elapsed = timer::elapsed_sec(
            session.started_at,
            session.paused_at,
            session.paused_total_sec,
            now,
        );
        Ok(TimerSnapshot {
            phase,
            session_id: Some(session.id),
            subject_id: Some(subject.id),
            subject_name: Some(subject.name),
            subject_color: Some(subject.color),
            started_at: Some(session.started_at),
            paused_at: session.paused_at,
            paused_total_sec: session.paused_total_sec,
            elapsed_sec: elapsed,
        })
    }

    pub fn start_session(&mut self, subject_id: i64, now: i64) -> Result<TimerSnapshot> {
        if self.active_session()?.is_some() {
            return Err(CoreError::SessionAlreadyRunning);
        }
        let subject = self.subject(subject_id)?;
        if subject.archived {
            return Err(CoreError::msg(
                "Mata pelajaran ini sudah diarsipkan. Pilih mata pelajaran aktif.",
            ));
        }
        self.conn.execute(
            "INSERT INTO sessions (subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source)
             VALUES (?1, ?2, NULL, NULL, 0, NULL, NULL, 'timer')",
            params![subject_id, now],
        )?;
        self.timer_snapshot_at(now)
    }

    fn with_active<F>(&mut self, now: i64, action: F) -> Result<TimerSnapshot>
    where
        F: FnOnce(&Session, i64) -> Result<ActiveUpdate>,
    {
        let session = self.active_session()?.ok_or(CoreError::NoActiveSession)?;
        let update = action(&session, now)?;
        self.conn.execute(
            "UPDATE sessions SET paused_at = ?2, paused_total_sec = ?3, ended_at = ?4, duration_sec = ?5
             WHERE id = ?1",
            params![
                session.id,
                update.paused_at,
                update.paused_total_sec,
                update.ended_at,
                update.duration_sec
            ],
        )?;
        self.timer_snapshot_at(now)
    }

    pub fn pause_session(&mut self, now: i64) -> Result<TimerSnapshot> {
        self.with_active(now, |session, now| {
            let paused_at = timer::pause(session, now)?;
            Ok(ActiveUpdate {
                paused_at: Some(paused_at),
                paused_total_sec: session.paused_total_sec,
                ended_at: None,
                duration_sec: None,
            })
        })
    }

    pub fn resume_session(&mut self, now: i64) -> Result<TimerSnapshot> {
        self.with_active(now, |session, now| {
            let total = timer::resume(session, now)?;
            Ok(ActiveUpdate {
                paused_at: None,
                paused_total_sec: total,
                ended_at: None,
                duration_sec: None,
            })
        })
    }

    /// Menghentikan sesi aktif dan menyimpannya. Tidak ada dialog: sesi langsung tersimpan.
    pub fn stop_session(&mut self, now: i64) -> Result<Session> {
        let session = self.active_session()?.ok_or(CoreError::NoActiveSession)?;
        let (ended_at, duration) = timer::stop(&session, now)?;
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?2, paused_at = NULL, duration_sec = ?3 WHERE id = ?1",
            params![session.id, ended_at, duration],
        )?;
        self.session_view(session.id)
    }

    /// Sesi tertinggal saat aplikasi dibuka kembali: pengguna memilih jam selesainya.
    pub fn finish_orphan_session(&mut self, ended_at: i64) -> Result<Session> {
        let session = self.active_session()?.ok_or(CoreError::NoActiveSession)?;
        let duration = timer::final_duration(session.started_at, ended_at, session.paused_total_sec)?;
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?2, paused_at = NULL, duration_sec = ?3 WHERE id = ?1",
            params![session.id, ended_at, duration],
        )?;
        self.session_view(session.id)
    }

    /// Buang sejumlah detik dari sesi aktif karena pengguna tidak aktif.
    /// Detik yang dibuang dicatat sebagai jeda, jadi durasi akhir ikut berkurang.
    pub fn trim_session(&mut self, id: i64, secs: i64) -> Result<TimerSnapshot> {
        let session = self.active_session()?.ok_or(CoreError::NoActiveSession)?;
        if session.id != id {
            return Err(CoreError::SessionNotFound);
        }
        let now = stats::now_secs();
        let running = timer::elapsed_sec(
            session.started_at,
            session.paused_at,
            session.paused_total_sec,
            now,
        );
        let drop = secs.clamp(0, running);
        if drop > 0 {
            self.conn.execute(
                "UPDATE sessions SET paused_total_sec = paused_total_sec + ?2 WHERE id = ?1",
                params![session.id, drop],
            )?;
        }
        self.timer_snapshot()
    }

    pub fn discard_session(&mut self, id: i64) -> Result<()> {
        let changed = self.conn.execute(
            "DELETE FROM sessions WHERE id = ?1 AND ended_at IS NULL",
            params![id],
        )?;
        if changed == 0 {
            return Err(CoreError::SessionNotFound);
        }
        Ok(())
    }

    /// Urungkan berhenti: hapus sesi yang baru saja tersimpan.
    pub fn undo_stop(&mut self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(CoreError::SessionNotFound);
        }
        Ok(())
    }

    // -------------------------------------------------------------------- sesi

    pub fn session_view(&self, id: i64) -> Result<Session> {
        self.conn
            .query_row(
                "SELECT id, subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source
                 FROM sessions WHERE id = ?1",
                params![id],
                map_session,
            )
            .optional()?
            .ok_or(CoreError::SessionNotFound)
    }

    pub fn list_sessions(&self, from: i64, to: i64) -> Result<Vec<SessionView>> {
        let now = stats::now_secs();
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.subject_id, s.started_at, s.ended_at, s.paused_at, s.paused_total_sec,
                    s.duration_sec, s.note, s.source, sub.name, sub.color
             FROM sessions s JOIN subjects sub ON sub.id = s.subject_id
             WHERE s.started_at < ?2 AND (s.ended_at IS NULL OR s.ended_at >= ?1)
             ORDER BY s.started_at",
        )?;
        let rows = stmt.query_map(params![from, to], |row| map_session_view(row, now))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn day_summary(&self, date: NaiveDate) -> Result<DaySummary> {
        let hour = self.day_start_hour();
        let (from, to) = stats::day_bounds(date, hour);
        let sessions = self.list_sessions(from, to)?;
        let total: i64 = sessions.iter().map(|s| s.effective_duration_sec).sum();
        Ok(DaySummary {
            date: date.format("%Y-%m-%d").to_string(),
            total_sec: total,
            sessions,
        })
    }

    pub fn add_manual_session(&mut self, input: &ManualSessionInput) -> Result<Session> {
        let duration = timer::final_duration(input.started_at, input.ended_at, 0)?;
        self.subject(input.subject_id)?;
        self.conn.execute(
            "INSERT INTO sessions (subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source)
             VALUES (?1, ?2, ?3, NULL, 0, ?4, ?5, 'manual')",
            params![
                input.subject_id,
                input.started_at,
                input.ended_at,
                duration,
                input.note
            ],
        )?;
        self.session_view(self.conn.last_insert_rowid())
    }

    pub fn update_session(
        &mut self,
        id: i64,
        input: &ManualSessionInput,
    ) -> Result<Session> {
        let duration = timer::final_duration(input.started_at, input.ended_at, 0)?;
        self.session_view(id)?;
        self.conn.execute(
            "UPDATE sessions SET subject_id = ?2, started_at = ?3, ended_at = ?4,
                    duration_sec = ?5, note = ?6, paused_at = NULL, paused_total_sec = 0
             WHERE id = ?1",
            params![
                id,
                input.subject_id,
                input.started_at,
                input.ended_at,
                duration,
                input.note
            ],
        )?;
        self.session_view(id)
    }

    pub fn delete_session(&mut self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(CoreError::SessionNotFound);
        }
        Ok(())
    }

    /// Apakah rentang ini bertumpang tindih dengan sesi lain? UI memberi peringatan,
    /// tapi penyimpanan tetap diizinkan.
    pub fn has_overlap(&self, started_at: i64, ended_at: i64, exclude_id: Option<i64>) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sessions
             WHERE id IS NOT ?3 AND started_at < ?2 AND (ended_at IS NULL OR ended_at > ?1)",
            params![started_at, ended_at, exclude_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ------------------------------------------------------------ target dan D-day

    pub fn goals(&self) -> Result<Goals> {
        let mut goals = Goals::default();
        let mut stmt = self.conn.prepare("SELECT scope, target_sec FROM goals")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (scope, secs) = row?;
            match GoalScope::parse(&scope) {
                Some(GoalScope::Daily) => goals.daily_sec = secs,
                Some(GoalScope::Weekly) => goals.weekly_sec = secs,
                None => {}
            }
        }
        Ok(goals)
    }

    pub fn set_goal(&mut self, scope: GoalScope, target_sec: i64) -> Result<Goals> {
        if target_sec <= 0 {
            return Err(CoreError::InvalidGoal);
        }
        self.conn.execute(
            "INSERT INTO goals (scope, target_sec) VALUES (?1, ?2)
             ON CONFLICT(scope) DO UPDATE SET target_sec = excluded.target_sec",
            params![scope.as_str(), target_sec],
        )?;
        self.goals()
    }

    pub fn list_exams(&self) -> Result<Vec<ExamView>> {
        let today = self.today();
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, exam_date FROM exams ORDER BY exam_date, title")?;
        let rows = stmt.query_map([], map_exam)?;
        let mut exams = Vec::new();
        for exam in rows {
            let exam = exam?;
            let days_left = exam
                .parsed_date()
                .map(|date| (date - today).num_days())
                .unwrap_or(0);
            exams.push(ExamView { exam, days_left });
        }
        Ok(exams)
    }

    pub fn upsert_exam(&mut self, id: Option<i64>, title: &str, exam_date: &str) -> Result<Exam> {
        let title = title.trim();
        if title.is_empty() {
            return Err(CoreError::msg("Judul D-day tidak boleh kosong."));
        }
        if NaiveDate::parse_from_str(exam_date, "%Y-%m-%d").is_err() {
            return Err(CoreError::msg(
                "Tanggal tidak dikenali. Pakai format tahun-bulan-hari, misalnya 2026-11-30.",
            ));
        }
        match id {
            Some(existing) => {
                self.conn.execute(
                    "UPDATE exams SET title = ?2, exam_date = ?3 WHERE id = ?1",
                    params![existing, title, exam_date],
                )?;
                self.exam(existing)
            }
            None => {
                self.conn.execute(
                    "INSERT INTO exams (title, exam_date) VALUES (?1, ?2)",
                    params![title, exam_date],
                )?;
                self.exam(self.conn.last_insert_rowid())
            }
        }
    }

    fn exam(&self, id: i64) -> Result<Exam> {
        self.conn
            .query_row(
                "SELECT id, title, exam_date FROM exams WHERE id = ?1",
                params![id],
                map_exam,
            )
            .optional()?
            .ok_or_else(|| CoreError::msg("D-day tidak ditemukan."))
    }

    pub fn delete_exam(&mut self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM exams WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(CoreError::msg("D-day tidak ditemukan."));
        }
        Ok(())
    }

    // ----------------------------------------------------------------- statistik

    pub fn today(&self) -> NaiveDate {
        stats::study_day(stats::now_secs(), self.day_start_hour())
    }

    /// Total per (hari belajar, mata pelajaran) dalam rentang.
    fn aggregate(
        &self,
        from: i64,
        to: i64,
    ) -> Result<BTreeMap<(NaiveDate, i64), i64>> {
        let hour = self.day_start_hour();
        let now = stats::now_secs();
        let mut stmt = self.conn.prepare(
            "SELECT subject_id, started_at, ended_at, paused_total_sec FROM sessions
             WHERE started_at < ?2 AND (ended_at IS NULL OR ended_at > ?1)",
        )?;
        let rows = stmt.query_map(params![from, to], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        let mut totals: BTreeMap<(NaiveDate, i64), i64> = BTreeMap::new();
        let mut per_day: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for row in rows {
            let (subject_id, started_at, ended_at, paused_total) = row?;
            let end = ended_at.unwrap_or(now);
            let mut split = BTreeMap::new();
            stats::split_session(started_at, end, paused_total, stats::local_offset_secs(started_at), hour, &mut split);
            for (day, secs) in split {
                *totals.entry((day, subject_id)).or_insert(0) += secs;
                *per_day.entry(day).or_insert(0) += secs;
            }
        }
        Ok(totals)
    }

    fn day_series(&self, from: i64, to: i64) -> Result<BTreeMap<NaiveDate, i64>> {
        let per_pair = self.aggregate(from, to)?;
        let mut days: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for ((day, _), secs) in per_pair {
            *days.entry(day).or_insert(0) += secs;
        }
        Ok(days)
    }

    pub fn today_total(&self) -> Result<i64> {
        let day = self.today();
        let hour = self.day_start_hour();
        let (from, to) = stats::day_bounds(day, hour);
        Ok(self.day_series(from, to)?.get(&day).copied().unwrap_or(0))
    }

    pub fn week_total(&self) -> Result<i64> {
        let start = stats::week_start(self.today());
        let hour = self.day_start_hour();
        let (from, _) = stats::day_bounds(start, hour);
        let (_, to) = stats::day_bounds(start + chrono::Duration::days(6), hour);
        let series = self.day_series(from, to)?;
        Ok(series.values().sum())
    }

    pub fn stats_range(
        &self,
        from: NaiveDate,
        to: NaiveDate,
        granularity: Granularity,
    ) -> Result<RangeStats> {
        let hour = self.day_start_hour();
        let span_days = (to - from).num_days() + 1;
        let (from_ts, _) = stats::day_bounds(from, hour);
        let (_, to_ts) = stats::day_bounds(to, hour);

        let pairs = self.aggregate(from_ts, to_ts)?;
        let subjects: Vec<Subject> = self.list_subjects(true)?;
        let lookup: BTreeMap<i64, &Subject> = subjects.iter().map(|s| (s.id, s)).collect();

        let mut per_day: BTreeMap<NaiveDate, Vec<BucketPart>> = BTreeMap::new();
        let mut per_day_total: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for ((day, subject_id), secs) in &pairs {
            if secs <= &0 {
                continue;
            }
            let (name, color) = lookup
                .get(subject_id)
                .map(|s| (s.name.clone(), s.color.clone()))
                .unwrap_or_else(|| ("Tanpa mata pelajaran".to_string(), "rose".to_string()));
            per_day.entry(*day).or_default().push(BucketPart {
                subject_id: *subject_id,
                subject_name: name,
                color,
                secs: *secs,
            });
            *per_day_total.entry(*day).or_insert(0) += *secs;
        }

        let mut buckets = Vec::new();
        let mut cursor = from;
        while cursor <= to {
            let total = per_day_total.get(&cursor).copied().unwrap_or(0);
            let mut parts = per_day.remove(&cursor).unwrap_or_default();
            parts.sort_by(|a, b| b.secs.cmp(&a.secs));
            buckets.push(Bucket {
                label: bucket_label(cursor, granularity),
                date: cursor.format("%Y-%m-%d").to_string(),
                total_sec: total,
                parts,
            });
            cursor += chrono::Duration::days(1);
        }

        let previous_to = from - chrono::Duration::days(1);
        let previous_from = previous_to - chrono::Duration::days(span_days - 1);
        let (pf, _) = stats::day_bounds(previous_from, hour);
        let (_, pt) = stats::day_bounds(previous_to, hour);
        let previous_total_sec = self.day_series(pf, pt)?.values().sum();

        let best = per_day_total
            .iter()
            .max_by_key(|(_, secs)| **secs)
            .map(|(day, secs)| (*day, *secs));

        let total_sec: i64 = per_day_total.values().sum();
        let session_count = self.count_sessions(from_ts, to_ts)?;

        Ok(RangeStats {
            from: from.format("%Y-%m-%d").to_string(),
            to: to.format("%Y-%m-%d").to_string(),
            total_sec,
            previous_total_sec,
            session_count,
            best_day_sec: best.map(|(_, secs)| secs).unwrap_or(0),
            best_day_date: best.map(|(day, _)| day.format("%Y-%m-%d").to_string()),
            buckets,
        })
    }

    fn count_sessions(&self, from: i64, to: i64) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE started_at < ?2 AND (ended_at IS NULL OR ended_at > ?1)",
            params![from, to],
            |row| row.get(0),
        )?)
    }

    pub fn subject_breakdown(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<SubjectTotal>> {
        let hour = self.day_start_hour();
        let (from_ts, _) = stats::day_bounds(from, hour);
        let (_, to_ts) = stats::day_bounds(to, hour);
        let pairs = self.aggregate(from_ts, to_ts)?;
        let subjects = self.list_subjects(true)?;

        let mut totals: BTreeMap<i64, i64> = BTreeMap::new();
        for ((_, subject_id), secs) in pairs {
            *totals.entry(subject_id).or_insert(0) += secs;
        }

        let mut counts: BTreeMap<i64, i64> = BTreeMap::new();
        {
            let mut stmt = self.conn.prepare(
                "SELECT subject_id, COUNT(*) FROM sessions
                 WHERE started_at < ?2 AND (ended_at IS NULL OR ended_at > ?1)
                 GROUP BY subject_id",
            )?;
            let rows = stmt.query_map(params![from_ts, to_ts], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?;
            for row in rows {
                let (subject_id, count) = row?;
                counts.insert(subject_id, count);
            }
        }

        let mut rows: Vec<SubjectTotal> = subjects
            .into_iter()
            .map(|s| SubjectTotal {
                secs: totals.get(&s.id).copied().unwrap_or(0),
                session_count: counts.get(&s.id).copied().unwrap_or(0),
                subject_id: s.id,
                subject_name: s.name,
                color: s.color,
            })
            .filter(|row| row.secs > 0)
            .collect();
        rows.sort_by(|a, b| b.secs.cmp(&a.secs));
        Ok(rows)
    }

    pub fn heatmap(&self, weeks: u32) -> Result<Heatmap> {
        let hour = self.day_start_hour();
        let goals = self.goals()?;
        let today = self.today();
        let first = stats::week_start(today) - chrono::Duration::days(i64::from(weeks.max(1) - 1) * 7);
        let (from, _) = stats::day_bounds(first, hour);
        let (_, to) = stats::day_bounds(today, hour);
        let totals = self.day_series(from, to)?;
        let (columns, max_sec) = stats::build_heatmap(&totals, today, weeks, goals.daily_sec);
        Ok(Heatmap { weeks: columns, max_sec })
    }

    pub fn streaks(&self) -> Result<Streaks> {
        let hour = self.day_start_hour();
        let today = self.today();
        let first = self.conn.query_row(
            "SELECT MIN(started_at) FROM sessions",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?;
        let from = match first {
            Some(ts) => stats::study_day(ts, hour),
            None => today,
        };
        let (from_ts, _) = stats::day_bounds(from, hour);
        let (_, to_ts) = stats::day_bounds(today, hour);
        let series = self.day_series(from_ts, to_ts)?;

        let active: std::collections::BTreeSet<NaiveDate> =
            series.iter().filter(|(_, secs)| **secs > 0).map(|(day, _)| *day).collect();

        let best_day = series.iter().max_by_key(|(_, secs)| **secs);
        let mut per_week: BTreeMap<NaiveDate, i64> = BTreeMap::new();
        for (day, secs) in &series {
            *per_week.entry(stats::week_start(*day)).or_insert(0) += *secs;
        }
        let best_week = per_week.iter().max_by_key(|(_, secs)| **secs);

        Ok(Streaks {
            current_days: stats::current_streak(&active, today),
            longest_days: stats::longest_streak(&active),
            best_day_sec: best_day.map(|(_, secs)| *secs).unwrap_or(0),
            best_day_date: best_day.map(|(day, _)| day.format("%Y-%m-%d").to_string()),
            best_week_sec: best_week.map(|(_, secs)| *secs).unwrap_or(0),
            best_week_start: best_week.map(|(day, _)| day.format("%Y-%m-%d").to_string()),
        })
    }

    /// Total per hari untuk sebuah rentang, dipakai kalender Riwayat.
    pub fn day_totals(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<(NaiveDate, i64)>> {
        let hour = self.day_start_hour();
        let (from_ts, _) = stats::day_bounds(from, hour);
        let (_, to_ts) = stats::day_bounds(to, hour);
        Ok(self.day_series(from_ts, to_ts)?.into_iter().collect())
    }
}

struct ActiveUpdate {
    paused_at: Option<i64>,
    paused_total_sec: i64,
    ended_at: Option<i64>,
    duration_sec: Option<i64>,
}

fn bucket_label(date: NaiveDate, granularity: Granularity) -> String {
    use chrono::Datelike;
    match granularity {
        Granularity::Day => {
            const NAMES: [&str; 7] = ["Sen", "Sel", "Rab", "Kam", "Jum", "Sab", "Min"];
            NAMES[date.weekday().num_days_from_monday() as usize].to_string()
        }
        Granularity::Week => format!("{}", date.day()),
        Granularity::Month => format!("{}", date.day()),
    }
}

fn map_subject(row: &Row<'_>) -> rusqlite::Result<Subject> {
    Ok(Subject {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        archived: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?,
    })
}

fn map_exam(row: &Row<'_>) -> rusqlite::Result<Exam> {
    Ok(Exam {
        id: row.get(0)?,
        title: row.get(1)?,
        exam_date: row.get(2)?,
    })
}

fn map_session(row: &Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        paused_at: row.get(4)?,
        paused_total_sec: row.get(5)?,
        duration_sec: row.get(6)?,
        note: row.get(7)?,
        source: SessionSource::parse(&row.get::<_, String>(8)?),
    })
}

fn map_session_view(row: &Row<'_>, now: i64) -> rusqlite::Result<SessionView> {
    let session = Session {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        paused_at: row.get(4)?,
        paused_total_sec: row.get(5)?,
        duration_sec: row.get(6)?,
        note: row.get(7)?,
        source: SessionSource::parse(&row.get::<_, String>(8)?),
    };
    let effective = session.duration_sec.unwrap_or_else(|| {
        timer::elapsed_sec(
            session.started_at,
            session.paused_at,
            session.paused_total_sec,
            now,
        )
    });
    Ok(SessionView {
        session,
        subject_name: row.get(9)?,
        subject_color: row.get(10)?,
        effective_duration_sec: effective,
    })
}

// ------------------------------------------------------------------- pengaturan

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub day_start_hour: u32,
    /// 0 berarti deteksi tidak aktif dimatikan.
    pub idle_threshold_min: u32,
    pub count_sleep_time: bool,
    pub close_to_tray: bool,
    pub autostart: bool,
    pub theme: String,
    pub ui_scale: u32,
    pub global_shortcut: String,
    pub remember_window: bool,
    pub language: String,
    pub mini_always_on_top: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            day_start_hour: stats::DEFAULT_DAY_START_HOUR,
            idle_threshold_min: 0,
            count_sleep_time: false,
            close_to_tray: true,
            autostart: false,
            theme: "system".to_string(),
            ui_scale: 100,
            global_shortcut: "Ctrl+Alt+S".to_string(),
            remember_window: true,
            language: "id".to_string(),
            mini_always_on_top: false,
        }
    }
}

impl Settings {
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Self {
        let mut settings = Settings::default();
        let get = |key: &str| map.get(key).map(|s| s.as_str());
        if let Some(v) = get("day_start_hour").and_then(|v| v.parse::<u32>().ok()) {
            settings.day_start_hour = v.min(12);
        }
        if let Some(v) = get("idle_threshold_min").and_then(|v| v.parse::<u32>().ok()) {
            settings.idle_threshold_min = v;
        }
        settings.count_sleep_time = truthy(get("count_sleep_time"));
        settings.close_to_tray = truthy(get("close_to_tray"));
        settings.autostart = truthy(get("autostart"));
        settings.remember_window = truthy(get("remember_window"));
        settings.mini_always_on_top = truthy(get("mini_always_on_top"));
        if let Some(v) = get("theme") {
            settings.theme = v.to_string();
        }
        if let Some(v) = get("ui_scale").and_then(|v| v.parse::<u32>().ok()) {
            settings.ui_scale = v;
        }
        if let Some(v) = get("global_shortcut") {
            settings.global_shortcut = v.to_string();
        }
        if let Some(v) = get("language") {
            settings.language = v.to_string();
        }
        settings
    }

    /// Validasi nilai sebelum disimpan, supaya database tidak berisi sampah.
    pub fn validate(key: &str, value: &str) -> Result<String> {
        let trimmed = value.trim();
        let ok = match key {
            "day_start_hour" => trimmed.parse::<u32>().map(|v| v <= 12).unwrap_or(false),
            "idle_threshold_min" => {
                matches!(trimmed, "0" | "5" | "10" | "15")
            }
            "count_sleep_time" | "close_to_tray" | "autostart" | "remember_window"
            | "mini_always_on_top" => matches!(trimmed, "true" | "false"),
            "theme" => matches!(trimmed, "light" | "dark" | "system"),
            "ui_scale" => matches!(trimmed, "90" | "100" | "110" | "125"),
            "global_shortcut" => !trimmed.is_empty(),
            "language" => matches!(trimmed, "id"),
            _ => return Err(CoreError::UnknownSetting(key.to_string())),
        };
        if ok {
            Ok(trimmed.to_string())
        } else {
            Err(CoreError::UnknownSetting(format!("{key} = {value}")))
        }
    }
}

fn truthy(value: Option<&str>) -> bool {
    matches!(value, Some("true") | Some("1"))
}
