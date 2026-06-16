mod convert;
mod highlight;
mod html_convert;

use std::sync::Mutex;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Manager, State};
use tokio::time::{sleep, Duration};

struct SetupState {
    frontend_task: bool,
    backend_task: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(SetupState {
            frontend_task: false,
            backend_task: false,
        }))
        .invoke_handler(tauri::generate_handler![
            convert::convert_file,
            convert::read_file_content,
            convert::open_file,
            highlight::highlight_code,
            highlight::highlight_code_stream,
            set_complete,
        ])
        .setup(|app| {
            let splash = app.get_webview_window("splashscreen")
                .expect("splashscreen window not found");
            
            // Плавно показываем заставку после отрисовки окна
            splash.eval("document.body.classList.add('fade-in-done')").ok();
            
            spawn(setup(app.handle().clone()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn set_complete(
    app: AppHandle,
    state: State<'_, Mutex<SetupState>>,
    task: String,
) -> Result<(), ()> {
    let both_done = {
        let mut state_lock = state.lock().unwrap();
        match task.as_str() {
            "frontend" => state_lock.frontend_task = true,
            "backend" => state_lock.backend_task = true,
            _ => panic!("invalid task completed!"),
        }
        state_lock.backend_task && state_lock.frontend_task
    };

    if both_done {
        let splash = app.get_webview_window("splashscreen").unwrap();
        let main = app.get_webview_window("main").unwrap();
        
        // Запускаем анимацию исчезновения
        splash.eval("document.body.classList.add('fade-out')").ok();
        sleep(Duration::from_millis(1500)).await;
        
        // Сначала закрываем заставку, потом показываем главное окно
        splash.close().unwrap();
        main.show().unwrap();
    }
    Ok(())
}

async fn setup(app: AppHandle) {
    // Ждём пока анимации проиграются + 1 секунда чтобы полюбоваться
    sleep(Duration::from_millis(8000)).await;
    
    set_complete(
        app.clone(),
        app.state::<Mutex<SetupState>>(),
        "backend".to_string(),
    ).await.ok();
}