use std::path::PathBuf;

use parse_xml::{Harmony, XmlInfo};
use process::{DownloadInfo, ImageFilter, OutputInfo};
use tauri::{async_runtime::Mutex, Builder, Manager, State};

mod parse_xml;
mod process;

#[derive(Default)]
struct AppState {
    info: Option<Harmony>,
    filter: Option<ImageFilter>,
    output: Option<OutputInfo>,
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

#[tauri::command]
async fn set_output(
    dir: PathBuf,
    action: String,
    format: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.output = Some(OutputInfo {
        dir,
        action,
        format,
    });

    Ok(())
}

#[tauri::command]
async fn get_dl_info(state: State<'_, Mutex<AppState>>) -> Result<DownloadInfo, String> {
    let state = state.lock().await;

    DownloadInfo::try_from(&*state).map_err(|err| format!("{:?}", err))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_info,
            get_dl_info,
            set_filter,
            set_output,
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
