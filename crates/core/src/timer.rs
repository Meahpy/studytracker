//! State machine timer. Sumber kebenaran durasi adalah selisih timestamp,
//! bukan akumulasi per tick, jadi crash atau tidur komputer tidak membuat timer melenceng.

use crate::error::{CoreError, Result};
use crate::models::{Session, TimerPhase};

/// Sesi di atas 12 jam perlu konfirmasi sebelum disimpan.
pub const LONG_SESSION_CONFIRM_SEC: i64 = 12 * 3600;

/// Durasi tampilan saat berjalan (atau tetap saat dijeda).
pub fn elapsed_sec(started_at: i64, paused_at: Option<i64>, paused_total_sec: i64, now: i64) -> i64 {
    let end = paused_at.unwrap_or(now);
    (end - started_at - paused_total_sec).max(0)
}

/// Durasi final saat sesi dihentikan.
pub fn final_duration(started_at: i64, ended_at: i64, paused_total_sec: i64) -> Result<i64> {
    if ended_at < started_at {
        return Err(CoreError::InvalidTimeRange);
    }
    Ok((ended_at - started_at - paused_total_sec).max(0))
}

pub fn phase_of(session: &Session) -> TimerPhase {
    if session.ended_at.is_some() {
        TimerPhase::Idle
    } else if session.paused_at.is_some() {
        TimerPhase::Paused
    } else {
        TimerPhase::Running
    }
}

/// Jeda: catat `paused_at`. Sesi harus sedang berjalan.
pub fn pause(session: &Session, now: i64) -> Result<i64> {
    match phase_of(session) {
        TimerPhase::Paused => Err(CoreError::AlreadyPaused),
        TimerPhase::Idle => Err(CoreError::NoActiveSession),
        TimerPhase::Running => Ok(now),
    }
}

/// Lanjut: satu interval jeda dilipat ke `paused_total_sec`, lalu `paused_at` dikosongkan.
pub fn resume(session: &Session, now: i64) -> Result<i64> {
    match (phase_of(session), session.paused_at) {
        (TimerPhase::Paused, Some(paused_at)) => {
            Ok(session.paused_total_sec + (now - paused_at).max(0))
        }
        (TimerPhase::Idle, _) => Err(CoreError::NoActiveSession),
        _ => Err(CoreError::NotPaused),
    }
}

/// Berhenti: bila sedang dijeda, interval jeda terakhir ikut dilipat dulu.
pub fn stop(session: &Session, now: i64) -> Result<(i64, i64)> {
    match phase_of(session) {
        TimerPhase::Idle => Err(CoreError::NoActiveSession),
        TimerPhase::Running => {
            let duration = final_duration(session.started_at, now, session.paused_total_sec)?;
            Ok((now, duration))
        }
        TimerPhase::Paused => {
            let paused_at = session.paused_at.expect("paused phase selalu punya paused_at");
            // Sesi yang dijeda berakhir pada saat ia dijeda. Rentang jeda setelah itu
            // bukan bagian dari sesi, jadi tidak dilipat ke paused_total.
            let ended_at = paused_at.max(session.started_at);
            let duration = final_duration(session.started_at, ended_at, session.paused_total_sec)?;
            Ok((ended_at, duration))
        }
    }
}

/// Penanda sesi tertinggal: sesi aktif dari proses sebelumnya.
pub fn is_orphan(session: &Session) -> bool {
    session.ended_at.is_none()
}

pub fn needs_long_session_confirm(duration_sec: i64) -> bool {
    duration_sec > LONG_SESSION_CONFIRM_SEC
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionSource;

    fn session(started_at: i64, ended_at: Option<i64>, paused_at: Option<i64>, paused_total: i64) -> Session {
        Session {
            id: 1,
            subject_id: 1,
            started_at,
            ended_at,
            paused_at,
            paused_total_sec: paused_total,
            duration_sec: ended_at.map(|e| e - started_at - paused_total),
            note: None,
            source: SessionSource::Timer,
        }
    }

    #[test]
    fn elapsed_counts_from_timestamps() {
        assert_eq!(elapsed_sec(1_000, None, 0, 1_090), 90);
    }

    #[test]
    fn elapsed_frozen_while_paused() {
        let s = session(1_000, None, Some(1_060), 0);
        assert_eq!(elapsed_sec(1_000, s.paused_at, 0, 5_000), 60);
    }

    #[test]
    fn elapsed_never_negative_when_clock_moves_backwards() {
        assert_eq!(elapsed_sec(2_000, None, 0, 1_000), 0);
    }

    #[test]
    fn pause_then_resume_folds_the_gap() {
        let running = session(1_000, None, None, 0);
        let paused_at = pause(&running, 1_600).unwrap();
        assert_eq!(paused_at, 1_600);

        let paused = session(1_000, None, Some(paused_at), 0);
        let total = resume(&paused, 1_900).unwrap();
        assert_eq!(total, 300);

        let resumed = session(1_000, None, None, total);
        assert_eq!(elapsed_sec(resumed.started_at, resumed.paused_at, total, 2_500), 1_200);
    }

    #[test]
    fn resume_without_pause_is_rejected() {
        let running = session(1_000, None, None, 0);
        assert!(matches!(resume(&running, 1_100), Err(CoreError::NotPaused)));
    }

    #[test]
    fn double_pause_is_rejected() {
        let paused = session(1_000, None, Some(1_100), 0);
        assert!(matches!(pause(&paused, 1_200), Err(CoreError::AlreadyPaused)));
    }

    #[test]
    fn stop_while_paused_ends_at_pause_moment() {
        let paused = session(1_000, None, Some(1_600), 100);
        let (ended_at, duration) = stop(&paused, 3_000).unwrap();
        assert_eq!(ended_at, 1_600);
        // 1000 -> 1600 = 600 detik dikurangi 100 detik jeda sebelumnya.
        assert_eq!(duration, 500);
    }

    #[test]
    fn stop_while_running_uses_now() {
        let running = session(1_000, None, None, 60);
        let (ended_at, duration) = stop(&running, 1_660).unwrap();
        assert_eq!((ended_at, duration), (1_660, 600));
    }

    #[test]
    fn stop_without_session_is_rejected() {
        let done = session(1_000, Some(2_000), None, 0);
        assert!(matches!(stop(&done, 3_000), Err(CoreError::NoActiveSession)));
    }

    #[test]
    fn final_duration_rejects_backwards_clock() {
        assert!(matches!(
            final_duration(2_000, 1_000, 0),
            Err(CoreError::InvalidTimeRange)
        ));
    }

    #[test]
    fn long_session_needs_confirmation_above_twelve_hours() {
        assert!(!needs_long_session_confirm(12 * 3600));
        assert!(needs_long_session_confirm(12 * 3600 + 1));
    }
}
