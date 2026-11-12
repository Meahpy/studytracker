//! Error domain tunggal untuk seluruh logika core.
//!
//! Setiap pesan ditulis untuk pengguna akhir (bahasa Indonesia, sentence case,
//! menjelaskan masalah dan cara memperbaikinya) karena string ini naik apa adanya
//! ke UI lewat `tauri::command`.

use serde::{Serialize, Serializer};

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Sesi belum tersimpan. File data sedang dipakai program lain. Tutup program itu, lalu coba lagi.")]
    Database(#[from] rusqlite::Error),

    #[error("Database tidak bisa dibuka. Periksa izin folder data, lalu jalankan ulang StudyTracker.")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("Data tidak bisa ditulis ke disk. Periksa ruang penyimpanan dan izin folder data.")]
    Io(#[from] std::io::Error),

    #[error("File tidak bisa dibaca. Pastikan file hasil ekspor StudyTracker berformat JSON atau CSV.")]
    Format(#[from] serde_json::Error),

    #[error("Mata pelajaran tidak ditemukan.")]
    SubjectNotFound,

    #[error("Sesi tidak ditemukan.")]
    SessionNotFound,

    #[error("Masih ada sesi yang berjalan. Hentikan dulu sesi itu sebelum memulai yang baru.")]
    SessionAlreadyRunning,

    #[error("Belum ada sesi yang berjalan.")]
    NoActiveSession,

    #[error("Timer sedang jeda. Lanjutkan dulu sebelum menjeda lagi.")]
    AlreadyPaused,

    #[error("Timer tidak sedang jeda.")]
    NotPaused,

    #[error("Waktu selesai lebih awal dari waktu mulai. Periksa jam sistem, lalu perbaiki waktunya.")]
    InvalidTimeRange,

    #[error("Nama mata pelajaran tidak boleh kosong.")]
    EmptySubjectName,

    #[error("Mata pelajaran \"{0}\" sudah ada. Pakai nama lain.")]
    DuplicateSubjectName(String),

    #[error("Mata pelajaran ini punya riwayat sesi, jadi tidak bisa dihapus. Arsipkan saja.")]
    SubjectHasSessions,

    #[error("Warna \"{0}\" bukan warna StudyTracker yang tersedia.")]
    UnknownColor(String),

    #[error("Nilai untuk \"{0}\" tidak dikenal.")]
    UnknownSetting(String),

    #[error("Target harus lebih dari nol.")]
    InvalidGoal,

    #[error("{0}")]
    Message(String),
}

impl CoreError {
    pub fn msg(text: impl Into<String>) -> Self {
        CoreError::Message(text.into())
    }
}

/// Frontend hanya butuh pesan yang bisa ditampilkan, bukan tipe error Rust.
impl Serialize for CoreError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
