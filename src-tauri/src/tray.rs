use crate::{
    AppState, commands,
    timer::{Phase, Snapshot, Status},
    windows,
};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
struct TrayState {
    status: MenuItem<tauri::Wry>,
    timer: MenuItem<tauri::Wry>,
    next: MenuItem<tauri::Wry>,
    last: std::sync::Mutex<String>,
}
pub fn refresh(app: &tauri::AppHandle, snapshot: &Snapshot) -> tauri::Result<()> {
    let state = app.state::<TrayState>();
    let phase = match snapshot.phase {
        Phase::Focus => "专注",
        Phase::ShortBreak => "短休息",
        Phase::LongBreak => "长休息",
    };
    let status = match snapshot.status {
        Status::Ready => "准备好了",
        Status::Running => "进行中",
        Status::Paused => "已暂停",
        Status::Completed => "已完成",
    };
    let label = format!(
        "{phase} · {status} {:02}:{:02}",
        snapshot.remaining_seconds / 60,
        snapshot.remaining_seconds % 60
    );
    let unchanged = state
        .last
        .lock()
        .map(|last| *last == label)
        .unwrap_or(false);
    if unchanged {
        return Ok(());
    }
    state.status.set_text(&label)?;
    state.timer.set_text(match snapshot.status {
        Status::Ready => "开始专注",
        Status::Running => "暂停计时",
        Status::Paused => "继续计时",
        Status::Completed => "本轮已完成",
    })?;
    state
        .timer
        .set_enabled(snapshot.status != Status::Completed)?;
    state
        .next
        .set_enabled(snapshot.status == Status::Completed)?;
    state.next.set_text(if snapshot.phase == Phase::Focus {
        "开始休息"
    } else {
        "开始下一轮专注"
    })?;
    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_tooltip(Some(format!("小番茄 · {label}")))?;
    }
    if let Ok(mut last) = state.last.lock() {
        *last = label;
    }
    Ok(())
}
pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "小番茄 · 正在准备", false, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "显示 / 隐藏", true, None::<&str>)?;
    let timer = MenuItem::with_id(app, "timer", "开始 / 暂停 / 继续", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "开始下一阶段", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出小番茄", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置…", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&status, &timer, &next, &toggle, &settings, &quit])?;
    app.manage(TrayState {
        status,
        timer,
        next,
        last: std::sync::Mutex::new(String::new()),
    });
    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("configured icon"))
        .tooltip("小番茄 · 准备好了")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => windows::toggle(app),
            "settings" => {
                if let Err(error) = crate::settings::open_settings(app.clone()) {
                    let _ = app.emit("app-error", error);
                }
            }
            "quit" => {
                windows::show(app);
                let _ = app.emit("quit-requested", ());
            }
            "timer" | "next" => {
                let state = app.state::<AppState>();
                let current = state.timer.lock().ok().map(|timer| timer.snapshot.status);
                if let Some(status) = current {
                    let action = if event.id.as_ref() == "next" {
                        "next"
                    } else {
                        match status {
                            Status::Ready => "start",
                            Status::Running => "pause",
                            Status::Paused => "resume",
                            Status::Completed => "next",
                        }
                    };
                    if let Err(error) = commands::apply_action(&state, action) {
                        let _ = app.emit("app-error", error);
                    }
                    let _ = app.emit("timer-changed", ());
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
