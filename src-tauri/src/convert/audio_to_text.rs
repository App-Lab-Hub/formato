// src-tauri/src/convert/audio_to_text.rs

use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperInferenceParams};
use transcribe_rs::audio::read_wav_samples;
use transcribe_rs::transcriber::{VadChunked, VadChunkedConfig, Transcriber};
use transcribe_rs::vad::EnergyVad;
use transcribe_rs::TranscribeOptions;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse, stringify};
use crate::convert::audio::convert_audio_to_audio;
use std::path::{Path, PathBuf};
use ffmpeg_sidecar::command::FfmpegCommand;
use crate::convert::is_file_cached;
use sea_orm::DatabaseConnection;

pub async fn convert_audio_to_text(
    db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // Если входной файл не WAV - конвертируем в WAV через convert_audio_to_audio
    let audio_path = if from != "wav" {
        let wav_path = convert_audio_to_audio(path, from, "wav")?;
        wav_path
    } else {
        path.to_string()
    };
    
    let model_path = get_or_download_model()?;
    
    let mut engine = WhisperEngine::load(&model_path)
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
    
    let temp_path = get_safe_temp_wav_path();
    convert_to_16khz_wav(&audio_path, &temp_path)?;
    
    let samples = read_wav_samples(&temp_path)
        .map_err(|e| format!("Failed to load WAV: {}", e))?;
    
    let _ = std::fs::remove_file(&temp_path);

    // println!("REMOVE ===={}===========",!is_file_cached(db, path, from, "wav").await?);


    // Если создавали WAV через convert_audio_to_audio - проверяем кеш перед удалением
    if !is_file_cached(db, path, from, "wav").await? {
        println!("REMOVE ===={}===========",audio_path);
            let _ = std::fs::remove_file(&audio_path);
        }
    
    // Создаем VAD с настройками для 16kHz
    // frame_size = 512 samples = 32ms при 16kHz
    let vad = EnergyVad::new(512, 0.01);
    
    let config = VadChunkedConfig {
        min_chunk_secs: 2.0,        // минимальный чанк 2 сек
        max_chunk_secs: 30.0,       // максимальный чанк 30 сек
        padding_secs: 0.5,          // 0.5 сек контекста с каждой стороны
        smart_split_search_secs: Some(3.0), // искать тишину за 3 сек до максимума
        merge_separator: " ".to_string(),   // разделитель между чанками
    };
    
    // Создаем опции транскрипции
    let transcribe_options = TranscribeOptions {
        language: None,
        translate: false,
        ..Default::default()
    };
    
    // Создаем VAD чанкер
    let mut transcriber = VadChunked::new(Box::new(vad), config, transcribe_options);
    
    // Транскрибируем через VAD
    let result = transcriber.transcribe(&mut engine, &samples)
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    let full_text = result.text;
    
    // Обрабатываем результат
    if to == "txt" || to == "text" {
        let hash = calculate_conversion_hash(path, from, to)
            .map_err(|e| format!("Hash error: {}", e))?;
        let output_path = get_app_dir_path_with_hash(path, to, &hash,true)?;
        
        if let Some(parent) = Path::new(&output_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create output directory: {}", e))?;
            }
        }
        
        std::fs::write(&output_path, &full_text)
            .map_err(|e| format!("Cannot write output file: {}", e))?;
        
        return Ok(output_path);
    }
    
    // Для других форматов - парсим текст в JSON и конвертируем
    let parsed = parse(&full_text, "txt")?;
    let output_path = stringify(&parsed, to, path, from)?;
    
    Ok(output_path)
}
/// Конвертирует в 16kHz моно WAV для Whisper с очисткой и улучшением речи
fn convert_to_16khz_wav(input_path: &str, output_path: &Path) -> Result<(), String> {
    let output_str = output_path.to_str().ok_or("Invalid temp path")?;
    // let filter_string = "highpass=f=200,afftdn,dynaudnorm";

    let mut cmd = FfmpegCommand::new();
    cmd.input(input_path);
    // cmd.args(&["-af", filter_string]);
    cmd.args(&["-ar", "16000"]);
    cmd.args(&["-ac", "1"]);
    cmd.args(&["-c:a", "pcm_s16le"]);
    cmd.args(&["-y"]);
    cmd.output(output_str);
    
    let mut child = cmd.spawn().map_err(|e| format!("FFmpeg spawn failed: {}", e))?;
    let status = child.wait().map_err(|e| format!("FFmpeg wait failed: {}", e))?;
    
    if !status.success() {
        return Err(format!("FFmpeg conversion failed with status: {}", status));
    }
    
    if !output_path.exists() {
        return Err("FFmpeg didn't create output file".to_string());
    }
    
    Ok(())
}

/// Генерирует уникальный путь в системной папке /tmp
fn get_safe_temp_wav_path() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
        
    std::env::temp_dir().join(format!("whisper_input_{}.wav", timestamp))
}

fn get_or_download_model() -> Result<std::path::PathBuf, String> {
    let model_dir = crate::paths::app_root().join("models/whisper");
    if !model_dir.exists() {
        std::fs::create_dir_all(&model_dir).map_err(|e| format!("Cannot create model dir: {}", e))?;
    }
    
    let model_name = "ggml-tiny-q5_1.bin";
    // let model_name = "ggml-base-q5_1.bin";

    let model_path = model_dir.join(model_name);
    if model_path.exists() {
        return Ok(model_path);
    }
    
    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", model_name);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client.get(&url).send().map_err(|e| format!("Download request failed: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    let bytes = response.bytes().map_err(|e| format!("Failed to read download bytes: {}", e))?;
    let mut file = std::fs::File::create(&model_path).map_err(|e| format!("Failed to create model file: {}", e))?;
    std::io::Write::write_all(&mut file, &bytes).map_err(|e| format!("Failed to write model data: {}", e))?;
    
    Ok(model_path)
}