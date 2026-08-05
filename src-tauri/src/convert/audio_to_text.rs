use transcribe_rs::whisper_cpp::{WhisperEngine};
use transcribe_rs::audio::read_wav_samples;
use transcribe_rs::transcriber::{VadChunked, VadChunkedConfig, Transcriber};
use transcribe_rs::vad::EnergyVad;
use transcribe_rs::TranscribeOptions;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse, stringify};
use crate::convert::audio::convert_audio_to_audio;
use std::path::{Path, PathBuf};
use ffmpeg_sidecar::command::FfmpegCommand;
use sea_orm::DatabaseConnection;
use crate::settings::get_settings;
use crate::paths::{whisper_models_dir};
use std::time::{SystemTime, UNIX_EPOCH}; // Для генерации уникального имени

pub async fn convert_audio_to_text(
    _db: &DatabaseConnection,
    path: &str, 
    from: &str, 
    to: &str
) -> Result<String, String> {
    // Если входной файл не WAV - конвертируем в WAV через convert_audio_to_audio
    let audio_path = convert_audio_to_audio(path, from, "wav")?;

    // ✅ Получаем выбранную модель из настроек
    let settings = get_settings().await;
    let model_name = settings.recognition_model;
    let model_path = get_model_path(&model_name)?;
    
    let mut engine = WhisperEngine::load(&model_path)
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
    
    // 🛠 Генерируем уникальный путь в системной временной папке
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let temp_path = std::env::temp_dir().join(format!("whisper_input_{}.wav", timestamp));
    
    // Передаем путь во FFmpeg. Файла еще нет на диске, FFmpeg создаст его сам
    convert_to_16khz_wav(&audio_path, &temp_path)?;
    
    // ✅ Читаем сэмплы в оперативную память
    let samples = read_wav_samples(&temp_path)
        .map_err(|e| format!("Failed to load WAV: {}", e))?;
    
    // ✅ Вручную удаляем временный файл сразу после чтения
    let _ = std::fs::remove_file(&temp_path);
    
    // Создаем VAD с настройками для 16kHz
    let vad = EnergyVad::new(512, 0.01);
    
    let config = VadChunkedConfig {
        min_chunk_secs: 2.0,
        max_chunk_secs: 30.0,
        padding_secs: 0.5,
        smart_split_search_secs: Some(3.0),
        merge_separator: " ".to_string(),
    };
    
    let transcribe_options = TranscribeOptions {
        language: None,
        translate: false,
        ..Default::default()
    };
    
    let mut transcriber = VadChunked::new(Box::new(vad), config, transcribe_options);
    
    let result = transcriber.transcribe(&mut engine, &samples)
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    let full_text = result.text;
    
    // Обрабатываем результат
    if to == "txt" || to == "text" {
        let hash = calculate_conversion_hash(path, from, to)
            .map_err(|e| format!("Hash error convert_audio_to_text: {}", e))?;
        let output_path = get_app_dir_path_with_hash(path, to, &hash, true)?;
        
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
    
    let parsed = parse(&full_text, "txt")?;
    let output_path = stringify(&parsed, to, path, from)?;
    
    Ok(output_path)
}

/// Конвертирует в 16kHz моно WAV для Whisper с очисткой и улучшением речи
fn convert_to_16khz_wav(input_path: &str, output_path: &Path) -> Result<(), String> {
    let output_str = output_path.to_str().ok_or("Invalid temp path")?;

    let mut cmd = FfmpegCommand::new();
    cmd.input(input_path);
    
    // Атомарные аргументы для стабильности FFmpeg
    cmd.arg("-ar").arg("16000");
    cmd.arg("-ac").arg("1");
    cmd.arg("-c:a").arg("pcm_s16le");
    cmd.arg("-y"); 
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

/// Получить путь к модели Whisper (без скачивания)
fn get_model_path(model_name: &str) -> Result<std::path::PathBuf, String> {
    let model_dir = whisper_models_dir();
    let model_path = model_dir.join(model_name);
    if !model_path.exists() {
        return Err(format!("Model file not found: {:?}", model_path));
    }
    Ok(model_path)
}
