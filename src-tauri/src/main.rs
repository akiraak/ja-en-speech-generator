// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod utils;

#[tokio::main]
async fn main() {
    utils::file::ensure_directory_exists();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::submit_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
