
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;


use rust_xlsxwriter::*;
/// Извлечение текста из PDF (использует pdf-extract)
pub fn extract_text_from_pdf(path: &str) -> Result<String, String> {
    // ИСПРАВЛЕНО: Убран вызов Pandoc, так как он не умеет читать PDF файлы.
    // Сразу используем специализированную и быструю библиотеку pdf-extract
    pdf_extract::extract_text(path)
        .map_err(|e| format!("PDF extract error: {e}"))
}


/// Запуск Pandoc и возврат stdout (используется для получения HTML)
pub fn run_pandoc_output(args: &[&str]) -> Result<String, String> {
    let output = Command::new("pandoc")
        .args(args)
        .output()
        .map_err(|e| format!("Pandoc error: {e}"))?;
    if !output.status.success() {
        return Err(format!("Pandoc stderr: {}", String::from_utf8_lossy(&output.stderr)));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {e}"))
}





/// Запуск Pandoc с передачей аргументов (без возврата вывода)
pub fn run_pandoc(args: &[&str]) -> Result<(), String> {
    let status = Command::new("pandoc")
        .args(args)
        .status()
        .map_err(|e| format!("Pandoc error: {e}"))?;
    if !status.success() {
        return Err("Pandoc завершился с ошибкой".to_string());
    }
    Ok(())
}










pub fn generate_pdf(input_path: &str, output_path: &str) -> Result<(), String> {
    let status = Command::new("pandoc")
        .args([
            input_path,
            "-t", "html", // ИСПРАВЛЕНО: Pandoc транслирует структуру в HTML, а WeasyPrint красит её в PDF
            "--pdf-engine=weasyprint",
            "-o", output_path,
        ])
        .status()
        .map_err(|e| format!("Pandoc/WeasyPrint error: {e}"))?;

    if !status.success() {
        return Err("Ошибка при генерации PDF через Pandoc/WeasyPrint".to_string());
    }
    Ok(())
}






/// Создание документа из текста (улучшенная версия с Pandoc)
/// Создание документа из текста (улучшенная версия)
pub fn create_document_from_text(text: &str, format: &str, output_path: &str) -> Result<(), String> {
    // ИСПРАВЛЕНО: Для Pandoc мы передаем ЧИСТЫЙ текст (или минимальный фрагмент <pre>),
    // БЕЗ тегов <!DOCTYPE html> и <html>, чтобы они не выводились вверху страниц документа.
    // Мы используем тег <pre> для сохранения переносов строк (whitespace-преформатирование).
    let html_fragment = format!("<pre style='white-space: pre-wrap;'>{}</pre>", text);

    match format {
        "docx" => {
            let temp_html = write_temp_file(&html_fragment)?;
            run_pandoc(&[&temp_html, "-t", "docx", "-o", output_path])?;
            Ok(())
        }
        "odt" => {
            let temp_html = write_temp_file(&html_fragment)?;
            run_pandoc(&[&temp_html, "-t", "odt", "-o", output_path])?;
            Ok(())
        }
        "xlsx" => {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            
            for (row_idx, line) in text.lines().enumerate() {
                let words: Vec<&str> = line.split_whitespace().collect();
                for (col_idx, word) in words.iter().enumerate() {
                    worksheet.write_string(row_idx as u32, col_idx as u16, *word)
                        .map_err(|e| format!("XLSX write: {}", e))?;
                }
            }
            workbook.save(output_path).map_err(|e| format!("Save XLSX: {}", e))?;
            Ok(())
        }
        "pdf" => {
            // Для PDF WeasyPrint требует полноценный html-документ со стилями, 
            // но чтобы он не отображался текстом, Pandoc должен знать, что это html-вход
            let full_html = format!(
                "<!DOCTYPE html><html><head><meta charset='utf-8'></head><body><pre style='white-space: pre-wrap;'>{}</pre></body></html>", 
                text
            );
            let temp_html = write_temp_file(&full_html)?;
            generate_pdf(&temp_html, output_path)?;
            Ok(())
        }
        _ => Err(format!("Unsupported target format: {}", format)),
    }
}















/// Конвертация XLSX в HTML с fallback на calamine
pub fn xlsx_to_html(path: &str) -> Result<String, String> {
    // 1. Проверяем, существует ли файл вообще
    if !std::path::Path::new(path).exists() {
        return Err(format!("Файл не найден: {}", path));
    }

    // 2. Попытка вызвать внешний инструмент/Pandoc 
    // УБРАН флаг --standalone, чтобы Pandoc возвращал только фрагмент таблицы без <!DOCTYPE html>
    match run_pandoc_output(&[path, "-t", "html"]) {
        Ok(html) => {
            // Проверяем, что на выходе действительно валидный HTML с таблицей
            if html.contains("<table") && !html.trim().is_empty() {
                return Ok(html);
            }
            xlsx_to_html_calamine(path)
        }
        Err(_) => {
            xlsx_to_html_calamine(path)
        }
    }
}

/// Конвертация XLSX в HTML через calamine (резервный вариант)
fn xlsx_to_html_calamine(path: &str) -> Result<String, String> {
    use calamine::{Reader, Xlsx, Data};

    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| format!("XLSX open error: {e}"))?;
    
    // ИСПРАВЛЕНО: Начинаем сразу с пустой строки, без тегов документа
    let mut html = String::new();
    
    // Получаем список всех листов
    let sheet_names = workbook.sheet_names().to_vec();
    
    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name)
            .map_err(|e| format!("Sheet read error on '{}': {e}", sheet_name))?;
        
        // Создаем заголовок листа и открываем таблицу
        html.push_str(&format!("<h2>{}</h2>\n<table border='1' style='border-collapse: collapse; margin-bottom: 20px;'>\n", sheet_name));
        
        for row in range.rows() {
            html.push_str("  <tr>\n");
            for cell in row {
                let value = match cell {
                    Data::String(s) => s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"),
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => {
                        if f.fract() == 0.0 && f.is_finite() {
                            (f.trunc() as i64).to_string()
                        } else {
                            f.to_string()
                        }
                    },
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("Error: {:?}", e),
                    Data::Empty => String::new(),
                };
                html.push_str(&format!("    <td>{}</td>\n", value));
            }
            html.push_str("  </tr>\n");
        }
        html.push_str("</table>\n");
    }
    
    // ИСПРАВЛЕНО: Убраны закрывающие теги </body></html>
    Ok(html)
}


