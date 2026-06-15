mod convert;
mod highlight;
mod html_convert;

use tauri::Manager;
use std::time::Duration;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            convert::convert_file,
            convert::read_file_content,
            convert::open_file,
            highlight::highlight_code,
            highlight::highlight_code_stream,
        ])
        .setup(|app| {
            let splash = app.get_webview_window("splashscreen")
                .expect("splashscreen window not found");
            let main = app.get_webview_window("main")
                .expect("main window not found");

            tauri::async_runtime::spawn(async move {
                // Ждём минимум пока анимация дойдёт до конца (3.5s)
                tokio::time::sleep(Duration::from_millis(3500)).await;

                // Показываем главное окно ПОД заставкой (невидимо)
                main.show().unwrap();
                
                // Даём кадр на отрисовку главного окна
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Запускаем плавное исчезновение заставки
                splash.eval("document.body.classList.add('fade-out')").unwrap();
                
                // Ждём окончания CSS-анимации (600ms)
                tokio::time::sleep(Duration::from_millis(650)).await;
                
                // Закрываем заставку
                splash.close().unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}