// src-tauri/src/convert/audio_to_text.rs

use crate::convert::audio::convert_audio_to_audio;
use crate::convert::{calculate_conversion_hash, get_app_dir_path_with_hash, parse, stringify};
use crate::paths::whisper_models_dir;
use crate::settings::get_settings;
use ffmpeg_sidecar::command::FfmpegCommand;
use sea_orm::DatabaseConnection;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use transcribe_rs::accel::GPU_DEVICE_AUTO;
use transcribe_rs::audio::read_wav_samples;
use transcribe_rs::transcriber::{Transcriber, VadChunked, VadChunkedConfig};
use transcribe_rs::vad::EnergyVad;
use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperLoadParams};
use transcribe_rs::TranscribeOptions;

/// Перехват stderr (отключает вывод whisper.cpp)
struct StderrSilencer {
    saved_fd: std::os::unix::io::RawFd,
    devnull_fd: std::os::unix::io::RawFd,
}

#[allow(clippy::manual_c_str_literals)]
impl StderrSilencer {
    fn new() -> Self {
        unsafe {
            let saved_fd = libc::dup(2);
            let devnull_fd = libc::open(b"/dev/null\0".as_ptr() as *const i8, libc::O_WRONLY);
            libc::dup2(devnull_fd, 2);
            Self {
                saved_fd,
                devnull_fd,
            }
        }
    }
}

impl Drop for StderrSilencer {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.saved_fd, 2);
            libc::close(self.saved_fd);
            libc::close(self.devnull_fd);
        }
    }
}

/// Определяет язык текста (ru или en)
fn detect_language_from_text(text: &str) -> String {
    let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();
    if total_chars == 0 {
        return "en".to_string();
    }

    let cyrillic_count = text
        .chars()
        .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
        .count();

    let cyrillic_percent = cyrillic_count as f32 / total_chars as f32 * 100.0;

    if cyrillic_percent >= 51.0 {
        "ru".to_string()
    } else {
        "en".to_string()
    }
}

/// Пробует транскрибировать маленький кусок для определения языка
fn detect_language_from_audio(
    engine: &mut WhisperEngine,
    samples: &[f32],
) -> Result<String, String> {
    let test_len = (16000 * 3).min(samples.len());
    if test_len == 0 {
        return Ok("en".to_string());
    }
    let test_samples = &samples[..test_len];

    let vad = EnergyVad::new(512, 0.03);
    let config = VadChunkedConfig {
        min_chunk_secs: 1.0,
        max_chunk_secs: 3.0,
        padding_secs: 0.0,
        smart_split_search_secs: None,
        merge_separator: " ".to_string(),
    };

    let transcribe_options = TranscribeOptions {
        language: None,
        translate: false,
        ..Default::default()
    };

    let mut transcriber = VadChunked::new(Box::new(vad), config, transcribe_options);

    match transcriber.transcribe(engine, test_samples) {
        Ok(res) => {
            let text = res.text;
            if text.trim().is_empty() {
                Ok("en".to_string())
            } else {
                Ok(detect_language_from_text(&text))
            }
        }
        Err(e) => {
            println!("⚠️ Language detection failed, defaulting to English: {}", e);
            Ok("en".to_string())
        }
    }
}

