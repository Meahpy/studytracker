//! Deteksi tidak aktif. Dibungkus trait supaya implementasi per OS terpisah dan
//! aplikasi tetap bisa dibangun di platform lain.

pub trait IdleMonitor: Send + Sync {
    /// Berapa detik sejak input terakhir pengguna.
    fn idle_secs(&self) -> u64;
}

/// Implementasi bawaan: membaca idle level sistem.
pub struct SystemIdle;

impl IdleMonitor for SystemIdle {
    fn idle_secs(&self) -> u64 {
        imp::idle_secs()
    }
}

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    pub fn idle_secs() -> u64 {
        // SAFETY: `LASTINPUTINFO` diisi ukuran struct sebelum dipakai Windows.
        unsafe {
            let mut info = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            if GetLastInputInfo(&mut info) == 0 {
                return 0;
            }
            let now = GetTickCount();
            u64::from(now.wrapping_sub(info.dwTime)) / 1_000
        }
    }
}

#[cfg(not(windows))]
mod imp {
    /// Platform tanpa implementasi: idle tidak pernah terdeteksi, fitur dimatikan.
    pub fn idle_secs() -> u64 {
        0
    }
}
