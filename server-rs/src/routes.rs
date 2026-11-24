use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::Multipart;
use futures_util::TryStreamExt;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::warn;
use uuid::Uuid;

use crate::ai;
use crate::ffmpeg;
use crate::state::{now_ms, AppState, Job, JobStatus, ProcessRequest};

#[derive(Debug)]
pub struct ApiError(StatusCode, String);

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, message.into())
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, message.into())
    }

    fn internal(message: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, message.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

/// Terima UUID kanonik; untuk id non-UUID tetap aman selama hanya berisi karakter URI aman.
fn normalize_id(raw: &str) -> Result<String, ApiError> {
    let trimmed = raw.trim();
    if let Ok(uuid) = Uuid::parse_str(trimmed) {
        return Ok(uuid.to_string());
    }
    let cleaned: String = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .take(64)
        .collect();
    if cleaned.is_empty() {
        Err(ApiError::bad_request("id tidak valid"))
    } else {
        Ok(cleaned)
    }
}

fn query_id(query: &HashMap<String, String>) -> Result<String, ApiError> {
    let raw = query
        .get("id")
        .map(String::as_str)
        .ok_or_else(|| ApiError::bad_request("Parameter id wajib diisi"))?;
    normalize_id(raw)
}

fn extension_from_name(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            ext.to_ascii_lowercase()
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .take(8)
                .collect::<String>()
        })
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "mp4".to_string())
}

async fn save_uploaded_field(mut multipart: Multipart, path: &Path) -> Result<(String, Option<String>), ApiError> {
    let field = loop {
        match multipart
            .next_field()
            .await
            .map_err(|err| ApiError::bad_request(format!("Multipart tidak valid: {err}")))?
        {
            Some(field) if field.name() == Some("file") => break field,
            Some(_) => continue,
            None => return Err(ApiError::bad_request("Field 'file' wajib diisi")),
        }
    };

    let file_name = field.file_name().unwrap_or("input.mp4").to_string();
    let content_type = field.content_type().map(str::to_string);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| ApiError::internal(format!("Gagal membuat folder: {err}")))?;
    }

    let mut target = tokio::fs::File::create(path)
        .await
        .map_err(|err| ApiError::internal(format!("Gagal membuat file: {err}")))?;

    // Stream langsung ke disk, tidak menampung seluruh file di memory.
    let stream = field.map_err(|err| std::io::Error::other(err.to_string()));
    let mut reader = StreamReader::new(stream);
    tokio::io::copy(&mut reader, &mut target)
        .await
        .map_err(|err| ApiError::internal(format!("Gagal menyimpan file: {err}")))?;
    let _ = target.flush().await;

    Ok((file_name, content_type))
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Json<Job>, ApiError> {
    let id = Uuid::new_v4().to_string();
    let dir = state.upload_dir(&id);
    let ext = "mp4";
    let path = dir.join(format!("input.{ext}"));

    // Ekstensi asli belum diketahui sebelum field dibaca; simpan dulu sebagai .mp4 lalu ganti bila perlu.
    let (file_name, content_type) = save_uploaded_field(multipart, &path).await?;
    let real_ext = extension_from_name(&file_name);
    if real_ext != ext {
        let real_path = dir.join(format!("input.{real_ext}"));
        tokio::fs::rename(&path, &real_path)
            .await
            .map_err(|err| ApiError::internal(format!("Gagal mengganti nama file: {err}")))?;
    }

    let input_path = ffmpeg::find_named(&dir, "input.")
        .ok_or_else(|| ApiError::internal("File upload tidak ditemukan"))?;
    let size = tokio::fs::metadata(&input_path)
        .await
        .map(|meta| meta.len())
        .unwrap_or(0);

    let mime = content_type
        .filter(|value| value.starts_with("video/") || value.starts_with("audio/"))
        .unwrap_or_else(|| {
            mime_guess::from_path(&file_name)
                .first_or_octet_stream()
                .essence_str()
                .to_string()
        });

    let mut meta = ffmpeg::probe_video(&state, &input_path, &mime)
        .await
        .unwrap_or_else(|err| {
            warn!(%err, "gagal membaca metadata video");
            crate::state::VideoMeta {
                mime: mime.clone(),
                ..Default::default()
            }
        });

    let poster = dir.join("poster.jpg");
    if let Err(err) = ffmpeg::make_thumbnail(&state, &input_path, &poster, meta.duration).await {
        warn!(%err, "gagal membuat thumbnail");
        meta.duration = if meta.duration > 0.0 { meta.duration } else { 0.0 };
    }

    let now = now_ms();
    let job = Job {
        id: id.clone(),
        status: JobStatus::Ready,
        progress: 0,
        stage: "Siap diproses".to_string(),
        error: None,
        file_name,
        size,
        meta,
        output_name: None,
        created_at: now,
        updated_at: now,
    };
    state.upsert_job(job.clone()).await;
    Ok(Json(job))
}

