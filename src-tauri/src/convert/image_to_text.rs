use crate::convert::{
    image_utils::{get_image_metadata, open_image, zlib_and_then_base64},
    stringify,
};
use serde_json::json;
/// Конвертация изображения в текст (сжатый Base91 + метаданные)
pub async fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
    // 1. Читаем изображение с поддержкой PNM
    let img = open_image(path, from)?;

    // 2. Получаем метаданные
    let metadata = get_image_metadata(path, &img)?;

    // 3. Сжимаем и кодируем изображение в Base91
    let encoded_str = zlib_and_then_base64(path)?;

    // 4. Собираем всё в JSON
    let result = json!({
        "format": from,
        "encoding": "zlib+base64",
        "metadata": metadata,
        "data": encoded_str,
    });

    // 5. Используем stringify для сериализации и сохранения
    stringify(&result, to, path, from).await
}
