#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::Read;
use std::path::PathBuf;

use serde::Serialize;
use tauri::Manager;

const BACKEND_PORT: &str = "19090";

const UPSCALER_SOURCE: &str = include_str!("../../scripts/fsrcnn_upscaler.py");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelFile {
    filename: String,
    url: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogModel {
    id: String,
    name: String,
    runtime: String,
    scale: u32,
    size_mb: u32,
    ram: String,
    speed: String,
    quality: String,
    pros: Vec<String>,
    cons: Vec<String>,
    note: String,
    recommended: bool,
    files: Vec<ModelFile>,
}

fn catalog() -> Vec<CatalogModel> {
    let fs = "https://raw.githubusercontent.com/Saafke/FSRCNN_Tensorflow/master/models/";
    let misc = "https://raw.githubusercontent.com/dparikh79/Image-Super-Resolution/main/Project%20Files/Networks/";

    vec![
        CatalogModel {
            id: "fsrcnn".into(), name: "FSRCNN".into(), runtime: "opencv-cpu".into(), scale: 4,
            size_mb: 1, ram: "~200 MB".into(), speed: "sedang".into(), quality: "sedang".into(),
            pros: vec!["Paling ringan, aman di RAM 1 GB".into(), "Sudah teruji".into()],
            cons: vec!["Detail masih modest".into()],
            note: "Default. Model 39 KB, CPU murni, tanpa Vulkan.".into(), recommended: true,
            files: vec![
                ModelFile { filename: "FSRCNN_x2.pb".into(), url: format!("{fs}FSRCNN_x2.pb") },
                ModelFile { filename: "FSRCNN_x3.pb".into(), url: format!("{fs}FSRCNN_x3.pb") },
                ModelFile { filename: "FSRCNN_x4.pb".into(), url: format!("{fs}FSRCNN_x4.pb") },
            ],
        },
        CatalogModel {
            id: "espcn".into(), name: "ESPCN".into(), runtime: "opencv-cpu".into(), scale: 4,
            size_mb: 1, ram: "~150 MB".into(), speed: "paling cepat".into(), quality: "rendah".into(),
            pros: vec!["Paling cepat di CPU".into(), "Model ~100 KB".into()],
            cons: vec!["Garis bisa licin".into(), "Kualitas di bawah FSRCNN".into()],
            note: "Cocok untuk video panjang yang ingin tetap ringan.".into(), recommended: false,
            files: vec![ModelFile { filename: "ESPCN_x4.pb".into(), url: format!("{misc}ESPCN_x4.pb") }],
        },
        CatalogModel {
            id: "lapsrn".into(), name: "LapSRN".into(), runtime: "opencv-cpu".into(), scale: 8,
            size_mb: 4, ram: "~400 MB".into(), speed: "sedang".into(), quality: "cukup baik".into(),
            pros: vec!["Lebih tajam dari FSRCNN".into(), "Bisa sampai 8x".into()],
            cons: vec!["Model 4 MB".into(), "Lebih boram RAM".into()],
            note: "Perlu unduh LapSRN_x8.pb.".into(), recommended: false,
            files: vec![ModelFile { filename: "LapSRN_x8.pb".into(), url: format!("{misc}LapSRN_x8.pb") }],
        },
        CatalogModel {
            id: "edsr".into(), name: "EDSR".into(), runtime: "opencv-cpu".into(), scale: 4,
            size_mb: 38, ram: "~1 GB".into(), speed: "lambat".into(), quality: "tinggi".into(),
            pros: vec!["Detail paling tajam di kelas CPU".into()],
            cons: vec!["Model 38 MB".into(), "Berat di device lemah".into()],
            note: "Perlu unduh EDSR_x4.pb.".into(), recommended: false,
            files: vec![ModelFile { filename: "EDSR_x4.pb".into(), url: format!("{misc}EDSR_x4.pb") }],
        },
        CatalogModel {
            id: "realesr-animevideov3".into(), name: "Real-ESRGAN Anime v3".into(),
            runtime: "vulkan".into(), scale: 4, size_mb: 1, ram: "~300 MB".into(),
            speed: "cepat di GPU".into(), quality: "baik (anime)".into(),
            pros: vec!["Model 1,2 MB".into(), "Bagus untuk animasi".into()],
            cons: vec!["Butuh Vulkan/GPU".into(), "Bisa wedge di Radeon Vega 8".into()],
            note: "Unduh binary realesrgan-ncnn-vulkan manual, lalu set REALESRGAN_BIN.".into(),
            recommended: false, files: vec![],
        },
        CatalogModel {
            id: "realesrgan-x4plus".into(), name: "Real-ESRGAN x4plus".into(),
            runtime: "vulkan".into(), scale: 4, size_mb: 32, ram: "~600 MB".into(),
            speed: "lambat".into(), quality: "tertinggi".into(),
            pros: vec!["Kualitas terbaik untuk video nyata".into()],
            cons: vec!["Model 32 MB".into(), "Butuh GPU kuat & Vulkan stabil".into()],
            note: "Unduh binary + model manual, lalu set REALESRGAN_BIN.".into(),
            recommended: false, files: vec![],
        },
    ]
}

fn models_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("models");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
fn model_catalog() -> Vec<CatalogModel> {
    catalog()
}

#[tauri::command]
fn installed_models(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let dir = models_dir(&app)?;
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().is_file() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

#[tauri::command]
fn download_model(app: tauri::AppHandle, filename: String, url: String) -> Result<String, String> {
    let dir = models_dir(&app)?;
    let target = dir.join(&filename);
    if target.exists() {
        return Ok(target.to_string_lossy().into_owned());
    }

    let response = ureq::get(&url).call().map_err(|e| e.to_string())?;
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() < 1024 {
        return Err("File terlalu kecil, kemungkinan bukan model".into());
    }

    let temp = dir.join(format!("{filename}.part"));
    fs::write(&temp, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&temp, &target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
fn remove_model(app: tauri::AppHandle, filename: String) -> Result<(), String> {
    let path = models_dir(&app)?.join(filename);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn model_folder(app: tauri::AppHandle) -> Result<String, String> {
    Ok(models_dir(&app)?.to_string_lossy().into_owned())
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let data = app.path().app_data_dir()?.join("vidboost");
    let models = data.join("models");
    let runtime = data.join("runtime");
    fs::create_dir_all(&models)?;
    fs::create_dir_all(&runtime)?;
    fs::create_dir_all(data.join("data"))?;

    let upscaler_py = runtime.join("upscaler.py");
    fs::write(&upscaler_py, UPSCALER_SOURCE)?;

    let wrapper = runtime.join("upscaler.sh");
    #[cfg(unix)]
    fs::write(
        &wrapper,
        format!("#!/bin/sh\nexec python3 \"{}\" \"$@\"\n", upscaler_py.display()),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&wrapper)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wrapper, perms)?;
    }

    let backend_dir = data.join("bin");
    fs::create_dir_all(&backend_dir)?;

    tauri::process::Command::new_sidecar("vidboost-server")?
        .env("PORT", BACKEND_PORT)
        .env("DATA_DIR", data.join("data"))
        .env("AI_MODEL_DIR", &models)
        .env("FSRCNN_MODEL_DIR", &models)
        .env("AI_UPSCALER_BIN", &wrapper)
        .env("FFMPEG_PATH", "ffmpeg")
        .env("FFPROBE_PATH", "ffprobe")
        .spawn()?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            model_catalog,
            installed_models,
            download_model,
            remove_model,
            model_folder,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan VidBoost");
}
