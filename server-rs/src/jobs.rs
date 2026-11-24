use std::collections::HashMap;
use std::path::Path;

use tracing::{info, warn};

use crate::state::{now_ms, Job, JobStatus};

/// Muat jobs.json. Job yang masih `processing` saat restart ditandai error.
pub async fn load_jobs(data_dir: &Path) -> HashMap<String, Job> {
    let path = data_dir.join("jobs.json");
    let raw = match tokio::fs::read_to_string(&path).await {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return HashMap::new(),
        Err(err) => {
            warn!(%err, "gagal membaca jobs.json, mulai dengan daftar kosong");
            return HashMap::new();
        }
    };

    let mut jobs: HashMap<String, Job> = match serde_json::from_str(&raw) {
        Ok(jobs) => jobs,
        Err(err) => {
            warn!(%err, "jobs.json rusak, mulai dengan daftar kosong");
            return HashMap::new();
        }
    };

    let mut interrupted = 0usize;
    for job in jobs.values_mut() {
        if job.status == JobStatus::Processing {
            job.status = JobStatus::Error;
            job.error = Some("Server restarted saat encode berjalan.".to_string());
            job.stage = "Gagal — server restart".to_string();
            job.updated_at = now_ms();
            interrupted += 1;
        }
    }

    if interrupted > 0 {
        info!(interrupted, "menandai job yang terputus saat restart");
        if let Err(err) = save_jobs(data_dir, &jobs).await {
            warn!(%err, "gagal menyimpan jobs.json setelah recovery");
        }
    }

    jobs
}

/// Tulis atomik: tulis ke file temp dulu, lalu rename menimpa jobs.json.
pub async fn save_jobs(
    data_dir: &Path,
    jobs: &HashMap<String, Job>,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(jobs)
        .map_err(|err| std::io::Error::other(format!("gagal serialisasi jobs: {err}")))?;
    let temp = data_dir.join(format!("jobs.json.{}.tmp", std::process::id()));
    tokio::fs::write(&temp, json).await?;
    tokio::fs::rename(&temp, data_dir.join("jobs.json")).await?;
    Ok(())
}
