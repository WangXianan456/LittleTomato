mod commands;
mod database;
mod settings;
mod system_events;
mod timer;
mod tray;
mod windows;
use std::{fs, path::PathBuf, sync::Mutex, time::Duration};
use tauri::{Emitter, Manager};
pub struct AppState {
    database_path: PathBuf,
    timer: Mutex<timer::Timer>,
    system: system_events::SystemState,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Keep the explicit development data directory when launched at login.
    let args: Vec<_> = std::env::args_os().collect();
    if let Some(index) = args.iter().position(|arg| arg == "--runtime-dir")
        && let Some(path) = args
            .get(index + 1)
            .filter(|path| std::path::Path::new(path).is_absolute())
    {
        unsafe {
            std::env::set_var("LITTLE_TOMATO_RUNTIME_DIR", path);
        }
    }
    if let Some(runtime) = std::env::var_os("LITTLE_TOMATO_RUNTIME_DIR") {
        let webview = PathBuf::from(runtime).join("WebView2");
        fs::create_dir_all(&webview).expect("WebView2 directory");
        // Before WebView2 or any application worker threads start.
        unsafe {
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", webview);
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            windows::show(app)
        }))
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
                system: system_events::SystemState::default(),
            });
            app.manage(windows::Interaction::default());
            tray::setup(app)?;
            if let Some(window) = app.get_webview_window("main") {
                system_events::install(&window)?;
                windows::setup(window, &database_path)?;
            }
            settings::apply(app.handle(), &settings::load(&database_path)?)?;
            if let Some(panel) = app.get_webview_window("settings") {
                let hide = panel.clone();
                panel.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide.hide();
                    }
                });
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let state = handle.state::<AppState>();
                    if let Ok(mut timer) = state.timer.lock() {
                        let result = if state.system.blocked() {
                            timer.interrupt(&state.database_path, "system")
                        } else {
                            timer.tick(&state.database_path)
                        };
                        let s = timer.snapshot.clone();
                        // The UI thread can be waiting for this mutex in a command.
                        drop(timer);
                        if let Err(error) = result {
                            let _ = handle.emit("app-error", error);
                        }
                        if let Err(error) = tray::refresh(&handle, &s) {
                            let _ = handle.emit("app-error", error.to_string());
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
            settings::get_settings,
            settings::save_settings,
            settings::open_settings,
            windows::set_hit_areas
        ])
        .run(tauri::generate_context!())
        .expect("error while running Little Tomato");
}
