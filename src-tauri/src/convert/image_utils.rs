use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use exif::Reader;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use image::GenericImageView;
use serde_json::{json, Value as Json};
use std::fs;
use std::io::Write;

/// Открывает изображение с автодетектом формата (поддерживает PNM/PAM)
pub fn open_image(path: &str, from: &str) -> Result<image::DynamicImage, String> {
    let reader = image::ImageReader::open(path).map_err(|e| format!("Cannot open image: {}", e))?;

    if from == "pnm" || path.to_lowercase().ends_with(".pnm") {
        if let Ok(reader_with_format) = reader.with_guessed_format() {
            if let Ok(img) = reader_with_format.decode() {
                return Ok(img);
            }
        }

        match image::ImageReader::open(path) {
            Ok(r) => match r.decode() {
                Ok(img) => Ok(img),
                Err(e) => Err(format!("Cannot decode PNM/PAM file: {}", e)),
            },
            Err(e) => Err(format!("Cannot open PNM file: {}", e)),
        }
    } else {
        reader
            .decode()
            .map_err(|e| format!("Cannot decode image: {}", e))
    }
}

/// Получение метаданных изображения
pub fn get_image_metadata(path: &str, img: &image::DynamicImage) -> Result<Json, String> {
    let dimensions = img.dimensions();
    let color_type = format!("{:?}", img.color());

    let mut metadata = json!({
        "width": dimensions.0,
        "height": dimensions.1,
        "color_type": color_type,
        "file_size": get_file_size(path)?,
    });

    if let Ok(exif) = get_exif_data(path) {
        metadata["exif"] = exif;
    }

    Ok(metadata)
}

/// Получение EXIF данных
pub fn get_exif_data(path: &str) -> Result<Json, String> {
    let file = fs::File::open(path).map_err(|e| format!("Cannot open file for EXIF: {}", e))?;

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = Reader::new();

    let exif = exifreader
        .read_from_container(&mut bufreader)
        .map_err(|e| format!("Cannot read EXIF: {}", e))?;

    let mut exif_map = serde_json::Map::new();

    for field in exif.fields() {
        let tag_name = format!("{:?}", field.tag);
        let value_str = match &field.value {
            exif::Value::Ascii(v) => {
                if let Some(first) = v.first() {
                    String::from_utf8_lossy(first).to_string()
                } else {
                    String::new()
                }
            }
            exif::Value::Byte(v) => format!("{:?}", v),
            exif::Value::Short(v) => format!("{:?}", v),
            exif::Value::Long(v) => format!("{:?}", v),
            exif::Value::Rational(v) => format!("{:?}", v),
            exif::Value::SRational(v) => format!("{:?}", v),
            exif::Value::Undefined(v, _) => format!("{:?}", v),
            exif::Value::SByte(v) => format!("{:?}", v),
            exif::Value::SShort(v) => format!("{:?}", v),
            exif::Value::SLong(v) => format!("{:?}", v),
            exif::Value::Float(v) => format!("{:?}", v),
            exif::Value::Double(v) => format!("{:?}", v),
            _ => format!("{:?}", field.value),
        };
        exif_map.insert(tag_name, Json::String(value_str));
    }

    Ok(Json::Object(exif_map))
}

/// Сжатие Zlib + кодирование Base64
pub fn zlib_and_then_base64(path: &str) -> Result<String, String> {
    // 1. Читаем файл в байты
    let bytes = fs::read(path).map_err(|e| format!("Cannot read file: {}", e))?;

    // 2. Сжимаем байты (zlib)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&bytes)
        .map_err(|e| format!("Cannot compress: {}", e))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("Cannot finish compression: {}", e))?;

    // 3. Кодируем сжатые байты в Base64
    let encoded = BASE64_STANDARD.encode(&compressed);

    Ok(encoded)
}

/// Получение размера файла
pub fn get_file_size(path: &str) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Cannot get file metadata: {}", e))?;
    Ok(metadata.len())
}
