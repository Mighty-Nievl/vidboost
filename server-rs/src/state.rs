use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::process::Child;
use tokio::sync::Mutex;
use tracing::warn;

/// Drive Data besar dipakai sebagai lokasi default; fallback ke CWD bila tidak tersedia.
pub const DEFAULT_DATA_DIR: &str = "/run/media/rezal/Data/vidboost";
pub const FALLBACK_DATA_DIR: &str = ".data";

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    #[default]
    Ready,
    Processing,
    Done,
    Error,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VideoMeta {
    pub width: u64,
    pub height: u64,
    pub duration: f64,
    pub fps: f64,
    pub codec: String,
    pub bitrate: u64,
    pub mime: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub progress: u8,
    pub stage: String,
    pub error: Option<String>,
    pub file_name: String,
    pub size: u64,
    pub meta: VideoMeta,
    pub output_name: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Default for Job {
    fn default() -> Self {
        let now = now_ms();
        Self {
            id: String::new(),
            status: JobStatus::Ready,
            progress: 0,
            stage: String::new(),
            error: None,
            file_name: String::new(),
            size: 0,
            meta: VideoMeta::default(),
            output_name: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ColorSettings {
    pub enabled: bool,
    pub brightness: f64,
    pub contrast: f64,
    pub saturation: f64,
    pub gamma: f64,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DetailSettings {
    pub enabled: bool,
    pub sharpen: f64,
    pub denoise: f64,
}

impl Default for DetailSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sharpen: 25.0,
            denoise: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpscaleSettings {
    pub enabled: bool,
    pub factor: f64,
    pub ai: bool,
    pub ai_model: String,
}

impl Default for UpscaleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            factor: 2.0,
            ai: false,
            ai_model: "realesr-general-x4v3".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GeometrySettings {
    pub rotate: u32,
    pub crop_enabled: bool,
    pub crop_x: f64,
    pub crop_y: f64,
    pub crop_w: f64,
    pub crop_h: f64,
}

impl Default for GeometrySettings {
    fn default() -> Self {
        Self {
            rotate: 0,
            crop_enabled: false,
            crop_x: 0.0,
            crop_y: 0.0,
            crop_w: 100.0,
            crop_h: 100.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WatermarkSettings {
    pub mode: String,
    pub text: String,
    pub font_size: u32,
    pub color: String,
    pub position: String,
    pub logo_path: Option<String>,
    pub opacity: f64,
    pub scale: f64,
}

impl Default for WatermarkSettings {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
            text: String::new(),
            font_size: 48,
            color: "#ffffff".to_string(),
            position: "bottom-right".to_string(),
            logo_path: None,
            opacity: 1.0,
            scale: 20.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OutputSettings {
    pub format: String,
    pub codec: String,
    pub crf: u32,
    pub preset: String,
    pub audio: bool,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            format: "mp4".to_string(),
            codec: "auto".to_string(),
            crf: 23,
            preset: "medium".to_string(),
            audio: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub color: ColorSettings,
    pub detail: DetailSettings,
    pub upscale: UpscaleSettings,
    pub geometry: GeometrySettings,
    pub watermark: WatermarkSettings,
    pub output: OutputSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub id: String,
    #[serde(default)]
    pub settings: Settings,
}

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
