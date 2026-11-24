use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

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
            ai_model: "fsrcnn".to_string(),
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
            preset: "veryfast".to_string(),
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
