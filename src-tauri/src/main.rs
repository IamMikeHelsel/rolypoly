// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusty::gui::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            create_archive,
            extract_archive,
            list_archive,
            validate_archive,
            get_archive_stats,
            calculate_file_hash,
            get_app_info,
            health_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}