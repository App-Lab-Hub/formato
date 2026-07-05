
mod csv;
mod xml;
mod ini;
mod md;
use crate::AppState;
use serde::{ Serialize};
use std::{path::PathBuf};

use std::fs::File;
use std::io::Read;
use xxhash_rust::xxh3::Xxh3;

use serde_json::{Value as Json};
use crate::convert::csv::{parse_csv, stringify_csv};
use crate::convert::ini::{parse_ini, stringify_ini};
use crate::convert::md::{parse_markdown, stringify_markdown};
use crate::convert::xml::{parse_xml, stringify_xml};
use crate::db;
use crate::html_convert::{convert_to_html,parse_html};
use crate::paths::converted_dir;
use memmap2::Mmap;

#[derive(Debug, Serialize)]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub hash: Option<String>,
    pub extension: Option<String>,
    pub error: Option<String>,
}


pub fn convert(path: &str, from: &str, to: &str) -> Result<String, String> {
    let input = std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {e}"))?;
    let value = parse(&input, from)?;
    stringify(&value, to)
}

pub fn save_to_app_dir(content: &str, original_path: &str, to: &str) -> Result<String, String> {
    let input_path = PathBuf::from(original_path);
    let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("converted");
    
    let output_dir = converted_dir();
    let output_path = output_dir.join(format!("{}.{}", stem, to));
    std::fs::write(&output_path, content).map_err(|e| format!("Cannot write file: {e}"))?;
    
    Ok(output_path.to_string_lossy().to_string())
}


use std::path::Path;

#[tauri::command]
pub async fn convert_file(
    state: tauri::State<'_, AppState>,
    path: String,
    from: String,
    to: String,
    enable_cache: bool,
) -> Result<ConvertResult, String> {
    let input_hash = calculate_conversion_hash(&path, &from, &to)
        .map_err(|e| format!("Cannot read file: {e}"))?;
    
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    
    if enable_cache {
        if let Some(existing_path) = db::find_conversion(db, &input_hash).await {
            dbg!("✅ Cache HIT", &input_hash);
            let extension = Path::new(&existing_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_string());

            return Ok(ConvertResult {
                success: true,
                content: existing_path,
                hash: Some(input_hash),
                extension,
                error: None,
            });
        }
        dbg!("❌ Cache MISS, converting...", &input_hash);
    } else {
        dbg!("🔄 Cache DISABLED, direct conversion", &input_hash);
    }
    
    let (path_clone, from_clone, to_clone) = (path.clone(), from.clone(), to.clone());
    let output = tokio::task::spawn_blocking(move || {
        convert(&path_clone, &from_clone, &to_clone)
    }).await.map_err(|e| format!("Task join error: {e}"))??;
    
    let saved_path = save_to_app_dir(&output, &path, &to)?;
    let extension = Path::new(&saved_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_string());

    if enable_cache {
        db::save_conversion(db, &input_hash, &saved_path).await?;
    }

    Ok(ConvertResult {
        success: true,
        content: saved_path,
        hash: Some(input_hash),
        extension,
        error: None,
    })
}

fn calculate_conversion_hash(path: &str, from: &str, to: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut hasher = Xxh3::new();

    // Пробуем mmap
    match unsafe { Mmap::map(&file) } {
        Ok(mmap) => {
            hasher.update(&mmap);
        }
        Err(_) => {
            // Fallback на буфер
            let mut file = file;
            let mut buffer = [0; 65536];
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
        }
    }

    hasher.update(from.as_bytes());
    hasher.update(to.as_bytes());

    Ok(format!("{:x}", hasher.digest()))
}
#[tauri::command]
pub async fn hash_file(path: String) -> Result<String, String> {
    calculate_file_hash(&path).map_err(|e| format!("Cannot hash file: {e}"))
}

fn calculate_file_hash(path: &str) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut hasher = Xxh3::new();

    match unsafe { Mmap::map(&file) } {
        Ok(mmap) => {
            hasher.update(&mmap);
        }
        Err(_) => {
            let mut file = file;
            let mut buffer = [0; 65536];
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
        }
    }

    Ok(format!("{:x}", hasher.digest()))
}


// ============================================================
// ПАРСЕРЫ
// ============================================================

fn parse(input: &str, format: &str) -> Result<Json, String> {
    match format {
        "json" => serde_json::from_str(input).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::from_str(input).map_err(|e| format!("YAML: {e}")),
        "toml" => toml::from_str(input).map_err(|e| format!("TOML: {e}")),
        "xml" => parse_xml(input),
        "ini" => parse_ini(input),
        "md" => parse_markdown(input),
        "csv" => parse_csv(input),
        "html" => parse_html(input),
        _ => Err(format!("Unsupported: {format}")),
    }
}






// ============================================================
// СЕРИАЛИЗАТОРЫ
// ============================================================

fn stringify(value: &Json, format: &str) -> Result<String, String> {
    match format {
        "json" => serde_json::to_string_pretty(value).map_err(|e| format!("JSON: {e}")),
        "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| format!("YAML: {e}")),
        "toml" => {
            let value_for_toml = match value {
                Json::Array(arr) => {
                    let mut map = serde_json::Map::new();
                    map.insert("data".to_string(), Json::Array(arr.clone()));
                    Json::Object(map)
                }
                _ => value.clone(),
            };
            toml::to_string_pretty(&value_for_toml).map_err(|e| format!("TOML: {e}"))
        }
        "xml" => stringify_xml(value).map_err(|e| format!("XML: {e}")),
        "csv" => stringify_csv(value),
        "ini" => stringify_ini(value),
        "html" => Ok(convert_to_html(value)),
        "md" => stringify_markdown(value),
        _ => Err(format!("Unsupported: {format}")),
    }
}


#[tauri::command]
pub async fn read_file_content(path: String) -> Result<String, String> {
    tokio::fs::read_to_string(&path).await.map_err(|e| format!("Cannot read file: {e}"))
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    opener::open(&path).map_err(|e| format!("Cannot open file: {e}"))
}