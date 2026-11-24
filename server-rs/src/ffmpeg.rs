use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::ai;
use crate::state::{AppState, Settings, VideoMeta};
use vidboost_core::{
    base_filters, choose_codec, drawtext_position, filter_quote, fmt_num, output_extension,
    overlay_position, parse_out_time, safe_upscale_factor,
};

pub const CANCELLED_MESSAGE: &str = "Dibatalkan oleh pengguna";

async fn track_child(state: &Arc<AppState>, id: &str, child: Child) -> Arc<Mutex<Option<Child>>> {
    let holder = Arc::new(Mutex::new(Some(child)));
    state.children.lock().await.insert(id.to_string(), holder.clone());
    holder
}

async fn untrack_child(state: &Arc<AppState>, id: &str) {
    state.children.lock().await.remove(id);
}

/// Jalankan proses yang bisa dibatalkan. Child tetap disimpan di registry
/// supaya `cancel_job` bisa mengirim sinyal kill kapan saja.
async fn run_tracked(
    state: &Arc<AppState>,
    id: &str,
    command: &mut Command,
    timeout_secs: u64,
) -> Result<(), String> {
    let mut child = command.spawn().map_err(|err| err.to_string())?;
    let stderr_reader = child.stderr.take().map(|mut stderr| {
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match stderr.read(&mut chunk).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if buffer.len() < 2000 {
                            let take = n.min(2000 - buffer.len());
                            buffer.extend_from_slice(&chunk[..take]);
                        }
                    }
                }
            }
            String::from_utf8_lossy(&buffer).trim().to_string()
        })
    });

    let holder = track_child(state, id, child).await;
    let started = std::time::Instant::now();
    let mut status: Option<std::process::ExitStatus> = None;
    let mut timed_out = false;

    loop {
        if started.elapsed().as_secs() >= timeout_secs {
            timed_out = true;
            if let Some(mut child) = holder.lock().await.take() {
                let _ = child.start_kill();
            }
            break;
        }
        let polled = {
            let mut guard = holder.lock().await;
            match guard.as_mut() {
                Some(child) => child.try_wait().map_err(|err| err.to_string()),
                None => Ok(None),
            }
        };
        match polled {
            Ok(Some(value)) => {
                status = Some(value);
                break;
            }
            Ok(None) => {
                if holder.lock().await.is_none() {
                    break;
                }
            }
            Err(err) => {
                untrack_child(state, id).await;
                return Err(err);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    untrack_child(state, id).await;
    let stderr = match stderr_reader {
        Some(handle) => handle.await.unwrap_or_default(),
        None => String::new(),
    };

    if timed_out {
        return Err(format!("Proses dihentikan setelah {timeout_secs} detik (batas aman CPU/GPU)."));
    }
    match status {
        Some(value) if value.success() => Ok(()),
        Some(_) => Err(stderr),
        None => Err(CANCELLED_MESSAGE.to_string()),
    }
}

/// Kirim sinyal kill ke proses yang sedang berjalan untuk job ini.
pub async fn cancel_job(state: &Arc<AppState>, id: &str) -> bool {
    let holder = state.children.lock().await.remove(id);
    match holder {
        Some(holder) => {
            if let Some(mut child) = holder.lock().await.take() {
                return child.start_kill().is_ok();
            }
            false
        }
        None => false,
    }
}

/// Versi ffmpeg untuk GET /api/status.
pub async fn check_ffmpeg(ffmpeg_path: &str) -> (bool, String) {
    match Command::new(ffmpeg_path).arg("-version").output().await {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let version = text.lines().next().unwrap_or("").trim().to_string();
            (true, version)
        }
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stderr);
            (false, text.lines().next().unwrap_or("").trim().to_string())
        }
        Err(err) => (false, err.to_string()),
    }
}

/// Encoder yang benar-benar tersedia pada build ffmpeg ini.
pub async fn detect_encoders(ffmpeg_path: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    if let Ok(output) = Command::new(ffmpeg_path)
        .args(["-hide_banner", "-encoders"])
        .output()
        .await
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if !line.starts_with(' ') {
                continue;
            }
            let mut parts = line.split_whitespace();
            parts.next();
            if let Some(name) = parts.next() {
                found.insert(name.to_string());
            }
        }
    }
    found
}