/// Очищает текст от невалидных UTF-8 символов и бинарных токенов
fn clean_transcription_text(text: &str) -> String {
    text.chars()
        .filter(|c| {
            if c.is_whitespace() {
                return true;
            }
            if c.is_ascii_punctuation() || c.is_ascii_graphic() {
                return true;
            }
            if c.is_alphabetic() {
                return true;
            }
            false
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub async fn convert_audio_to_text(
    _db: &DatabaseConnection,
    path: &str,
    from: &str,
    to: &str,
) -> Result<String, String> {
    // 🔥 Отключаем логи whisper.cpp
    let _silencer = StderrSilencer::new();

    // 1. Асинхронные операции
    let settings = get_settings().await;
    let model_name = settings.recognition_model;
    let model_path = get_model_path(&model_name)?;

    let audio_path = convert_audio_to_audio(path, from, "wav")?;

    // 2. Генерируем путь для ресэмплинга
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let temp_path = std::env::temp_dir().join(format!("whisper_input_{}.wav", timestamp));

    convert_to_16khz_wav(&audio_path, &temp_path)?;

    // 3. Читаем сэмплы
    let samples = match read_wav_samples(&temp_path) {
        Ok(samples) => {
            if samples.is_empty() {
                let _ = std::fs::remove_file(&temp_path);
                return Err("Audio file is empty or contains no valid audio samples.".to_string());
            }
            samples
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            let err_msg = format!("Failed to load WAV: {}", e);
            if err_msg.contains("memory") || err_msg.contains("alloc") {
                return Err(
                    "Not enough memory to load audio samples. Please close other applications."
                        .to_string(),
                );
            }
            return Err(err_msg);
        }
    };

    let _ = std::fs::remove_file(&temp_path);

    // 4. Загружаем модель Whisper
    let params = WhisperLoadParams {
        use_gpu: true,
        flash_attn: true,
        gpu_device: GPU_DEVICE_AUTO,
    };

    let engine_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        WhisperEngine::load_with_params(&model_path, params)
    }));

    let mut engine = match engine_result {
        Ok(Ok(engine)) => engine,
        Ok(Err(e)) => {
            return Err(format!("Failed to load Whisper model: {}", e));
        }
        Err(_) => {
            return Err(
                "Fatal error while loading Whisper model (possible memory corruption)".to_string(),
            );
        }
    };

    // 5. Определяем язык из аудио
    let detected_lang = detect_language_from_audio(&mut engine, &samples)?;
    println!("🌐 Определён язык: {}", detected_lang);

    // 6. Настройка VAD с повышенным порогом
    let vad = EnergyVad::new(512, 0.03);
    let config = VadChunkedConfig {
        min_chunk_secs: 2.0,
        max_chunk_secs: 30.0,
        padding_secs: 0.5,
        smart_split_search_secs: Some(3.0),
        merge_separator: " ".to_string(),
    };

    let transcribe_options = TranscribeOptions {
        language: Some(detected_lang),
        translate: false,
        ..Default::default()
    };

    let mut transcriber = VadChunked::new(Box::new(vad), config, transcribe_options);

    // 7. Основная транскрипция
    let result_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        transcriber.transcribe(&mut engine, &samples)
    }));

    let full_text = match result_result {
        Ok(Ok(res)) => {
            let cleaned = clean_transcription_text(&res.text);
            if cleaned.is_empty() {
                return Err("No speech detected in audio".to_string());
            }
            cleaned
        }
        Ok(Err(e)) => {
            let err_msg = format!("{}", e);
            if err_msg.contains("UTF-8") || err_msg.contains("inference") {
                println!(
                    "⚠️ UTF-8/Inference crash detected. Running fallback with 'en' language..."
                );

                let fallback_config = VadChunkedConfig {
                    min_chunk_secs: 2.0,
                    max_chunk_secs: 30.0,
                    padding_secs: 0.2,
                    smart_split_search_secs: None,
                    merge_separator: " ".to_string(),
                };

                // Пробуем русский
                let ru_options = TranscribeOptions {
                    language: Some("ru".to_string()),
                    translate: false,
                    ..Default::default()
                };

                let mut ru_transcriber = VadChunked::new(
                    Box::new(EnergyVad::new(512, 0.05)),
                    fallback_config,
                    ru_options,
                );

                if let Ok(ru_res) = ru_transcriber.transcribe(&mut engine, &samples) {
                    let cleaned = clean_transcription_text(&ru_res.text);
                    if !cleaned.is_empty() {
                        return Ok(cleaned);
                    }
                }

                // Если русский не помог — пробуем английский
                let en_fallback_config = VadChunkedConfig {
                    min_chunk_secs: 2.0,
                    max_chunk_secs: 30.0,
                    padding_secs: 0.2,
                    smart_split_search_secs: None,
                    merge_separator: " ".to_string(),
                };

                let en_options = TranscribeOptions {
                    language: Some("en".to_string()),
                    translate: false,
                    ..Default::default()
                };

                let mut en_transcriber = VadChunked::new(
                    Box::new(EnergyVad::new(512, 0.05)),
                    en_fallback_config,
                    en_options,
                );

                match en_transcriber.transcribe(&mut engine, &samples) {
                    Ok(en_res) => {
                        let cleaned = clean_transcription_text(&en_res.text);
                        if cleaned.is_empty() {
                            return Err("No speech detected in audio".to_string());
                        }
                        cleaned
                    }
                    Err(fb_err) => {
                        return Err(format!("Fallback also failed: {}", fb_err));
                    }
                }
            } else {
                return Err(format!("Transcription failed: {}", err_msg));
            }
        }
        Err(_) => {
            return Err("Whisper engine panicked critically during processing.".to_string());
        }
    };

    // 8. Сохраняем результат
    if to == "txt" || to == "text" {
        let hash =
            calculate_conversion_hash(path, from, to).map_err(|e| format!("Hash error: {}", e))?;
        let output_path = get_app_dir_path_with_hash(path, to, &hash)?;

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
    let output_path = stringify(&parsed, to, path, from).await?;

    Ok(output_path)
}

/// Конвертирует в 16kHz моно WAV для Whisper
fn convert_to_16khz_wav(input_path: &str, output_path: &Path) -> Result<(), String> {
    let output_str = output_path.to_str().ok_or("Invalid temp path")?;

    let mut cmd = FfmpegCommand::new();
    cmd.input(input_path);

    cmd.arg("-ar").arg("16000");
    cmd.arg("-ac").arg("1");
    cmd.arg("-c:a").arg("pcm_s16le");
    cmd.arg("-y");
    cmd.output(output_str);

    let ffmpeg_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;
        let status = child
            .wait()
            .map_err(|e| format!("FFmpeg wait failed: {}", e))?;
        Ok::<_, String>(status)
    }));

    let status = match ffmpeg_result {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("FFmpeg process panicked.".to_string()),
    };

    if !status.success() {
        return Err(format!("FFmpeg failed with status: {}", status));
    }

    if !output_path.exists() {
        return Err("FFmpeg output file not found.".to_string());
    }

    Ok(())
}

/// Получить путь к модели Whisper
fn get_model_path(model_name: &str) -> Result<std::path::PathBuf, String> {
    let model_dir = whisper_models_dir();
    let model_path = model_dir.join(model_name);
    if !model_path.exists() {
        return Err(format!("Model file not found: {:?}", model_path));
    }
    Ok(model_path)
}
