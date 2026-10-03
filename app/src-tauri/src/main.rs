//! Desktop shell of WSim. Thin adapter between the simulation core and the UI:
//! it forwards commands and returns views, but contains no game logic.

// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wsim_core::CoreInfo;

#[tauri::command]
fn kern_info() -> CoreInfo {
    wsim_core::core_info()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![kern_info])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von WSim");
}