fn json_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn parse_rate(value: &str) -> Option<f64> {
    let (num, den) = value.split_once('/')?;
    let den: f64 = den.trim().parse().ok()?;
    if den == 0.0 {
        return None;
    }
    let num: f64 = num.trim().parse().ok()?;
    let rate = num / den;
    (rate.is_finite() && rate > 0.0).then_some(rate)
}

pub async fn probe_video(state: &AppState, input: &Path, mime: &str) -> Result<VideoMeta, String> {
    let output = Command::new(&state.ffprobe_path)
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(input)
        .output()
        .await
        .map_err(|err| format!("Gagal menjalankan ffprobe: {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe gagal: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("Gagal membaca hasil ffprobe: {err}"))?;

    let streams = json.get("streams").and_then(Value::as_array);
    let video = streams
        .and_then(|list| {
            list.iter()
                .find(|stream| stream.get("codec_type").and_then(Value::as_str) == Some("video"))
                .cloned()
        })
        .unwrap_or(Value::Null);
    let format = json.get("format").cloned().unwrap_or(Value::Null);

    let fps = video
        .get("avg_frame_rate")
        .and_then(Value::as_str)
        .and_then(parse_rate)
        .or_else(|| {
            video
                .get("r_frame_rate")
                .and_then(Value::as_str)
                .and_then(parse_rate)
        })
        .unwrap_or(0.0);

    let duration = video
        .get("duration")
        .and_then(json_f64)
        .or_else(|| format.get("duration").and_then(json_f64))
        .unwrap_or(0.0);

    let bitrate = format
        .get("bit_rate")
        .and_then(json_f64)
        .or_else(|| video.get("bit_rate").and_then(json_f64))
        .unwrap_or(0.0);

    Ok(VideoMeta {
        width: video
            .get("width")
            .and_then(Value::as_u64)
            .or_else(|| video.get("coded_width").and_then(Value::as_u64))
            .unwrap_or(0),
        height: video
            .get("height")
            .and_then(Value::as_u64)
            .or_else(|| video.get("coded_height").and_then(Value::as_u64))
            .unwrap_or(0),
        duration,
        fps,
        codec: video
            .get("codec_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        bitrate: bitrate as u64,
        mime: mime.to_string(),
    })
}

pub async fn make_thumbnail(
    state: &AppState,
    input: &Path,
    poster: &Path,
    duration: f64,
) -> Result<(), String> {
    if let Some(parent) = poster.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| format!("Gagal membuat folder thumbnail: {err}"))?;
    }

    let time = if duration.is_finite() && duration > 0.0 {
        1.0_f64.min(duration / 2.0)
    } else {
        0.5
    };

    let output = Command::new(&state.ffmpeg_path)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{time:.3}"))
        .arg("-i")
        .arg(input)
        .args(["-frames:v", "1", "-q:v", "4", "-update", "1"])
        .arg(poster)
        .output()
        .await
        .map_err(|err| format!("Gagal membuat thumbnail: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Thumbnail gagal: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// Cari file secara lokal di folder job, mis. `input.` atau `enhanced.`.
pub fn find_named(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut matches: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(prefix))
        })
        .collect();
    matches.sort();
    matches.into_iter().next()
}

async fn text_watermark_filter(state: &AppState, id: &str, settings: &Settings) -> Result<String, String> {
    let wm = &settings.watermark;
    let dir = state.output_dir(id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|err| format!("Gagal membuat folder output: {err}"))?;
    let text_file = dir.join("wm-text.txt");
    tokio::fs::write(&text_file, wm.text.as_bytes())
        .await
        .map_err(|err| format!("Gagal menulis file teks watermark: {err}"))?;

    let (x, y) = drawtext_position(&wm.position);
    let font = state
        .fontfile
        .as_ref()
        .map(|path| format!("fontfile={}:", filter_quote(path)))
        .unwrap_or_default();

    Ok(format!(
        "drawtext={font}textfile={}:fontsize={}:fontcolor={}:x={}:y={}:box=1:boxcolor=black@0.35:boxborderw=8",
        filter_quote(&text_file),
        wm.font_size,
        wm.color,
        x,
        y
    ))
}

struct EncodePlan {
    args: Vec<String>,
    output_path: PathBuf,
    output_name: String,
}

async fn build_encode_plan(
    state: &AppState,
    id: &str,
    settings: &Settings,
    meta: &VideoMeta,
    ai_frames: Option<(&Path, &Path)>,
) -> Result<EncodePlan, String> {
    let ai_mode = ai_frames.is_some();
    let codec = choose_codec(&state.encoders, settings, meta);
    let requested_factor = settings.upscale.factor;
    let safe_factor = safe_upscale_factor(
        requested_factor,
        meta.width,
        meta.height,
        &codec.name,
        state.max_output_pixels,
    );
    let mut filters = base_filters(settings, !ai_mode, safe_factor);

    if ai_mode && safe_factor < requested_factor && requested_factor > 0.0 {
        let ratio = safe_factor / requested_factor;
        filters.push(format!(
            "scale=trunc(iw*{ratio}/2)*2:trunc(ih*{ratio}/2)*2:flags=lanczos"
        ));
    }

    let wm = &settings.watermark;
    let image_watermark = wm.mode == "image" && wm.logo_path.as_deref().is_some_and(|p| !p.is_empty());
    let text_watermark = wm.mode == "text" && !wm.text.is_empty();

    if text_watermark && !image_watermark {
        filters.push(text_watermark_filter(state, id, settings).await?);
    }

    let extension = output_extension(&settings.output.format);
    let output_dir = state.output_dir(id);
    let output_name = format!("enhanced.{extension}");
    let output_path = output_dir.join(&output_name);

    let mut args: Vec<String> = vec![
        "-nostdin".into(),
        "-nostats".into(),
        "-progress".into(),
        "pipe:1".into(),
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];

    let video_index = 0usize;
    let audio_index = if ai_mode { 1usize } else { 0usize };

    match ai_frames {
        Some((_frames, up)) => {
            let fps = if meta.fps.is_finite() && meta.fps > 0.0 {
                meta.fps.min(240.0)
            } else {
                30.0
            };
            args.push("-framerate".into());
            args.push(fmt_num(fps));
            args.push("-i".into());
            args.push(up.join("%06d.png").to_string_lossy().into_owned());
            // Audio berasal dari file input asli.
            args.push("-i".into());
            args.push(
                find_named(&state.upload_dir(id), "input.")
                    .ok_or("File input tidak ditemukan")?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        None => {
            args.push("-i".into());
            args.push(
                find_named(&state.upload_dir(id), "input.")
                    .ok_or("File input tidak ditemukan")?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }

    let mut video_map = "0:v:0".to_string();

    if image_watermark {
        let logo_path = wm.logo_path.as_deref().unwrap_or("");
        let logo = Path::new(logo_path);
        if !logo.is_file() {
            return Err(format!("File logo tidak ditemukan: {logo_path}"));
        }
        let logo_index = if ai_mode { 2 } else { 1 };
        args.push("-loop".into());
        args.push("1".into());
        args.push("-i".into());
        args.push(logo.to_string_lossy().into_owned());

        let scale = (wm.scale / 100.0).clamp(0.01, 10.0);
        let alpha = wm.opacity.clamp(0.0, 1.0);
        let (x, y) = overlay_position(&wm.position);

        let logo_chain = format!(
            "[{logo_index}:v]scale=iw*{}:-1,format=rgba,colorchannelmixer=aa={}[logo]",
            fmt_num(scale),
            fmt_num(alpha)
        );
        let base_chain = if filters.is_empty() {
            format!("[{video_index}:v]null[base]")
        } else {
            format!("[{video_index}:v]{}[base]", filters.join(","))
        };
        let overlay_chain = format!(
            "[base][logo]overlay=x='{x}':y='{y}':format=auto[vout]"
        );
        args.push("-filter_complex".into());
        args.push(format!("{logo_chain};{base_chain};{overlay_chain}"));
        video_map = "[vout]".to_string();
    } else if !filters.is_empty() {
        args.push("-vf".into());
        args.push(filters.join(","));
    }

    args.push("-map".into());
    args.push(video_map);

    if settings.output.audio {
        args.push("-map".into());
        args.push(format!("{audio_index}:a?"));
        if settings.output.format == "webm" {
            args.push("-c:a".into());
            args.push("libopus".into());
        } else {
            args.push("-c:a".into());
            args.push("aac".into());
        }
        args.push("-b:a".into());
        args.push("160k".into());
    } else {
        args.push("-an".into());
    }

    args.push("-c:v".into());
    args.push(codec.name);
    args.extend(codec.quality);
    args.push("-pix_fmt".into());
    args.push("yuv420p".into());

    if matches!(settings.output.format.as_str(), "mp4" | "mov") {
        args.push("-movflags".into());
        args.push("+faststart".into());
    }
    if settings.output.format == "mov" {
        args.push("-f".into());
        args.push("mov".into());
    }

    if image_watermark {
        args.push("-shortest".into());
    }

    args.push(output_path.to_string_lossy().into_owned());

    Ok(EncodePlan {
        args,
        output_path,
        output_name,
    })
}

async fn run_encode(
    state: &Arc<AppState>,
    id: &str,
    args: Vec<String>,
    duration: f64,
) -> Result<(), String> {
    let mut command = Command::new(&state.ffmpeg_path);
    command
        .arg("-threads")
        .arg(&state.ffmpeg_threads)
        .args(args)
        .arg("-threads")
        .arg(&state.ffmpeg_threads)
        .arg("-filter_threads")
        .arg(&state.ffmpeg_threads)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|err| format!("Gagal menjalankan ffmpeg: {err}"))?;

    // Paralel: kuras stderr supaya pipe tidak penuh saat stdout dibaca baris per baris.
    let stderr_reader = child.stderr.take().map(|mut stderr| {
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match stderr.read(&mut chunk).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if buffer.len() < 2000 {
                            let take = n.min(2000 - buffer.len());
                            buffer.extend_from_slice(&chunk[..take]);
                        }
                    }
                }
            }
            String::from_utf8_lossy(&buffer).trim().to_string()
        })
    });

    let stdout = child
        .stdout
        .take()
        .ok_or("Tidak bisa membaca progres ffmpeg")?;
    let holder = track_child(state, id, child).await;

    // Watchdog: kalau encode tidak selesai dalam 15 menit, hentikan.
    let watchdog_holder = holder.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(900)).await;
        if let Some(mut child) = watchdog_holder.lock().await.take() {
            let _ = child.start_kill();
            warn!("encode dihentikan watchdog setelah 900 detik");
        }
    });

    let mut lines = BufReader::new(stdout).lines();
    let mut last_progress = 0u8;
    let mut speed = String::new();

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(time_us) = parse_out_time(&line) {
            if duration > 0.0 && time_us.is_finite() {
                let percent =
                    ((time_us / 1_000_000.0 / duration) * 100.0).clamp(0.0, 99.0) as u8;
                if percent > last_progress {
                    last_progress = percent;
                    let stage = if speed.is_empty() || speed == "N/A" {
                        format!("Mengencode video · {percent}%")
                    } else {
                        format!("Mengencode · {percent}% · kecepatan {speed}x realtime")
                    };
                    state
                        .update_job(id, |job| {
                            job.progress = percent;
                            job.stage = stage;
                        })
                        .await;
                }
            }
        } else if let Some(value) = line.strip_prefix("speed=") {
            speed = value.trim().to_string();
        }
    }
    drop(lines);

    let status = {
        let mut guard = holder.lock().await;
        match guard.as_mut() {
            Some(child) => Some(
                child
                    .wait()
                    .await
                    .map_err(|err| format!("Gagal menunggu ffmpeg: {err}"))?,
            ),
            None => None,
        }
    };
    untrack_child(state, id).await;

    let stderr = match stderr_reader {
        Some(handle) => handle.await.unwrap_or_default(),
        None => String::new(),
    };

    match status {
        Some(value) if value.success() => Ok(()),
        Some(value) if stderr.is_empty() => Err(format!("ffmpeg gagal dengan kode {:?}", value.code())),
        Some(_) => Err(stderr),
        None => Err(CANCELLED_MESSAGE.to_string()),
    }
}

