mod commands;
mod database;
mod timer;
mod tray;
mod windows;
use std::{fs, path::PathBuf, sync::Mutex, time::Duration};
use tauri::{Emitter, Manager};
pub struct AppState {
    database_path: PathBuf,
    timer: Mutex<timer::Timer>,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Some(runtime) = std::env::var_os("LITTLE_TOMATO_RUNTIME_DIR") {
        let webview = PathBuf::from(runtime).join("WebView2");
        fs::create_dir_all(&webview).expect("WebView2 directory");
        // Before WebView2 or any application worker threads start.
        unsafe {
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", webview);
        }
    }
    tauri::Builder::default()
        .setup(|app| {
            let runtime = match std::env::var_os("LITTLE_TOMATO_RUNTIME_DIR") {
                Some(p) => PathBuf::from(p),
                None => app.path().app_local_data_dir()?,
            };
            let database_path = runtime.join("data/little-tomato.sqlite3");
            database::initialize(&database_path)?;
            let timer = timer::Timer::load(&database_path)?;
            app.manage(AppState {
                database_path: database_path.clone(),
                timer: Mutex::new(timer),
            });
            app.manage(windows::Interaction::default());
            tray::setup(app)?;
            if let Some(window) = app.get_webview_window("main") {
                windows::setup(window, &database_path)?;
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let state = handle.state::<AppState>();
                    if let Ok(mut timer) = state.timer.lock() {
                        let result = timer.tick(&state.database_path);
                        let s = timer.snapshot.clone();
                        // The UI thread can be waiting for this mutex in a command.
                        drop(timer);
                        if let Err(error) = result {
                            let _ = handle.emit("app-error", error);
                        }
                        if let Some(tray) = handle.tray_by_id("main-tray") {
                            let phase = match s.phase {
                                timer::Phase::Focus => "专注",
                                timer::Phase::ShortBreak => "短休息",
                                timer::Phase::LongBreak => "长休息",
                            };
                            let _ = tray.set_tooltip(Some(format!(
                                "小番茄 · {phase} {:02}:{:02}",
                                s.remaining_seconds / 60,
                                s.remaining_seconds % 60
                            )));
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::runtime_status,
            commands::timer_snapshot,
            commands::timer_action,
            commands::quit_app,
            windows::set_hit_areas
        ])
        .run(tauri::generate_context!())
        .expect("error while running Little Tomato");
}
