//! State aplikasi yang dibagi antar command.

use std::path::PathBuf;
use std::sync::Mutex;

use studytracker_core::{CoreError, Store};

pub struct AppState {
    pub store: Mutex<Store>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, Store>, CoreError> {
        self.store
            .lock()
            .map_err(|_| CoreError::msg("Data sedang dipakai. Coba lagi sebentar lagi."))
    }
}
