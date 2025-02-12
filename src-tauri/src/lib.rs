use std::sync::Mutex;

use tauri::{Builder, Manager};

mod parse_xml;

enum AppState {
    Started,
    ParsedXml(parse_xml::Harmony),
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            parse_xml::parse_xml,
            parse_xml::get_info,
        ])
        .setup(|app| {
            app.manage(Mutex::new(AppState::Started));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
