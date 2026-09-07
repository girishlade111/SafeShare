#[tauri::command]
fn engine_info() -> pii_engine::EngineInfo {
    pii_engine::engine_info()
}

/// Scan `text` for PII and return every match with its offsets.
///
/// Offsets are character indices, so the frontend can slice the original
/// string with `text.slice(startIndex, endIndex)` directly.
#[tauri::command]
fn detect_pii(text: String) -> Vec<pii_engine::PiiMatch> {
    pii_engine::detect(&text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![engine_info, detect_pii])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}