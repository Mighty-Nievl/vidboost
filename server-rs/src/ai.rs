use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::state::AppState;

pub const AI_NOT_CONFIGURED: &str = "AI upscaler belum dikonfigurasi. Set AI_UPSCALER_BIN ke path upscaler (lihat docs/MODELS.md).";
pub const DEFAULT_AI_BINARY: &str = "fsrcnn-upscaler.sh";

#[derive(Clone, Debug, Serialize)]
pub struct AiModel {
    pub id: String,
    pub name: String,
    pub model: String,
    pub scale: u32,
    pub runtime: String,
    pub size_mb: u32,
    pub vram: String,
    pub ram: String,
    pub speed: String,
    pub quality: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub note: String,
    pub recommended: bool,
}

pub fn models() -> Vec<AiModel> {
    vec![
        AiModel {
            id: "fsrcnn".into(),
            name: "FSRCNN".into(),
            model: "fsrcnn".into(),
            scale: 4,
            runtime: "opencv-cpu".into(),
            size_mb: 1,
            vram: "0".into(),
            ram: "~200 MB".into(),
            speed: "sedang".into(),
            quality: "sedang".into(),
            pros: vec!["Paling ringan, aman di RAM 1 GB".into(), "Sudah terpasang & teruji".into()],
            cons: vec!["Detail masih modest".into()],
            note: "Pilihan default. Model 39 KB, jalan di CPU tanpa Vulkan.".into(),
            recommended: true,
        },
        AiModel {
            id: "espcn".into(),
            name: "ESPCN".into(),
            model: "espcn".into(),
            scale: 4,
            runtime: "opencv-cpu".into(),
            size_mb: 1,
            vram: "0".into(),
            ram: "~150 MB".into(),
            speed: "paling cepat".into(),
            quality: "rendah".into(),
            pros: vec!["Paling cepat di CPU".into(), "Model ~100 KB".into()],
            cons: vec!["Garis bisa terlihat licin".into(), "Kualitas di bawah FSRCNN".into()],
            note: "Cocok untuk video panjang yang ingin tetap ringan.".into(),
            recommended: false,
        },
        AiModel {
            id: "lapsrn".into(),
            name: "LapSRN".into(),
            model: "lapsrn".into(),
            scale: 8,
            runtime: "opencv-cpu".into(),
            size_mb: 4,
            vram: "0".into(),
            ram: "~400 MB".into(),
            speed: "sedang".into(),
            quality: "cukup baik".into(),
            pros: vec!["Lebih tajam dari FSRCNN".into(), "Bisa sampai 8x".into()],
            cons: vec!["Model 4 MB".into(), "Lebih lambat & boram RAM".into()],
            note: "Perlu unduh model LapSRN_x8.pb".into(),
            recommended: false,
        },
        AiModel {
            id: "edsr".into(),
            name: "EDSR".into(),
            model: "edsr".into(),
            scale: 4,
            runtime: "opencv-cpu".into(),
            size_mb: 38,
            vram: "0".into(),
            ram: "~1 GB".into(),
            speed: "lambat".into(),
            quality: "tinggi".into(),
            pros: vec!["Detail paling tajam di kelas CPU".into()],
            cons: vec!["Model 38 MB".into(), "Berat untuk VPS 2 vCPU".into()],
            note: "Perlu unduh model EDSR_x4.pb".into(),
            recommended: false,
        },
        AiModel {
            id: "realesr-animevideov3".into(),
            name: "Real-ESRGAN Anime v3".into(),
            model: "realesr-animevideov3".into(),
            scale: 4,
            runtime: "vulkan".into(),
            size_mb: 1,
            vram: "~600 MB".into(),
            ram: "~300 MB".into(),
            speed: "cepat di GPU".into(),
            quality: "baik (anime)".into(),
            pros: vec!["Model 1,2 MB".into(), "Cepat & bagus untuk animasi".into()],
            cons: vec!["Butuh Vulkan/GPU".into(), "Pernah bikin Radeon Vega 8 wedge".into()],
            note: "Hanya disarankan di device dengan Vulkan stabil.".into(),
            recommended: false,
        },
        AiModel {
            id: "realesrgan-x4plus".into(),
            name: "Real-ESRGAN x4plus".into(),
            model: "realesrgan-x4plus".into(),
            scale: 4,
            runtime: "vulkan".into(),
            size_mb: 32,
            vram: "~1,5 GB".into(),
            ram: "~600 MB".into(),
            speed: "lambat".into(),
            quality: "tertinggi".into(),
            pros: vec!["Kualitas terbaik untuk video nyata".into()],
            cons: vec!["Model 32 MB".into(), "Butuh Vulkan & GPU kuat".into()],
            note: "Pakai hanya di device dengan GPU diskrit/Vulkan stabil.".into(),
            recommended: false,
        },
    ]
}

/// Cari binary di PATH.
fn which(binary: &str) -> Option<PathBuf> {
    if binary.contains('/') {
        let path = PathBuf::from(binary);
        return path.is_file().then_some(path);
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Hasil pengecekan untuk GET /api/status.
pub fn check_binary(state: &AppState) -> (bool, String, String) {
    let configured = state
        .ai_upscaler_bin
        .clone()
        .filter(|s| !s.trim().is_empty());
    let binary = configured.clone().unwrap_or_else(|| DEFAULT_AI_BINARY.to_string());

    match which(&binary) {
        Some(path) => {
            let display = path.to_string_lossy().into_owned();
            (true, display, binary)
        }
        None => (false, String::new(), binary),
    }
}

/// Path binary yang dipakai proses AI. None kalau belum dikonfigurasi / tidak ada.
pub fn resolve_binary(state: &AppState) -> Option<PathBuf> {
    let configured = state
        .ai_upscaler_bin
        .clone()
        .filter(|s| !s.trim().is_empty())?;
    let path = Path::new(&configured);
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    which(&configured)
}
