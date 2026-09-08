use crate::{AppState, timer::Status, windows};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)?;
    let timer = MenuItem::with_id(app, "timer", "开始 / 暂停 / 继续", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "开始下一阶段", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出小番茄", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&timer, &next, &toggle, &quit])?;
    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("configured icon"))
        .tooltip("小番茄 · 准备好了")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => windows::toggle(app),
            "quit" => {
                windows::show(app);
                let _ = app.emit("quit-requested", ());
            }
            "timer" | "next" => {
                let state = app.state::<AppState>();
                if let Ok(mut timer) = state.timer.lock() {
                    let action = if event.id.as_ref() == "next" {
                        "next"
                    } else {
                        match timer.snapshot.status {
                            Status::Ready => "start",
                            Status::Running => "pause",
                            Status::Paused => "resume",
                            Status::Completed => "next",
                        }
                    };
                    if let Err(error) = timer.action(&state.database_path, action) {
                        let _ = app.emit("app-error", error);
                    }
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                windows::toggle(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