/// Запись строки во временный файл, возврат его пути
pub fn write_temp_file(content: &str) -> Result<String, String> {
    let mut file = NamedTempFile::new().map_err(|e| format!("Temp file: {e}"))?;
    file.write_all(content.as_bytes()).map_err(|e| format!("Write temp: {e}"))?;
    let path = file.path().to_str().ok_or("Invalid temp path")?.to_string();
    // Сохраняем файл, чтобы он не удалился при выходе из области видимости
    file.keep().map_err(|e| format!("Keep temp: {e}"))?;
    Ok(path)
}




// pub fn convert_with_libreoffice(input_path: &str, from: &str, to: &str) -> Result<Vec<u8>, String> {
//     let input_bytes = std::fs::read(input_path)
//         .map_err(|e| format!("Read {}: {}", from, e))?;
    
//     let output_bytes = convert_bytes(&input_bytes, from, to)
//         .map_err(|e| format!("Convert {} to {}: {}", from, to, e))?;
    
//     Ok(output_bytes)
// }






use std::path::Path;
use std::fs;

pub fn convert_with_soffice_explicit(input_path: &str, output_path: &str) -> Result<(), String> {
    let input_path_obj = Path::new(input_path);
    let output_path_obj = Path::new(output_path);

    // 1. Извлекаем расширение из финального пути (например, "pdf")
    let ext = output_path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("pdf");

    // 2. Создаем изолированную временную папку (защита от конфликтов имен)
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    // 3. Конвертируем документ во временную папку
    let status = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to", ext,
            "--outdir", &temp_dir.path().to_string_lossy(),
            input_path,
        ])
        .status()
        .map_err(|e| format!("soffice error: {}", e))?;

    if !status.success() {
        return Err("soffice conversion failed".to_string());
    }

    // 4. Находим файл, который создал soffice (исходное имя + новое расширение)
    let input_stem = input_path_obj
        .file_stem()
        .ok_or_else(|| "Invalid input file name".to_string())?;
    
    let temp_output = temp_dir.path().join(format!("{}.{}", input_stem.to_string_lossy(), ext));

    // 5. Всегда переименовываем и перемещаем файл в целевой output_path
    if temp_output.exists() {
        // fs::rename автоматически перезапишет старый файл по пути output_path, если он существовал
        fs::rename(&temp_output, output_path_obj)
            .map_err(|e| format!("Failed to move and rename to {}: {}", output_path, e))?;
    } else {
        return Err("soffice did not create output file in temp dir".to_string());
    }

    Ok(())
}