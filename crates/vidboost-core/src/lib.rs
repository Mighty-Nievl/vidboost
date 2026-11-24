pub mod filters;
pub mod registry;
pub mod types;

pub use registry::JobRegistry;
pub use filters::{
    base_filters, choose_codec, drawtext_position, estimate_bitrate, filter_quote, fmt_num,
    output_extension, overlay_position, preset_cpu_used, safe_upscale_factor, CodecChoice,
};
pub use types::{
    now_ms, ColorSettings, DetailSettings, GeometrySettings, Job, JobStatus, OutputSettings,
    ProcessRequest, Settings, UpscaleSettings, VideoMeta, WatermarkSettings,
};

fn parse_timestamp(value: &str) -> Option<f64> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: f64 = parts[0].trim().parse().ok()?;
    let minutes: f64 = parts[1].trim().parse().ok()?;
    let seconds: f64 = parts[2].trim().parse().ok()?;
    Some((hours * 3600.0 + minutes * 60.0 + seconds) * 1_000_000.0)
}

/// Parsed elapsed time in microseconds dari baris `-progress pipe:1`.
pub fn parse_out_time(line: &str) -> Option<f64> {
    if let Some(value) = line.strip_prefix("out_time_us=") {
        return value.trim().parse().ok();
    }
    if let Some(value) = line.strip_prefix("out_time_ms=") {
        return value.trim().parse().ok();
    }
    if let Some(value) = line.strip_prefix("out_time=") {
        return parse_timestamp(value.trim());
    }
    None
}

pub fn progress_percent(elapsed_us: f64, duration: f64) -> u8 {
    if duration <= 0.0 || !elapsed_us.is_finite() {
        return 0;
    }
    ((elapsed_us / 1_000_000.0 / duration) * 100.0).clamp(0.0, 99.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_lines() {
        assert_eq!(parse_out_time("out_time_us=1500000"), Some(1_500_000.0));
        assert_eq!(parse_out_time("out_time=00:00:02.50"), Some(2_500_000.0));
    }

    #[test]
    fn caps_upscale_factor() {
        let factor = safe_upscale_factor(2.0, 1080, 1920, "libx264", 5_000_000);
        assert!(factor > 1.0 && factor < 2.0);
    }
}