async fn set_error(state: &Arc<AppState>, id: &str, message: impl Into<String>) {
    let message = message.into();
    let cancelled = state
        .get_job(id)
        .await
        .and_then(|job| job.error)
        .is_some_and(|error| error == CANCELLED_MESSAGE);
    if cancelled {
        return;
    }
    state
        .update_job(id, |job| {
            job.status = crate::state::JobStatus::Error;
            job.error = Some(message);
            job.stage = "Gagal".to_string();
        })
        .await;
}

/// Orkestrasi penuh: AI (opsional) → encode → update job.
pub async fn process_job(state: Arc<AppState>, id: String, settings: Settings) {
    let meta = state
        .get_job(&id)
        .await
        .map(|job| job.meta)
        .unwrap_or_default();
    let ai_mode = settings.upscale.enabled && settings.upscale.ai;

    // AI bekerja per-frame. Video 4K/60fps bisa menghasilkan ribuan PNG dan
    // menguras CPU/RAM. Tolak lebih awal dengan pesan yang bisa ditindaklanjuti.
    if ai_mode {
        let too_large = meta.width > 1280 || meta.height > 1280;
        let too_fast = meta.fps > 30.0;
        let too_long = meta.duration > 10.0;
        if too_large || too_fast || too_long {
            let reason = if too_large {
                format!("{}x{}", meta.width, meta.height)
            } else if too_fast {
                format!("{} fps", meta.fps.round())
            } else {
                format!("{} detik", meta.duration.round())
            };
            set_error(
                &state,
                &id,
                format!(
                    "AI upscale tidak didukung untuk video {reason}. Batas aman: 720p, 30 fps, 10 detik. Pakai Lanczos atau resize video dulu."
                ),
            )
            .await;
            return;
        }
    }

    let upload_dir = state.upload_dir(&id);
    let output_dir = state.output_dir(&id);

    if let Err(err) = tokio::fs::create_dir_all(&output_dir).await {
        warn!(%err, "gagal membuat folder output");
    }

    let mut ai_cleanup: Option<(PathBuf, PathBuf)> = None;
    let mut ai_small_dir: Option<PathBuf> = None;

    if ai_mode {
        let binary = match ai::resolve_binary(&state) {
            Some(binary) => binary,
            None => {
                state
                    .update_job(&id, |job| {
                        job.status = crate::state::JobStatus::Error;
                        job.error = Some(ai::AI_NOT_CONFIGURED.to_string());
                        job.stage = "Gagal".to_string();
                    })
                    .await;
                return;
            }
        };

        let input = match find_named(&upload_dir, "input.") {
            Some(path) => path,
            None => {
                set_error(&state, &id, "File input tidak ditemukan").await;
                return;
            }
        };

        state
            .update_job(&id, |job| {
                job.stage = "Menyiapkan frame untuk AI…".to_string();
            })
            .await;

        let ai_root = upload_dir.join("_ai");
        let frames_dir = ai_root.join("frames");
        let up_dir = ai_root.join("up");
        for dir in [&frames_dir, &up_dir] {
            if let Err(err) = tokio::fs::create_dir_all(dir).await {
                set_error(&state, &id, format!("Gagal membuat folder AI: {err}")).await;
                return;
            }
        }

        // -fps_mode passthrough = pengganti `-vsync 0` pada ffmpeg 9.
        let mut extract_command = Command::new(&state.ffmpeg_path);
        extract_command
            .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
            .arg(&input)
            .args(["-threads", "2", "-fps_mode", "passthrough"])
            .arg(frames_dir.join("%06d.png"))
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if let Err(err) = run_tracked(&state, &id, &mut extract_command, 300).await {
            set_error(&state, &id, format!("Gagal mengekstrak frame: {err}")).await;
            return;
        }

        state
            .update_job(&id, |job| {
                job.stage = "Meningkatkan resolusi dengan AI…".to_string();
            })
            .await;

        // FSRCNN bekerja per-frame di CPU. Kecilkan frame dulu ke lebar 640
        // supaya beban tetap terikat, berapapun resolusi sumbernya.
        let small_dir = ai_root.join("small");
        if let Err(err) = tokio::fs::create_dir_all(&small_dir).await {
            set_error(&state, &id, format!("Gagal membuat folder AI: {err}")).await;
            return;
        }
        let mut resize_command = Command::new(&state.ffmpeg_path);
        resize_command
            .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
            .arg(frames_dir.join("%06d.png"))
            .args(["-vf", "scale=640:-2", "-threads", "2"])
            .arg(small_dir.join("%06d.png"));
        if let Err(err) = run_tracked(&state, &id, &mut resize_command, 300).await {
            set_error(&state, &id, format!("Gagal menyiapkan frame AI: {err}")).await;
            return;
        }
        ai_small_dir = Some(small_dir.clone());

        let factor = settings.upscale.factor.round().max(1.0) as u32;
        let models_dir = binary
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("models");
        let mut upscale_command = Command::new(&binary);
        upscale_command
            .args(["-i"])
            .arg(&small_dir)
            .args(["-o"])
            .arg(&up_dir)
            .args(["-n", &settings.upscale.ai_model, "-s"])
            .arg(factor.to_string())
            .args(["-t", "128", "-j", "1:1:1", "-f", "png"]);
        if let Ok(gpu) = std::env::var("AI_UPSCALER_GPU") {
            if !gpu.trim().is_empty() {
                upscale_command.arg("-g").arg(gpu.trim());
            }
        }
        upscale_command
            .arg("-m")
            .arg(&models_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        if let Err(err) = run_tracked(&state, &id, &mut upscale_command, 300).await {
            set_error(&state, &id, format!("AI upscaler gagal: {err}")).await;
            return;
        }

        ai_cleanup = Some((frames_dir, up_dir));
        state
            .update_job(&id, |job| {
                job.stage = "Mengencode hasil AI…".to_string();
            })
            .await;
    } else {
        state
            .update_job(&id, |job| {
                job.stage = "Menyiapkan encode…".to_string();
            })
            .await;
    }

    let plan = match build_encode_plan(
        &state,
        &id,
        &settings,
        &meta,
        ai_cleanup.as_ref().map(|(frames, up)| (frames.as_path(), up.as_path())),
    )
    .await
    {
        Ok(plan) => plan,
        Err(err) => {
            set_error(&state, &id, err).await;
            return;
        }
    };

    let result = run_encode(&state, &id, plan.args, meta.duration).await;

    if let Some((frames, up)) = ai_cleanup {
        let _ = tokio::fs::remove_dir_all(&frames).await;
        let _ = tokio::fs::remove_dir_all(&up).await;
    }
    if let Some(small) = ai_small_dir {
        let _ = tokio::fs::remove_dir_all(&small).await;
    }

    match result {
        Ok(()) => {
            if plan.output_path.exists() {
                state
                    .update_job(&id, |job| {
                        job.status = crate::state::JobStatus::Done;
                        job.progress = 100;
                        job.stage = "Selesai".to_string();
                        job.error = None;
                        job.output_name = Some(plan.output_name.clone());
                    })
                    .await;
                info!(id = %id, "encode selesai");
            } else {
                set_error(&state, &id, "Encode selesai tetapi file output tidak ditemukan").await;
            }
        }
        Err(err) => {
            // Buang file setengah jadi supaya download/preview tidak menyajikan video rusak.
            let _ = tokio::fs::remove_file(&plan.output_path).await;
            set_error(&state, &id, err).await;
        }
    }
}
