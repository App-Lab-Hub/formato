// src-tauri/src/highlight.rs
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use tauri::Emitter;

lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

#[tauri::command]
pub async fn highlight_code(code: String, lang: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let syntax = SYNTAX_SET
            .find_syntax_by_token(&lang)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let theme = &THEME_SET.themes["base16-eighties.dark"];
        highlighted_html_for_string(&code, &SYNTAX_SET, syntax, theme)
            .map_err(|e| format!("Highlight error: {e}"))
    })
    .await
    .map_err(|e| format!("Join error: {e}"))?
}

#[tauri::command]
pub async fn highlight_code_stream(
    app: tauri::AppHandle,
    code: String,
    lang: String,
    chunk_size: usize,
) -> Result<(), String> {

    tokio::task::spawn_blocking(move || {
        let lines: Vec<&str> = code.lines().collect();
        let total = lines.len().div_ceil(chunk_size);
        let syntax = SYNTAX_SET
            .find_syntax_by_token(&lang)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        let theme = &THEME_SET.themes["base16-eighties.dark"];
        
        for (i, chunk) in lines.chunks(chunk_size).enumerate() {
            let code_chunk = chunk.join("\n");
            let html = highlighted_html_for_string(&code_chunk, &SYNTAX_SET, syntax, theme)
                .unwrap_or_else(|_| format!("<pre>{}</pre>", code_chunk));
            
            let _ = app.emit("highlight-chunk", serde_json::json!({
                "index": i,
                "total": total,
                "html": html,
            }));
            
            if i >= 3 {
                std::thread::sleep(std::time::Duration::from_micros(500));
            }
        }
        
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Join error: {e}"))??;
    
    Ok(())
}