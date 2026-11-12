//! Ekspor dan impor data. Formatnya terbuka: JSON (lengkap) dan CSV (sesi saja).
//!
//! Impor selalu divalidasi penuh di memori sebelum satu baris pun ditulis, lalu
//! dijalankan dalam satu transaksi.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::models::*;
use crate::stats;
use crate::timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn parse(raw: &str) -> ExportFormat {
        match raw {
            "csv" => ExportFormat::Csv,
            _ => ExportFormat::Json,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub format: String,
    pub version: u32,
    pub exported_at: i64,
    pub subjects: Vec<Subject>,
    pub sessions: Vec<Session>,
    pub goals: Vec<GoalRow>,
    pub exams: Vec<Exam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalRow {
    pub scope: String,
    pub target_sec: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub subjects_added: usize,
    pub sessions_added: usize,
    pub exams_added: usize,
    pub goals_updated: usize,
}

/// Kumpulkan seluruh isi database menjadi satu bundle.
pub fn collect(conn: &Connection) -> Result<Bundle> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, archived, created_at FROM subjects ORDER BY id",
    )?;
    let subjects = stmt
        .query_map([], |row| {
            Ok(Subject {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                archived: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    let mut stmt = conn.prepare(
        "SELECT id, subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source
         FROM sessions ORDER BY started_at",
    )?;
    let sessions = stmt
        .query_map([], |row| {
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
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    let mut stmt = conn.prepare("SELECT scope, target_sec FROM goals")?;
    let goals = stmt
        .query_map([], |row| {
            Ok(GoalRow { scope: row.get(0)?, target_sec: row.get(1)? })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    let mut stmt = conn.prepare("SELECT id, title, exam_date FROM exams ORDER BY exam_date")?;
    let exams = stmt
        .query_map([], |row| {
            Ok(Exam { id: row.get(0)?, title: row.get(1)?, exam_date: row.get(2)? })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Bundle {
        format: "studytracker".to_string(),
        version: 1,
        exported_at: stats::now_secs(),
        subjects,
        sessions,
        goals,
        exams,
    })
}

pub fn export(conn: &Connection, path: &Path, format: ExportFormat) -> Result<usize> {
    let bundle = collect(conn)?;
    let text = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&bundle)?,
        ExportFormat::Csv => sessions_to_csv(&bundle),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(bundle.sessions.len())
}

const CSV_HEADER: &str =
    "id,mata_pelajaran,warna,mulai,selesai,durasi_detik,catatan,sumber";

fn sessions_to_csv(bundle: &Bundle) -> String {
    let subjects: BTreeMap<i64, (&str, &str)> = bundle
        .subjects
        .iter()
        .map(|s| (s.id, (s.name.as_str(), s.color.as_str())))
        .collect();

    let mut out = String::from(CSV_HEADER);
    out.push('\n');
    for session in &bundle.sessions {
        let (name, color) = subjects
            .get(&session.subject_id)
            .copied()
            .unwrap_or(("Tanpa mata pelajaran", "rose"));
        let started = fmt_iso(session.started_at);
        let ended = session.ended_at.map(fmt_iso).unwrap_or_default();
        let duration = session
            .duration_sec
            .unwrap_or_else(|| {
                timer::elapsed_sec(
                    session.started_at,
                    session.paused_at,
                    session.paused_total_sec,
                    stats::now_secs(),
                )
            })
            .to_string();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            session.id,
            csv_escape(name),
            color,
            started,
            ended,
            duration,
            csv_escape(session.note.as_deref().unwrap_or("")),
            session.source.as_str()
        ));
    }
    out
}

fn fmt_iso(ts: i64) -> String {
    match chrono::Local.timestamp_opt(ts, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => String::new(),
    }
}

use chrono::TimeZone;

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Baca file, deteksi format, dan validasi seluruh isinya sebelum menulis.
pub fn import(conn: &Connection, path: &Path) -> Result<ImportReport> {
    let text = std::fs::read_to_string(path).map_err(|_| {
        CoreError::msg("File tidak bisa dibaca. Pastikan file hasil ekspor StudyTracker ada dan tidak sedang dipakai program lain.")
    })?;
    let trimmed = text.trim_start();
    let report = if trimmed.starts_with('{') {
        import_json(conn, &text)?
    } else {
        import_csv(conn, &text)?
    };
    Ok(report)
}

fn import_json(conn: &Connection, text: &str) -> Result<ImportReport> {
    let bundle: Bundle = serde_json::from_str(text)?;
    if bundle.format != "studytracker" {
        return Err(CoreError::Format(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "format bukan studytracker",
        ))));
    }

    // Validasi seluruh sesi sebelum menulis apa pun.
    for session in &bundle.sessions {
        if !is_valid_color(&bundle_color(&bundle, session.subject_id)) {
            return Err(CoreError::UnknownColor(session.subject_id.to_string()));
        }
        if let Some(ended_at) = session.ended_at {
            if ended_at < session.started_at {
                return Err(CoreError::InvalidTimeRange);
            }
        }
    }

    let tx = conn.unchecked_transaction()?;
    let mut report = ImportReport {
        subjects_added: 0,
        sessions_added: 0,
        exams_added: 0,
        goals_updated: 0,
    };

    let mut id_map: BTreeMap<i64, i64> = BTreeMap::new();
    for subject in &bundle.subjects {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM subjects WHERE name = ?1 COLLATE NOCASE",
                params![subject.name],
                |row| row.get(0),
            )
            .optional()?;
        let id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO subjects (name, color, archived, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![subject.name, subject.color, i32::from(subject.archived), subject.created_at],
                )?;
                report.subjects_added += 1;
                tx.last_insert_rowid()
            }
        };
        id_map.insert(subject.id, id);
    }

    for session in &bundle.sessions {
        let subject_id = *id_map
            .get(&session.subject_id)
            .ok_or(CoreError::SubjectNotFound)?;
        let exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM sessions WHERE started_at = ?1 AND subject_id = ?2 AND ended_at IS ?3",
                params![session.started_at, subject_id, session.ended_at],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_some() {
            continue;
        }
        tx.execute(
            "INSERT INTO sessions (subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source)
             VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7)",
            params![
                subject_id,
                session.started_at,
                session.ended_at,
                session.paused_total_sec,
                session.duration_sec,
                session.note,
                session.source.as_str()
            ],
        )?;
        report.sessions_added += 1;
    }

    for goal in &bundle.goals {
        if GoalScope::parse(&goal.scope).is_some() && goal.target_sec > 0 {
            tx.execute(
                "INSERT INTO goals (scope, target_sec) VALUES (?1, ?2)
                 ON CONFLICT(scope) DO UPDATE SET target_sec = excluded.target_sec",
                params![goal.scope, goal.target_sec],
            )?;
            report.goals_updated += 1;
        }
    }

    for exam in &bundle.exams {
        if exam.parsed_date().is_none() {
            continue;
        }
        let exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM exams WHERE title = ?1 AND exam_date = ?2",
                params![exam.title, exam.exam_date],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_none() {
            tx.execute(
                "INSERT INTO exams (title, exam_date) VALUES (?1, ?2)",
                params![exam.title, exam.exam_date],
            )?;
            report.exams_added += 1;
        }
    }

    tx.commit()?;
    Ok(report)
}

