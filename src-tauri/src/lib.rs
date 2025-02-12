use parse_xml::{Harmony, XmlInfo};
use process::ImageFilter;
use tauri::{async_runtime::Mutex, Builder, Manager, State};

mod parse_xml;
mod process;

#[derive(Default)]
struct AppState {
    info: Option<Harmony>,
    filter: Option<ImageFilter>
}

#[tauri::command]
async fn get_info(state: State<'_, Mutex<AppState>>) -> Result<XmlInfo, String> {
    let state = state.lock().await;

    match state.info {
        Some(ref h) => Ok(XmlInfo::from(h)),
        None => Err("App has not yet generated Harmony Information".into()),
    }
}

#[tauri::command]
async fn set_filter(filter: ImageFilter, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.filter = Some(filter);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_info,
            set_filter,
            parse_xml::parse_xml,
            process::test_download,
        ])
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
