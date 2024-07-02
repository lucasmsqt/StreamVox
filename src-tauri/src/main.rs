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
fn get_audio_devices() -> Result<(Vec<String>, Vec<String>), String> {
    println!("Fetching audio devices...");
    match audio_capture::list_audio_devices() {
        Ok(devices) => {
            println!("Audio devices fetched: {:?}", devices);
            Ok(devices)
        },
        Err(e) => {
            println!("Error fetching audio devices: {}", e);
            Err(e.to_string())
        },
    }
}

#[command]
fn start_audio_capture(inputDevice: String, outputDevice: String) -> Result<String, String> {
    println!("Starting audio capture with input device: {} and output device: {}", inputDevice, outputDevice);
    std::thread::spawn(move || {
        match audio_capture::capture_and_play_audio(&inputDevice, &outputDevice) {
            Ok(_) => println!("Audio capture started successfully."),
            Err(e) => eprintln!("Error capturing audio: {}", e),
        }
    });
    Ok("Audio capture started".into())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_audio_devices, start_audio_capture])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
