use std::collections::HashSet;
use std::path::Path;

use crate::types::{Settings, VideoMeta};

pub fn fmt_num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    if value.fract() == 0.0 {
        return format!("{}", value as i64);
    }
    let text = format!("{value:.4}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn filter_quote(path: &Path) -> String {
    format!(
        "'{}'",
        path.to_string_lossy()
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
    )
}

pub fn drawtext_position(position: &str) -> (&'static str, &'static str) {
    match position {
        "top-left" => ("24", "24"),
        "top-right" => ("w-tw-24", "24"),
        "bottom-left" => ("24", "h-th-24"),
        "center" => ("(w-tw)/2", "(h-th)/2"),
        _ => ("w-tw-24", "h-th-24"),
    }
}

/// ffmpeg 9 memakai `main_w`/`overlay_w`; alias lama `ow`/`oh` tidak dikenal.
pub fn overlay_position(position: &str) -> (&'static str, &'static str) {
    match position {
        "top-left" => ("24", "24"),
        "top-right" => ("main_w-overlay_w-24", "24"),
        "bottom-left" => ("24", "main_h-overlay_h-24"),
        "center" => ("(main_w-overlay_w)/2", "(main_h-overlay_h)/2"),
        _ => ("main_w-overlay_w-24", "main_h-overlay_h-24"),
    }
}

/// Turunkan faktor upscale supaya encode tidak melewati batas pixel.
pub fn safe_upscale_factor(
    requested: f64,
    width: u64,
    height: u64,
    codec: &str,
    global_max_pixels: u64,
) -> f64 {
    let codec_max = if codec == "libopenh264" {
        9_200_000.0
    } else {
        f64::MAX
    };
    let max_pixels = codec_max.min(global_max_pixels as f64);
    let source = (width as f64) * (height as f64);
    if source <= 0.0 || requested <= 1.0 {
        return requested;
    }
    let allowed = (max_pixels / source).sqrt().max(1.0);
    requested.min(allowed)
}

pub fn base_filters(settings: &Settings, include_upscale: bool, upscale_factor: f64) -> Vec<String> {
    let mut filters = Vec::new();

    match settings.geometry.rotate {
        90 => filters.push("transpose=clock".to_string()),
        180 => filters.push("hflip,vflip".to_string()),
        270 => filters.push("transpose=cclock".to_string()),
        _ => {}
    }

    if settings.geometry.crop_enabled {
        let x = settings.geometry.crop_x.clamp(0.0, 90.0);
        let y = settings.geometry.crop_y.clamp(0.0, 90.0);
        let w = settings.geometry.crop_w.clamp(10.0, 100.0 - x).max(10.0);
        let h = settings.geometry.crop_h.clamp(10.0, 100.0 - y).max(10.0);
        filters.push(format!(
            "crop=trunc(iw*{}/2)*2:trunc(ih*{}/2)*2:trunc(iw*{}/2)*2:trunc(ih*{}/2)*2",
            fmt_num(w / 100.0),
            fmt_num(h / 100.0),
            fmt_num(x / 100.0),
            fmt_num(y / 100.0)
        ));
    }

    if settings.color.enabled {
        filters.push(format!(
            "eq=brightness={}:contrast={}:saturation={}:gamma={}",
            fmt_num(settings.color.brightness),
            fmt_num(settings.color.contrast),
            fmt_num(settings.color.saturation),
            fmt_num(settings.color.gamma)
        ));
    }

    if settings.detail.enabled {
        if settings.detail.denoise > 0.0 {
            let s = settings.detail.denoise / 12.5;
            filters.push(format!(
                "hqdn3d={}:{}:{}:{}",
                fmt_num(s),
                fmt_num(s * 1.5),
                fmt_num(s * 2.0),
                fmt_num(s * 3.0)
            ));
        }
        if settings.detail.sharpen > 0.0 {
            let amount = settings.detail.sharpen / 50.0;
            filters.push(format!("unsharp=5:5:{}:5:5:0", fmt_num(amount)));
        }
    }

    if include_upscale && settings.upscale.enabled && !settings.upscale.ai && upscale_factor > 0.0 {
        let factor = fmt_num(upscale_factor);
        filters.push(format!(
            "scale=trunc(iw*{factor}/2)*2:trunc(ih*{factor}/2)*2:flags=lanczos"
        ));
    }

    filters
}

pub fn preset_cpu_used(preset: &str) -> u32 {
    match preset {
        "ultrafast" => 8,
        "superfast" => 7,
        "veryfast" => 6,
        "faster" => 5,
        "fast" => 4,
        "slow" => 2,
        "slower" => 1,
        "veryslow" => 0,
        _ => 3,
    }
}

pub fn output_extension(format: &str) -> String {
    match format {
        "mov" | "webm" | "mkv" | "avi" => format.to_string(),
        _ => "mp4".to_string(),
    }
}

pub fn estimate_bitrate(width: u64, height: u64, fps: f64, crf: u32) -> u64 {
    let pixels = (width.max(1) as f64) * (height.max(1) as f64) * fps.max(1.0).min(120.0);
    let factor = (23.0 / crf.max(1) as f64).powf(1.4);
    (pixels * 0.12 * factor).clamp(300_000.0, 40_000_000.0) as u64
}

#[derive(Clone, Debug)]
pub struct CodecChoice {
    pub name: String,
    pub quality: Vec<String>,
}

pub fn choose_codec(
    encoders: &HashSet<String>,
    settings: &Settings,
    meta: &VideoMeta,
) -> CodecChoice {
    let output = &settings.output;
    let is_webm = output.format == "webm";
    let crf = output.crf.to_string();

    let needs_openh264_fallback = !is_webm
        && matches!(output.codec.as_str(), "auto" | "h264")
        && !encoders.contains("libx264")
        && encoders.contains("libopenh264");

    if needs_openh264_fallback {
        let bitrate = estimate_bitrate(meta.width, meta.height, meta.fps, output.crf);
        return CodecChoice {
            name: "libopenh264".to_string(),
            quality: vec!["-b:v".to_string(), bitrate.to_string()],
        };
    }

    let name = match output.codec.as_str() {
        "h264" => "libx264",
        "hevc" => "libx265",
        "vp9" => "libvpx-vp9",
        "av1" => "libaom-av1",
        _ if is_webm => "libvpx-vp9",
        _ => "libx264",
    }
    .to_string();

    let quality = if name == "libx264" {
        vec![
            "-crf".to_string(),
            crf,
            "-preset".to_string(),
            output.preset.clone(),
            "-profile:v".to_string(),
            "high".to_string(),
            "-level:v".to_string(),
            "5.1".to_string(),
        ]
    } else if name == "libx265" {
        vec![
            "-crf".to_string(),
            crf,
            "-preset".to_string(),
            output.preset.clone(),
        ]
    } else {
        vec![
            "-b:v".to_string(),
            "0".to_string(),
            "-crf".to_string(),
            crf,
            "-deadline".to_string(),
            "good".to_string(),
            "-cpu-used".to_string(),
            preset_cpu_used(&output.preset).to_string(),
        ]
    };

    CodecChoice { name, quality }
}
