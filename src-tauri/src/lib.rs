use std::{alloc::Layout, cell::LazyCell, sync::LazyLock, time::SystemTime};

use mac_address::get_mac_address;
use tauri::{AppHandle, Manager};
use uuid::{ClockSequence, Timestamp, Uuid};

mod fs_mon {
    pub mod fs_mon;
}

struct Clock(u128);

impl Clock {
    fn new() -> Self {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let time_stamp = duration_since_epoch.as_nanos();

        Clock(time_stamp)
    }
}

impl ClockSequence for Clock {
    type Output = u128;

    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output {
        self.0 + (1_000_000_000 * seconds as u128) + (subsec_nanos as u128)
    }
}

static CLOCK: LazyLock<Clock> = LazyLock::new(|| -> Clock {
    Clock::new()
});

#[tauri::command]
fn monitor_command(action: &str, file: &str) -> String {
    let mac_address = get_mac_address().unwrap().unwrap();
    let clock = &*CLOCK;
    
    let uuid = Uuid::new_v1(Timestamp::now(clock), &mac_address.bytes());
    let as_str = format!("{}", uuid);

    println!("Command {} {}", action, file);
    as_str
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