fn bundle_color(bundle: &Bundle, subject_id: i64) -> String {
    bundle
        .subjects
        .iter()
        .find(|s| s.id == subject_id)
        .map(|s| s.color.clone())
        .unwrap_or_else(|| "rose".to_string())
}

fn import_csv(conn: &Connection, text: &str) -> Result<ImportReport> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| CoreError::msg("File CSV kosong. Ekspor dulu dari StudyTracker."))?;
    let columns = split_csv(header);
    let expected = ["mata_pelajaran", "mulai", "durasi_detik"];
    for column in expected {
        if !columns.iter().any(|c| c == column) {
            return Err(CoreError::msg(format!(
                "Kolom \"{column}\" tidak ditemukan di file CSV. Pakai file hasil ekspor StudyTracker."
            )));
        }
    }
    let index = |name: &str| columns.iter().position(|c| c == name);
    let idx_subject = index("mata_pelajaran").unwrap();
    let idx_color = index("warna");
    let idx_started = index("mulai").unwrap();
    let idx_ended = index("selesai");
    let idx_duration = index("durasi_detik").unwrap();
    let idx_note = index("catatan");
    let idx_source = index("sumber");

    struct Pending {
        subject: String,
        color: String,
        started_at: i64,
        ended_at: i64,
        duration: i64,
        note: Option<String>,
        source: String,
    }

    let mut pending = Vec::new();
    for line in lines {
        let cells = split_csv(line);
        let get = |i: Option<usize>| i.and_then(|i| cells.get(i)).map(|s| s.trim()).unwrap_or("");
        let subject = get(Some(idx_subject)).to_string();
        if subject.is_empty() {
            continue;
        }
        let started_at = parse_iso(get(Some(idx_started)))?;
        let duration: i64 = get(Some(idx_duration))
            .parse()
            .map_err(|_| CoreError::msg("Kolom durasi_detik berisi angka yang tidak valid."))?;
        let color = match get(idx_color) {
            "" => "rose".to_string(),
            value if is_valid_color(value) => value.to_string(),
            _ => "rose".to_string(),
        };
        let ended_at = match get(idx_ended) {
            "" => started_at + duration,
            value => parse_iso(value)?,
        };
        if ended_at < started_at {
            return Err(CoreError::InvalidTimeRange);
        }
        pending.push(Pending {
            subject,
            color,
            started_at,
            ended_at,
            duration,
            note: match get(idx_note) {
                "" => None,
                v => Some(v.to_string()),
            },
            source: match get(idx_source) {
                "manual" => "manual".to_string(),
                _ => "timer".to_string(),
            },
        });
    }

    let tx = conn.unchecked_transaction()?;
    let mut report = ImportReport {
        subjects_added: 0,
        sessions_added: 0,
        exams_added: 0,
        goals_updated: 0,
    };

    for row in pending {
        let existing: Option<i64> = tx
            .query_row(
                "SELECT id FROM subjects WHERE name = ?1 COLLATE NOCASE",
                params![row.subject],
                |row| row.get(0),
            )
            .optional()?;
        let subject_id = match existing {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO subjects (name, color, archived, created_at) VALUES (?1, ?2, 0, ?3)",
                    params![row.subject, row.color, stats::now_secs()],
                )?;
                report.subjects_added += 1;
                tx.last_insert_rowid()
            }
        };
        let exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM sessions WHERE started_at = ?1 AND subject_id = ?2",
                params![row.started_at, subject_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_some() {
            continue;
        }
        tx.execute(
            "INSERT INTO sessions (subject_id, started_at, ended_at, paused_at, paused_total_sec, duration_sec, note, source)
             VALUES (?1, ?2, ?3, NULL, 0, ?4, ?5, ?6)",
            params![subject_id, row.started_at, row.ended_at, row.duration, row.note, row.source],
        )?;
        report.sessions_added += 1;
    }

    tx.commit()?;
    Ok(report)
}

fn parse_iso(value: &str) -> Result<i64> {
    if let Ok(ts) = value.parse::<i64>() {
        return Ok(ts);
    }
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, format) {
            if let Some(dt) = chrono::Local.from_local_datetime(&naive).earliest() {
                return Ok(dt.timestamp());
            }
        }
    }
    Err(CoreError::msg(format!(
        "Waktu \"{value}\" tidak dikenali. Contoh yang benar: 2026-11-12 09:15:00."
    )))
}

fn split_csv(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                cells.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    cells.push(current);
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_round_trip_keeps_quoted_notes() {
        let cells = split_csv("1,Matematika,rose,2026-11-12 09:00:00,,3600,\"bab 3, latihan\",timer");
        assert_eq!(cells.len(), 8);
        assert_eq!(cells[6], "bab 3, latihan");
    }

    #[test]
    fn csv_escape_quotes_and_commas() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("kata \"kutip\""), "\"kata \"\"kutip\"\"\"");
        assert_eq!(csv_escape("polos"), "polos");
    }

    #[test]
    fn iso_timestamps_parse_back_to_epoch() {
        let ts = parse_iso("2026-11-12 09:15:00").unwrap();
        assert_eq!(fmt_iso(ts), "2026-11-12 09:15:00");
    }
}
