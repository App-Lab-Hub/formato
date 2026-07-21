
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;
/// Извлечение текста из PDF (использует pdf-extract)
pub fn extract_text_from_pdf(path: &str) -> Result<String, String> {
    // Pandoc умеет извлекать текст из PDF через фильтры
    // Но для чистого текста лучше использовать специализированные инструменты
    // Попробуем Pandoc с флагом --to=plain
    match run_pandoc_output(&[path, "-t", "plain"]) {
        Ok(text) => {
            // Если текст не пустой - возвращаем
            if !text.trim().is_empty() {
                return Ok(text);
            }
            // Если пусто - падаем на pdf-extract
            pdf_extract::extract_text(path)
                .map_err(|e| format!("PDF extract: {e}"))
        }
        Err(_) => {
            // Если Pandoc не справился - используем pdf-extract
            pdf_extract::extract_text(path)
                .map_err(|e| format!("PDF extract: {e}"))
        }
    }
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

/// Запуск WeasyPrint для генерации PDF из HTML
pub fn run_weasyprint(input: &str, output: &str) -> Result<(), String> {
    let status = Command::new("weasyprint")
        .args(&[input, output])
        .status()
        .map_err(|e| format!("WeasyPrint error: {e}"))?;
    if !status.success() {
        return Err("WeasyPrint завершился с ошибкой".to_string());
    }
    Ok(())
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









pub fn xlsx_to_html(path: &str) -> Result<String, String> {
    // Попробуем Pandoc для XLSX -> HTML
    // Pandoc может читать XLSX и конвертировать в HTML с таблицами
    match run_pandoc_output(&[path, "-t", "html", "--standalone"]) {
        Ok(html) => {
            // Проверяем, что есть таблицы
            if html.contains("<table") && !html.trim().is_empty() {
                return Ok(html);
            }
            // Если Pandoc не дал таблиц - используем calamine
            xlsx_to_html_calamine(path)
        }
        Err(_) => {
            // Fallback на calamine
            xlsx_to_html_calamine(path)
        }
    }
}
fn xlsx_to_html_calamine(path: &str) -> Result<String, String> {
    use calamine::{Reader, Xlsx, Data};
    
    let mut workbook: Xlsx<_> = calamine::open_workbook(path)
        .map_err(|e| format!("XLSX open: {e}"))?;
    
    let mut html = String::from("<!DOCTYPE html><html><body>");
    
    for sheet_name in workbook.sheet_names() {
        let range = workbook.worksheet_range(&sheet_name)
            .map_err(|e| format!("Sheet read: {e}"))?;
        
        html.push_str(&format!("<h2>{}</h2><table border='1'>", sheet_name));
        
        for row in range.rows() {
            html.push_str("<tr>");
            for cell in row {
                let value = match cell {
                    Data::String(s) => s.clone(),
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
                html.push_str(&format!("<td>{}</td>", value));
            }
            html.push_str("</tr>");
        }
        html.push_str("</table>");
    }
    
    html.push_str("</body></html>");
    Ok(html)
}





pub fn generate_pdf(input_path: &str, output_path: &str) -> Result<(), String> {
    let status = Command::new("pandoc")
        .args(&[
            input_path,
            "-t", "pdf",
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