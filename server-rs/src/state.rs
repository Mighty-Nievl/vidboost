use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::process::Child;
use tokio::sync::Mutex;
use tracing::warn;

/// Drive Data besar dipakai sebagai lokasi default; fallback ke CWD bila tidak tersedia.
pub const DEFAULT_DATA_DIR: &str = "/run/media/rezal/Data/vidboost";
pub const FALLBACK_DATA_DIR: &str = ".data";

pub use vidboost_core::types::*;

/// State bersama. `jobs` memakai Mutex async supaya update progres aman lintas task.
#[derive(Clone)]
pub struct AppState {
    pub jobs: Arc<Mutex<HashMap<String, Job>>>,
    pub data_dir: PathBuf,
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub ai_upscaler_bin: Option<String>,
    pub fontfile: Option<PathBuf>,
    /// Daftar encoder yang benar-benar ada pada build ffmpeg (untuk fallback libx264).
    pub encoders: Arc<HashSet<String>>,
    /// Proses anak yang sedang berjalan per job, dipakai tombol Batal.
    pub children: Arc<Mutex<HashMap<String, Arc<Mutex<Option<Child>>>>>>,
    /// Batas thread ffmpeg saat encode (mengikuti jumlah vCPU).
    pub ffmpeg_threads: String,
    /// Batas total pixel output agar tidak menguras RAM.
    pub max_output_pixels: u64,
}

impl AppState {
    pub fn upload_dir(&self, id: &str) -> PathBuf {
        self.data_dir.join("uploads").join(id)
    }

    pub fn output_dir(&self, id: &str) -> PathBuf {
        self.data_dir.join("outputs").join(id)
    }

    pub async fn get_job(&self, id: &str) -> Option<Job> {
        let jobs = self.jobs.lock().await;
        jobs.get(id).cloned()
    }

    pub async fn upsert_job(&self, job: Job) {
        let id = job.id.clone();
        let mut jobs = self.jobs.lock().await;
        jobs.insert(id, job);
        if let Err(err) = crate::jobs::save_jobs(&self.data_dir, &jobs).await {
            warn!(%err, "gagal menyimpan jobs.json");
        }
    }

    /// Ubah job lalu persist (atomik). Dipanggil tiap kali progres naik.
    pub async fn update_job<F>(&self, id: &str, f: F) -> Option<Job>
    where
        F: FnOnce(&mut Job) + Send,
    {
        let mut jobs = self.jobs.lock().await;
        let job = jobs.get_mut(id)?;
        f(job);
        job.updated_at = now_ms();
        let updated = job.clone();
        if let Err(err) = crate::jobs::save_jobs(&self.data_dir, &jobs).await {
            warn!(%err, "gagal menyimpan jobs.json");
        }
        Some(updated)
    }
}

/// Tentukan direktori data: env DATA_DIR → lokasi default di drive Data → CWD/.data.
pub async fn resolve_data_dir() -> PathBuf {
    let preferred = std::env::var("DATA_DIR")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_DATA_DIR.to_string());

    for candidate in [preferred, FALLBACK_DATA_DIR.to_string()] {
        let path = PathBuf::from(candidate);
        if tokio::fs::create_dir_all(&path).await.is_ok() {
            if let Ok(absolute) = std::fs::canonicalize(&path) {
                return absolute;
            }
        }
    }

    PathBuf::from(FALLBACK_DATA_DIR)
}

/// Cari font untuk drawtext; path sistem didahulukan, lalu env DRAWTEXT_FONTFILE.
pub fn find_fontfile() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        "/usr/share/fonts/dejavu-sans-fonts/DejaVuSans.ttf",
        "/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf",
    ];

    for candidate in CANDIDATES {
        let path = Path::new(candidate);
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }

    std::env::var("DRAWTEXT_FONTFILE")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_file())
}