pub async fn logo(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    let id = query_id(&query)?;
    let dir = state.upload_dir(&id);
    let temp = dir.join("logo.tmp");
    let (file_name, _) = save_uploaded_field(multipart, &temp).await?;
    let ext = extension_from_name(&file_name);
    let path = dir.join(format!("logo.{ext}"));
    tokio::fs::rename(&temp, &path)
        .await
        .map_err(|err| ApiError::internal(format!("Gagal menyimpan logo: {err}")))?;

    Ok(Json(json!({ "path": path.to_string_lossy() })))
}

pub async fn process(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProcessRequest>,
) -> Result<Json<Value>, ApiError> {
    let id = normalize_id(&payload.id)?;
    if state.get_job(&id).await.is_none() {
        return Err(ApiError::not_found("Job tidak ditemukan"));
    }

    state
        .update_job(&id, |job| {
            job.status = JobStatus::Processing;
            job.progress = 0;
            job.stage = "Menyiapkan…".to_string();
            job.error = None;
            job.output_name = None;
        })
        .await;

    let task_state = state.clone();
    let task_id = id.clone();
    let settings = payload.settings;
    tokio::spawn(async move {
        ffmpeg::process_job(task_state, task_id, settings).await;
    });

    Ok(Json(json!({ "ok": true, "id": id })))
}

pub async fn cancel(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let id = query_id(&query)?;
    if state.get_job(&id).await.is_none() {
        return Err(ApiError::not_found("Job tidak ditemukan"));
    }

    let killed = ffmpeg::cancel_job(&state, &id).await;
    state
        .update_job(&id, |job| {
            if job.status == JobStatus::Processing {
                job.status = JobStatus::Error;
                job.error = Some(ffmpeg::CANCELLED_MESSAGE.to_string());
                job.stage = "Dibatalkan".to_string();
            }
        })
        .await;

    Ok(Json(json!({ "ok": true, "killed": killed, "id": id })))
}

pub async fn job(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Job>, ApiError> {
    let id = query_id(&query)?;
    state
        .get_job(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found("Job tidak ditemukan"))
}

pub async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let (ffmpeg_ok, version) = ffmpeg::check_ffmpeg(&state.ffmpeg_path).await;
    let (ai_ok, ai_path, ai_binary) = ai::check_binary(&state);
    Json(json!({
        "ffmpeg": { "ok": ffmpeg_ok, "version": version },
        "ai": { "ok": ai_ok, "path": ai_path, "binary": ai_binary },
        "encoders": state.encoders.as_ref(),
        "maxOutputPixels": state.max_output_pixels,
        "ffmpegThreads": state.ffmpeg_threads,
        "dataDir": state.data_dir.to_string_lossy()
    }))
}

pub async fn ai_models() -> Json<Value> {
    Json(json!(ai::models()))
}

enum RangeSpec {
    Suffix(u64),
    Interval(u64, u64),
}

fn parse_range(value: &str) -> Option<RangeSpec> {
    let value = value.strip_prefix("bytes=")?.trim();
    let (start_text, end_text) = value.split_once('-')?;
    if start_text.trim().is_empty() {
        let suffix: u64 = end_text.trim().parse().ok()?;
        Some(RangeSpec::Suffix(suffix))
    } else {
        let start: u64 = start_text.trim().parse().ok()?;
        let end = if end_text.trim().is_empty() {
            u64::MAX
        } else {
            end_text.trim().parse().ok()?
        };
        Some(RangeSpec::Interval(start, end))
    }
}

