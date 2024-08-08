
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
use tauri::Manager;
fn main() {
    tauri::Builder::default()
      .setup(|app| {
        #[cfg(debug_assertions)]
        app.get_window("main").unwrap().open_devtools(); // `main` is the first window from tauri.conf.json without an explicit label
        Ok(())
      })
      .invoke_handler(tauri::generate_handler![greet])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
 }
