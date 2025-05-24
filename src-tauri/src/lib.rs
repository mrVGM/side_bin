use fs_mon::{file_tag::{get_tag, tag_file}, trackers::{self, get_tracker_state, tick}};
use serde_json::json;
use tauri::{AppHandle, Manager};

mod fs_mon {
    pub mod fs_mon;
    pub mod file_tag;
    pub mod trackers;
}

#[tauri::command]
fn monitor_command(action: &str, file: &str) -> String {
    println!("Command {} {}", action, file);
    match action {
        "tick" => {
            tick();
        }
        "register" => {
            let file_id = fs_mon::trackers::register_file(file);
            let file_id = json!({
                "id": file_id
            });
            return file_id.to_string();
        }
        "unregister" => {
            fs_mon::trackers::unregister_file(file);
            return json!({
                "unregistered": file
            }).to_string();
        }
        "update" => {
            let state = get_tracker_state(file);
            match state {
                Some(state) => {
                    let state_json = serde_json::to_string(&state).unwrap();
                    return state_json;
                }
                None => {
                    return "{}".to_string();
                }
            }
        }
        _ => {}
    }
    "{}".into()
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn resize_win(app: AppHandle, x: i32, y: i32, w: u32, h: u32) {
    let window = app.get_webview_window("main").unwrap();

    let pos = tauri::Position::Physical(tauri::PhysicalPosition {
        x,
        y
    });
    let s = tauri::Size::Physical(tauri::PhysicalSize{
        width: w,
        height: h
    });

    window.set_position(pos).unwrap();
    window.set_size(s).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_drag::init())
    .invoke_handler(tauri::generate_handler![resize_win, monitor_command])
    .run(tauri::generate_context!())
    .unwrap();
}