fn range_not_satisfiable(total: u64, content_type: &str, disposition: Option<&str>) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_RANGE, format!("bytes */{total}"))
        .header(header::CONTENT_LENGTH, "0");
    if let Some(value) = disposition {
        builder = builder.header(header::CONTENT_DISPOSITION, value);
    }
    builder.body(Body::empty()).unwrap()
}

/// Stream file dengan dukungan HTTP Range (200 / 206 / 416).
pub async fn serve_range(
    path: &Path,
    headers: &HeaderMap,
    content_type: &str,
    disposition: Option<&str>,
) -> Response {
    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "File tidak ditemukan" })),
            )
                .into_response()
        }
    };
    let total = metadata.len();

    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_range);

    let (status, start, end) = match range {
        None => (StatusCode::OK, 0u64, total.saturating_sub(1)),
        Some(RangeSpec::Suffix(suffix)) => {
            if suffix == 0 || total == 0 {
                return range_not_satisfiable(total, content_type, disposition);
            }
            let start = total.saturating_sub(suffix.min(total));
            (StatusCode::PARTIAL_CONTENT, start, total - 1)
        }
        Some(RangeSpec::Interval(requested_start, requested_end)) => {
            if total == 0 || requested_start >= total || requested_start > requested_end {
                return range_not_satisfiable(total, content_type, disposition);
            }
            (
                StatusCode::PARTIAL_CONTENT,
                requested_start,
                requested_end.min(total - 1),
            )
        }
    };

    let length = if total == 0 { 0 } else { end - start + 1 };
    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "File tidak ditemukan" })),
            )
                .into_response()
        }
    };

    if start > 0 {
        if let Err(err) = file.seek(SeekFrom::Start(start)).await {
            warn!(%err, "gagal seek file");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    let stream = ReaderStream::new(file.take(length));

    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::CONTENT_LENGTH, length.to_string());
    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        );
    }
    if let Some(value) = disposition {
        builder = builder.header(header::CONTENT_DISPOSITION, value);
    }
    builder.body(Body::from_stream(stream)).unwrap()
}

pub async fn source(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let id = match query_id(&query) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let input = match ffmpeg::find_named(&state.upload_dir(&id), "input.") {
        Some(path) => path,
        None => return ApiError::not_found("Video sumber tidak ditemukan").into_response(),
    };
    let mime = state
        .get_job(&id)
        .await
        .map(|job| job.meta.mime)
        .filter(|mime| !mime.is_empty())
        .unwrap_or_else(|| "video/mp4".to_string());
    serve_range(&input, &headers, &mime, None).await
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let id = match query_id(&query) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    // ?inline=1 dipakai frontend sebagai sumber <video> preview; tanpa itu tetap attachment.
    let inline = query.get("inline").is_some_and(|value| {
        let value = value.trim().to_ascii_lowercase();
        value.is_empty() || matches!(value.as_str(), "1" | "true" | "yes")
    });
    let job = state.get_job(&id).await;
    let preferred = job
        .as_ref()
        .and_then(|job| job.output_name.as_deref())
        .filter(|name| !name.is_empty() && !name.contains('/') && !name.contains('\\'))
        .map(|name| state.output_dir(&id).join(name))
        .filter(|path| path.is_file());

    // Kalau job pernah diproses ke beberapa format, utamakan nama output terakhir.
    let output = match preferred
        .or_else(|| ffmpeg::find_named(&state.output_dir(&id), "enhanced."))
    {
        Some(path) => path,
        None => return ApiError::not_found("File hasil tidak ditemukan").into_response(),
    };
    let name = job
        .and_then(|job| job.output_name)
        .unwrap_or_else(|| {
            output
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| "enhanced.mp4".to_string())
        });
    let disposition = if inline {
        None
    } else {
        let safe_name = name.replace('"', "").replace(['\r', '\n'], "");
        Some(format!("attachment; filename=\"{safe_name}\""))
    };
    let mime = mime_guess::from_path(&output)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    serve_range(&output, &headers, &mime, disposition.as_deref()).await
}

pub async fn thumbnail(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let id = match query_id(&query) {
        Ok(id) => id,
        Err(err) => return err.into_response(),
    };
    let path: PathBuf = state.upload_dir(&id).join("poster.jpg");
    serve_range(&path, &headers, "image/jpeg", None).await
}
