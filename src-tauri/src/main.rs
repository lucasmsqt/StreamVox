#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio_capture;

use tauri::command;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn get_audio_devices() -> Vec<String> {
    match audio_capture::list_audio_devices() {
        Ok(devices) => devices,
        Err(e) => {
            println!("Erro ao listar dispositivos: {}", e);
            vec!["Erro ao listar dispositivos".to_string()]
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_audio_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
