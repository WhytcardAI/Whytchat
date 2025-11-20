use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<f32>,
}

#[derive(Serialize, Clone)]
pub struct DownloadFinished {
    pub path: String,
    pub sha256: String,
    pub verified: bool,
    pub expected: Option<String>,
}

pub fn app_data_models_dir(app: &tauri::AppHandle) -> Result<PathBuf> {
    let base = app
        .path()
        .app_local_data_dir()
        .context("app_local_data_dir not available")?;
    let dir = base.join("models");
    std::fs::create_dir_all(&dir).with_context(|| format!("create dir {:?}", dir))?;
    Ok(dir)
}

pub fn app_llama_server_dir(app: &tauri::AppHandle) -> Result<PathBuf> {
    let base = app
        .path()
        .app_local_data_dir()
        .context("app_local_data_dir not available")?;
    let dir = base.join("llama-server");
    std::fs::create_dir_all(&dir).with_context(|| format!("create dir {:?}", dir))?;
    Ok(dir)
}

pub fn find_llama_server(app: &tauri::AppHandle) -> Option<PathBuf> {
    // Check in app data dir first
    if let Ok(dir) = app_llama_server_dir(app) {
        let exe = dir.join("llama-server.exe");
        if exe.exists() {
            return Some(exe);
        }
    }
    None
}

/// Downloads the default Qwen2.5-3B-Instruct model file and emits progress/finished events.
/// Returns the absolute path to the downloaded file.
#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    expected_sha256: Option<String>,
) -> Result<String, String> {
    let url = "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf?download=true";
    let dir = app_data_models_dir(&app).map_err(|e| e.to_string())?;
    let dst = dir.join("qwen2.5-3b-instruct-q4_k_m.gguf");
    if dst.exists() {
        return Ok(dst.to_string_lossy().into_owned());
    }

    let client = reqwest::Client::builder()
        .user_agent("WhytChat/0.1 (tauri)")
        .build()
        .map_err(|e| e.to_string())?;

    let mut resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;

    let total = resp.content_length();
    let mut file = fs::File::create(&dst).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(100) {
            let percent = total.map(|t| (downloaded as f32 / t as f32) * 100.0);
            let _ = app.emit(
                "download/progress",
                DownloadProgress {
                    downloaded,
                    total,
                    percent,
                },
            );
            last_emit = Instant::now();
        }
    }
    // Final progress emit
    let percent = total.map(|_| 100.0);
    let _ = app.emit(
        "download/progress",
        DownloadProgress {
            downloaded,
            total,
            percent,
        },
    );

    let hash_bytes = hasher.finalize();
    let sha256 = format!("{:x}", hash_bytes);
    let verified = expected_sha256
        .as_ref()
        .map(|exp| exp.eq_ignore_ascii_case(&sha256))
        .unwrap_or(false);
    let path = dst.to_string_lossy().into_owned();
    let _ = app.emit(
        "download/finished",
        DownloadFinished {
            path: path.clone(),
            sha256: sha256.clone(),
            verified,
            expected: expected_sha256.clone(),
        },
    );
    Ok(path)
}

fn server_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app_llama_server_dir(app).map_err(|e| e.to_string())
}

#[derive(Serialize, Clone)]
pub struct InstallFinished {
    pub exe_path: String,
    pub zip_sha256: String,
}

/// Install llama-server from official ggml-org/llama.cpp Releases (Windows CPU by default).
/// backend: "cpu" (default) or specific like "cuda-12.4" to match asset suffix.
#[tauri::command]
pub async fn install_server(
    app: tauri::AppHandle,
    backend: Option<String>,
) -> Result<String, String> {
    let suffix = match backend.as_deref() {
        Some(b) if b.starts_with("cuda") => format!("win-{}-x64.zip", b),
        _ => "win-cpu-x64.zip".to_string(),
    };

    let dest_dir = server_dir(&app)?;
    let exe_path = dest_dir.join("llama-server.exe");
    if exe_path.exists() {
        return Ok(exe_path.to_string_lossy().into_owned());
    }

    let api = "https://api.github.com/repos/ggml-org/llama.cpp/releases/latest";
    let client = reqwest::Client::builder()
        .user_agent("WhytChat/0.1 (tauri)")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(api)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let assets = json["assets"]
        .as_array()
        .ok_or_else(|| "unexpected releases payload".to_owned())?;

    let mut asset_url: Option<String> = None;
    let mut asset_name: Option<String> = None;
    for a in assets {
        let name = a["name"].as_str().unwrap_or("");
        if name.ends_with(&suffix) && name.contains("-bin-") {
            asset_url = a["browser_download_url"].as_str().map(|s| s.to_string());
            asset_name = Some(name.to_string());
            break;
        }
    }
    let asset_url =
        asset_url.ok_or_else(|| format!("no release asset found for suffix {}", suffix))?;
    let asset_name = asset_name.unwrap_or_else(|| "llama-bin.zip".to_string());

    // Download ZIP
    let mut resp = client
        .get(&asset_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let total = resp.content_length();

    let tmp_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let zip_path = tmp_dir.join(&asset_name);
    let mut file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();

    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() >= Duration::from_millis(100) {
            let percent = total.map(|t| (downloaded as f32 / t as f32) * 100.0);
            let _ = app.emit(
                "server/install/progress",
                DownloadProgress {
                    downloaded,
                    total,
                    percent,
                },
            );
            last_emit = Instant::now();
        }
    }
    // Final progress emit
    let percent = total.map(|_| 100.0);
    let _ = app.emit(
        "server/install/progress",
        DownloadProgress {
            downloaded,
            total,
            percent,
        },
    );

    let zip_sha = format!("{:x}", hasher.finalize());

    // Extract all files
    let dest_dir = server_dir(&app)?;
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let zip_file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
    let mut found = false;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().replace('\\', "/");
        let base = name.split('/').next_back().unwrap_or("");
        if base.is_empty() {
            continue;
        }

        let mut out = fs::File::create(dest_dir.join(base)).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;

        if base.eq_ignore_ascii_case("llama-server.exe") {
            found = true;
        }
    }
    if !found {
        return Err("llama-server.exe not found in archive".into());
    }

    let exe_path = dest_dir.join("llama-server.exe");
    let exe_path_str = exe_path.to_string_lossy().into_owned();

    let _ = app.emit(
        "server/install/finished",
        InstallFinished {
            exe_path: exe_path_str.clone(),
            zip_sha256: zip_sha.clone(),
        },
    );
    Ok(exe_path_str)
}
