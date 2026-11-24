use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::types::{Job, JobStatus, now_ms};

/// Job registry bersama, dipakai web backend maupun Tauri commands.
#[derive(Clone, Debug, Default)]
pub struct JobRegistry {
    jobs: Arc<Mutex<HashMap<String, Job>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(&self, job: Job) {
        self.jobs.lock().await.insert(job.id.clone(), job);
    }

    pub async fn get(&self, id: &str) -> Option<Job> {
        self.jobs.lock().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<Job> {
        let mut items: Vec<Job> = self.jobs.lock().await.values().cloned().collect();
        items.sort_by_key(|job| job.created_at);
        items
    }

    pub async fn update<F>(&self, id: &str, mutate: F)
    where
        F: FnOnce(&mut Job),
    {
        if let Some(job) = self.jobs.lock().await.get_mut(id) {
            mutate(job);
            job.updated_at = now_ms();
        }
    }

    pub async fn mark_interrupted(&self) -> usize {
        let mut count = 0;
        for job in self.jobs.lock().await.values_mut() {
            if job.status == JobStatus::Processing {
                job.status = JobStatus::Error;
                job.error = Some("Server restarted saat encode berjalan.".to_string());
                job.stage = "Gagal".to_string();
                job.updated_at = now_ms();
                count += 1;
            }
        }
        count
    }
}
