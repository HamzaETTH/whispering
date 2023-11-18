// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent, Manager, WindowEvent};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                if id == "quit" {
                    std::process::exit(0);
                }
            }
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            _ => {}
        })
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            let window_clone = window.clone(); // Clone the window handle

            window.on_window_event(move |event| {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close(); // Prevents closing the app
                        window_clone.hide().unwrap(); // Minimizes to tray using the cloned handle
                    }
                    _ => {}
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![convert_to_mp3, paste, set_tray_icon])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


use std::process::Command;

#[tauri::command]
fn convert_to_mp3(input: &str, output: &str) -> Result<String, String> {
    Command::new("lame")
        .arg(input)
        .arg(output)
        .status()
        .map_err(|e| format!("Failed to execute the lame command: {}", e))
        .map(|_| format!("Successfully converted {} to {}", input, output))
}

use enigo::{Enigo, Key, KeyboardControllable};

#[tauri::command]
fn paste() {
    let mut enigo = Enigo::new();
    if cfg!(target_os = "macos") {
        enigo.key_down(Key::Meta);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Meta);
    } else {
        enigo.key_down(Key::Control);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Control);
    }
}

#[tauri::command]
fn set_tray_icon(app: tauri::AppHandle, state: String) {
    let icon_bytes = match state.as_str() {
        "recording" => include_bytes!("../icons/128x128@2x_recording.png").to_vec(),
        "transcribing" => include_bytes!("../icons/128x128@2x_transcribing.png").to_vec(),
        _ => include_bytes!("../icons/128x128@2x.png").to_vec(),
    };
    app.tray_handle().set_icon(tauri::Icon::Raw(icon_bytes)).unwrap();
}