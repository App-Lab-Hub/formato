use tokio::process::Command;
use tokio::sync::Semaphore;
use std::path::Path;
use tokio::fs;
use tempfile::tempdir;
use std::sync::LazyLock;
use uuid::Uuid;

// Глобальный семафор — только 1 вызов soffice одновременно
static SOFFICE_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

/// Конвертация через soffice с явным фильтром (асинхронная)
pub async fn convert_with_soffice_explicit(
    input_path: &str, 
    output_path: &str
) -> Result<(), String> {
    // Захватываем семафор
    let _permit = SOFFICE_SEMAPHORE.acquire().await.unwrap();
    
    let input_path_obj = Path::new(input_path);
    let output_path_obj = Path::new(output_path);

    let input_ext = input_path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    let output_ext = output_path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("pdf");

    // Выбираем фильтр
    let filter = match (input_ext, output_ext) {
        ("odt", "pdf") => "writer_pdf_Export",
        ("odt", "docx") => "Office Open XML Text",
        ("odt", "odt") => "writer8",
        ("docx", "pdf") => "writer_pdf_Export",
        ("docx", "odt") => "writer8",
        ("xlsx", "pdf") => "calc_pdf_Export",
        ("xlsx", "docx") => "MS Excel 2007 XML",
        ("xlsx", "odt") => "MS Excel 2007 XML",
        _ => match output_ext {
            "pdf" => "writer_pdf_Export",
            "docx" => "Office Open XML Text",
            "odt" => "writer8",
            _ => "writer8",
        }
    };

    let temp_dir = tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    
    // 🚀 Запускаем soffice с фильтром
    let status = Command::new("soffice")
        .env("JAVA_OPTS", "-Djava.awt.headless=true")
        .env("SAL_USE_VCLPLUGIN", "svp")
        .args([
            "--headless",
            "--nologo",
            "--norestore",
            "--nofirststartwizard",
            "--invisible",
            "--convert-to", &format!("{}:{}", output_ext, filter),
            "--outdir", &temp_dir_path,
            input_path,
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|e| format!("soffice error: {}", e))?;

    if !status.success() {
        // 🔄 Если не получилось, пробуем без фильтра
        return fallback_convert(input_path, output_path, output_ext).await;
    }

    let input_stem = input_path_obj
        .file_stem()
        .ok_or_else(|| "Invalid input file name".to_string())?
        .to_string_lossy()
        .to_string();
    
    let temp_output = Path::new(&temp_dir_path).join(format!("{}.{}", input_stem, output_ext));

    if temp_output.exists() {
        fs::rename(&temp_output, output_path_obj)
            .await
            .map_err(|e| format!("Failed to move to {}: {}", output_path, e))?;
        Ok(())
    } else {
        // 🔄 Если файл не создан, пробуем без фильтра
        fallback_convert(input_path, output_path, output_ext).await
    }
}

/// Резервный вариант конвертации без явного фильтра (асинхронный)
async fn fallback_convert(
    input_path: &str, 
    output_path: &str, 
    output_ext: &str
) -> Result<(), String> {
    let input_path_obj = Path::new(input_path);
    let output_path_obj = Path::new(output_path);

    let temp_dir = tempdir()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    
    let status = Command::new("soffice")
        .env("JAVA_OPTS", "-Djava.awt.headless=true")
        .env("SAL_USE_VCLPLUGIN", "svp")
        .args([
            "--headless",
            "--nologo",
            "--norestore",
            "--nofirststartwizard",
            "--invisible",
            "--convert-to", output_ext,
            "--outdir", &temp_dir_path,
            input_path,
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|e| format!("soffice error: {}", e))?;

    if !status.success() {
        return Err("soffice conversion failed".to_string());
    }

    let input_stem = input_path_obj
        .file_stem()
        .ok_or_else(|| "Invalid input file name".to_string())?
        .to_string_lossy()
        .to_string();
    
    let temp_output = Path::new(&temp_dir_path).join(format!("{}.{}", input_stem, output_ext));

    if temp_output.exists() {
        fs::rename(&temp_output, output_path_obj)
            .await
            .map_err(|e| format!("Failed to move to {}: {}", output_path, e))?;
        Ok(())
    } else {
        Err("soffice did not create output file".to_string())
    }
}

/// Конвертирует XML в HTML через soffice (LibreOffice) — асинхронная версия
pub async fn xml_to_html_via_soffice(xml_str: &str) -> Result<String, String> {
    // Захватываем семафор
    let _permit = SOFFICE_SEMAPHORE.acquire().await.unwrap();
    
    // Генерируем уникальные имена для каждого вызова
    let uuid = Uuid::new_v4().simple().to_string();
    let xml_path = std::env::temp_dir().join(format!("temp_{}.xml", uuid));
    let html_path = std::env::temp_dir().join(format!("temp_{}.html", uuid));
    
    // Сохраняем XML во временный файл
    fs::write(&xml_path, xml_str)
        .await
        .map_err(|e| format!("Cannot write XML: {}", e))?;
    
    let out_dir = std::env::temp_dir();
    let out_dir_str = out_dir.to_string_lossy().to_string();
    
    // Конвертируем через soffice
    let status = Command::new("soffice")
        .env("JAVA_OPTS", "-Djava.awt.headless=true")
        .env("SAL_USE_VCLPLUGIN", "svp")
        .args([
            "--headless",
            "--nologo",
            "--norestore",
            "--nofirststartwizard",
            "--invisible",
            "--convert-to", "html",
            "--outdir", &out_dir_str,
            xml_path.to_str().unwrap(),
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|e| format!("soffice error: {}", e))?;
    
    if !status.success() {
        let _ = fs::remove_file(&xml_path).await;
        return Err("soffice conversion failed".to_string());
    }
    
    // Читаем результат
    let html_content = fs::read_to_string(&html_path)
        .await
        .map_err(|e| format!("Cannot read HTML: {}", e))?;
    
    // Удаляем временные файлы
    let _ = fs::remove_file(&xml_path).await;
    let _ = fs::remove_file(&html_path).await;
    
    Ok(html_content)
}

/// Конвертирует DOCX в RTF через soffice (асинхронная версия)
pub async fn convert_docx_to_rtf(
    docx_path: &str, 
    original_path: &str, 
    to: &str
) -> Result<String, String> {
    // Захватываем семафор
    let _permit = SOFFICE_SEMAPHORE.acquire().await.unwrap();
    
    // Проверяем наличие soffice
    let check = Command::new("soffice")
        .arg("--version")
        .output()
        .await;
    
    if check.is_err() {
        return Err("soffice not found. Please install LibreOffice.".to_string());
    }

    // Получаем директорию DOCX
    let docx_dir = Path::new(docx_path)
        .parent()
        .ok_or("Invalid docx path")?
        .to_str()
        .ok_or("Invalid docx path")?
        .to_string();

    // Временный RTF с тем же именем
    let docx_stem = Path::new(docx_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid docx filename")?
        .to_string();
    
    let temp_rtf = format!("{}/{}.rtf", docx_dir, docx_stem);

    // Конвертируем через soffice
    let status = Command::new("soffice")
        .env("JAVA_OPTS", "-Djava.awt.headless=true")
        .env("SAL_USE_VCLPLUGIN", "svp")
        .args([
            "--headless",
            "--nologo",
            "--norestore",
            "--nofirststartwizard",
            "--invisible",
            "--convert-to", "rtf",
            "--outdir", &docx_dir,
            docx_path,
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|e| format!("soffice error: {}", e))?;

    if !status.success() {
        return Err("soffice conversion failed".to_string());
    }

    // Проверяем, что файл создан
    if !Path::new(&temp_rtf).exists() {
        return Err("soffice did not create RTF file".to_string());
    }

    // Перемещаем в нужную папку с хешем
    let hash = crate::convert::calculate_conversion_hash(original_path, "docx", to)
        .map_err(|e| format!("Hash error convert_docx_to_rtf: {}", e))?;
    
    let final_path = crate::convert::get_app_dir_path_with_hash(original_path, to, &hash, true)?;

    // Создаем директорию
    if let Some(parent) = Path::new(&final_path).parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Cannot create output dir: {}", e))?;
        }
    }

    // Перемещаем
    tokio::fs::rename(&temp_rtf, &final_path)
        .await
        .map_err(|e| format!("Cannot rename RTF file: {}", e))?;

    Ok(final_path)
}


/// Конвертирует XML в RTF через soffice (LibreOffice) — асинхронная версия
pub async fn xml_to_rtf_via_soffice(xml_str: &str) -> Result<String, String> {
    // Захватываем семафор
    let _permit = SOFFICE_SEMAPHORE.acquire().await.unwrap();
    
    // Генерируем уникальные имена для каждого вызова
    let uuid = Uuid::new_v4().simple().to_string();
    let xml_path = std::env::temp_dir().join(format!("temp_{}.xml", uuid));
    let rtf_path = std::env::temp_dir().join(format!("temp_{}.rtf", uuid));
    
    // Сохраняем XML во временный файл
    tokio::fs::write(&xml_path, xml_str)
        .await
        .map_err(|e| format!("Cannot write XML: {}", e))?;
    
    let out_dir = std::env::temp_dir();
    let out_dir_str = out_dir.to_string_lossy().to_string();
    
    // Конвертируем через soffice
    let status = Command::new("soffice")
        .env("JAVA_OPTS", "-Djava.awt.headless=true")
        .env("SAL_USE_VCLPLUGIN", "svp")
        .args([
            "--headless",
            "--nologo",
            "--norestore",
            "--nofirststartwizard",
            "--invisible",
            "--convert-to", "rtf",
            "--outdir", &out_dir_str,
            xml_path.to_str().unwrap(),
        ])
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|e| format!("soffice error: {}", e))?;
    
    if !status.success() {
        let _ = tokio::fs::remove_file(&xml_path).await;
        return Err("soffice conversion failed".to_string());
    }
    
    // Читаем результат
    let rtf_content = tokio::fs::read_to_string(&rtf_path)
        .await
        .map_err(|e| format!("Cannot read RTF: {}", e))?;
    
    // Удаляем временные файлы
    let _ = tokio::fs::remove_file(&xml_path).await;
    let _ = tokio::fs::remove_file(&rtf_path).await;
    
    Ok(rtf_content)
}