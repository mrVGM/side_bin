use std::io::{BufWriter, Cursor};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::SystemTime;
use std::{env, panic};

use fs_mon::{file_tag::get_tag, trackers::{get_tracker_state, tick}};
use image::ImageEncoder;
use serde_json::json;
use tauri::{AppHandle, Manager};

mod fs_mon {
    pub mod fs_mon;
    pub mod file_tag;
    pub mod trackers;
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn open_file_directory(file: String) {
    let path = {
        let path = std::path::PathBuf::from_str(&file);
        match path {
            Ok(path) => {
                path
            }
            Err(_) => {
                return;
            }
        }
    };
    let _ = opener::reveal(path);
}

static CONFIG: Mutex<Option<(SystemTime, String)>> = Mutex::new(None);
fn read_config_internal() -> Result<String, ()> {
    let current_exe = std::env::current_exe()
        .map_err(|_| ())?;
    let config = current_exe
        .parent()
        .ok_or(())?
        .join("config.json");

    let meta = config.metadata()
        .map_err(|_| ())?;

    let modified = meta.modified()
        .map_err(|_| ())?;

    let config_obj = &mut *CONFIG.lock().unwrap();
    let dur = match config_obj {
        None => {
            None
        }
        Some((time_point, _)) => {
            let dur = modified.duration_since(*time_point);
            dur.ok()
        }
    };

    let should_update = match dur {
        None => {
            true
        }
        Some(dur) => {
            !dur.is_zero()
        }
    };

    if should_update {
        let config_content = std::fs::read_to_string(&config)
            .map_err(|_| ())?;
        *config_obj = Some((modified, config_content));
    }

    let content = match config_obj {
        None => "{}".to_owned(),
        Some((_, content)) => content.to_owned()
    };

    return Ok(content);
}

static STYLE: Mutex<Option<(SystemTime, String)>> = Mutex::new(None);
fn read_style_internal() -> Result<(bool, String), ()> {
    let current_exe = std::env::current_exe()
        .map_err(|_| ())?;
    let style = current_exe
        .parent()
        .ok_or(())?
        .join("style.css");

    let meta = style.metadata()
        .map_err(|_| ())?;

    let modified = meta.modified()
        .map_err(|_| ())?;

    let style_obj = &mut *STYLE.lock().unwrap();
    let dur = match style_obj {
        None => {
            None
        }
        Some((time_point, _)) => {
            let dur = modified.duration_since(*time_point);
            dur.ok()
        }
    };

    let should_update = match dur {
        None => {
            true
        }
        Some(dur) => {
            !dur.is_zero()
        }
    };

    if should_update {
        let style_content = std::fs::read_to_string(&style)
            .map_err(|_| ())?;
        *style_obj = Some((modified, style_content));
    }

    match style_obj {
        None => Err(()),
        Some((_, content)) => {
            Ok((should_update, content.to_owned()))
        }
    }
}

#[tauri::command]
fn read_config() -> String {
    let content = read_config_internal();
    match content {
        Ok(s) => s,
        Err(()) => "{}".into()
    }
}

#[tauri::command]
fn read_style() -> String {
    let content = read_style_internal();
    let json = match content {
        Ok((changed, content)) => {
            json!({
                "valid": true,
                "changed": changed,
                "content": content
            })
        },
        Err(()) => {
            json!({
                "valid": false
            })
        }
    };
    json.to_string()
}

#[tauri::command]
fn get_file_tag(file: &str) -> String {
    let tag = get_tag(file);
    let response = match tag {
        Some(tag) => {
            json!({
                "valid": true,
                "tag": tag
            })
        }
        None => {
            json!({
                "valid": false
            })
        }
    };

    response.to_string()
}

#[tauri::command]
fn monitor_command(action: &str, file: &str) -> String {
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

#[tauri::command]
fn get_win_pos(app: AppHandle) -> String {
    let window = app.get_webview_window("main").unwrap();
    if let Ok(pos) = window.outer_position() {
        return json!({
            "valid": true,
            "pos": [pos.x, pos.y]
        }).to_string();
    }
    json!({
        "valid": false
    }).to_string()
}


#[tauri::command]
fn get_file_icon(file: &str) -> String {
    let res = json!({
        "valid": false
    }).to_string();

    let icon = file_icon_provider::get_file_icon(file, 32);
    let icon = match icon {
        Ok(icon) => icon,
        Err(_) => {
            return res;
        }
    };

    let mut png_data = Vec::new();
    let writer = BufWriter::new(Cursor::new(&mut png_data));
    let encoder = image::codecs::png::PngEncoder::new(writer);
    let write_res = encoder.write_image(
        &icon.pixels,
        icon.width,
        icon.height,
        image::ExtendedColorType::Rgba8);

    if let Err(_) = write_res {
        return res;
    }

    json!({
        "valid": true,
        "data": png_data
    }).to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env::set_var("RUST_BACKTRACE", "1");
    panic::set_hook(Box::new(|_| {
        let backtrace = std::backtrace::Backtrace::capture();
        let info = format!("{}", backtrace.to_string());
        let _ = std::fs::write("crash_dump.dmp", info);
    }));

    let running_instance = fs_mon::fs_mon::get_running_instance();
    if running_instance > 0 {
        println!("App already running at PID: {}", running_instance);
        return;
    }

    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_drag::init())
    .invoke_handler(
        tauri::generate_handler![
            resize_win,
            get_win_pos,
            monitor_command,
            get_file_tag,
            read_config,
            read_style,
            get_file_icon,
            open_file_directory,
            exit_app
        ])
    .run(tauri::generate_context!())
    .unwrap();
}

