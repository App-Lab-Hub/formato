mod convert;
mod html_convert;
use tauri::Manager;

#[tauri::command]
fn app_ready(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_background_color(Some(tauri::utils::config::Color(6, 6, 8, 255)));
        let _ = window.set_theme(Some(tauri::Theme::Dark));
        let _ = window.show();
    }
}

#[tauri::command]
fn set_window_background(app: tauri::AppHandle, r: u8, g: u8, b: u8, a: u8) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_background_color(Some(tauri::utils::config::Color(r, g, b, a)));
    }
}

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
            app_ready,
            set_window_background,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}