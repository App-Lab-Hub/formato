// use serde_json::json;
// use crate::convert::{
//     stringify,
//     image_utils::{
//         open_image,
//         get_image_metadata,
//         get_base64_data,
//     }
// };

// /// Конвертация изображения в текст (Base64 + метаданные)
// pub async fn convert_image_to_text(path: &str, from: &str, to: &str) -> Result<String, String> {
//     // 1. Читаем изображение с поддержкой PNM
//     let img = open_image(path, from)?;
    
//     // 2. Получаем метаданные
//     let metadata = get_image_metadata(path, &img)?;
    
//     // 3. Получаем Base64 представление
//     // let base64_data = get_base64_data(path)?;
    
//     // 4. Собираем всё в JSON
//     let result = json!({
//         "format": from,
//         "metadata": metadata,
//         // "base64": base64_data,
//     });

//     // 5. Используем stringify для сериализации и сохранения
//     stringify(&result, to, path, from).await
// }


use serde_json::json;
use crate::convert::{
    stringify,
    image_utils::{
        open_image,
        get_image_metadata,
        zlib_and_then_base64,
    }
};
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
        "encoding": "zlib+base64"
        "metadata": metadata,
        "data": encoded_str,
    });

    // 5. Используем stringify для сериализации и сохранения
    stringify(&result, to, path, from).await
}