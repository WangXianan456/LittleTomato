//! Native session/power events are processed on Tauri's window thread.
//! Never call a window API while holding the timer mutex.
use crate::AppState;
use std::sync::atomic::{AtomicU8, Ordering};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct SystemState {
    blocked: AtomicU8,
}
impl SystemState {
    pub fn blocked(&self) -> bool {
        self.blocked.load(Ordering::SeqCst) != 0
    }
    fn update(&self, bit: u8, blocked: bool) {
        if blocked {
            self.blocked.fetch_or(bit, Ordering::SeqCst);
        } else {
            self.blocked.fetch_and(!bit, Ordering::SeqCst);
        }
    }
}

fn handle(app: &tauri::AppHandle, bit: u8, blocked: bool, reason: &str) {
    let state = app.state::<AppState>();
    state.system.update(bit, blocked);
    // Suspend and resume both checkpoint defensively; unlock never starts a timer.
    let result = state
        .timer
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|mut timer| timer.interrupt(&state.database_path, reason));
    if let Err(error) = result {
        let _ = app.emit("app-error", error);
    }
    let _ = app.emit("timer-changed", ());
}

#[cfg(windows)]
mod native {
    use super::*;
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::RemoteDesktop::{
            NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
            WTSUnRegisterSessionNotification,
        },
        UI::{
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{WM_NCDESTROY, WM_POWERBROADCAST, WM_WTSSESSION_CHANGE},
        },
    };
    const SUBCLASS_ID: usize = 0x544f4d;
    unsafe extern "system" fn callback(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        data: usize,
    ) -> LRESULT {
        // The boxed handle lives until WM_NCDESTROY removes this subclass.
        let app = unsafe { &*(data as *const tauri::AppHandle) };
        match (message, wparam) {
            (WM_WTSSESSION_CHANGE, 7) => handle(app, 1, true, "locked"),
            (WM_WTSSESSION_CHANGE, 8) => handle(app, 1, false, "locked"),
            (WM_WTSSESSION_CHANGE, 2 | 4) => handle(app, 2, true, "disconnected"),
            (WM_WTSSESSION_CHANGE, 1 | 3) => handle(app, 2, false, "disconnected"),
            (WM_POWERBROADCAST, 4) => handle(app, 4, true, "sleep"),
            (WM_POWERBROADCAST, 7 | 18) => handle(app, 4, false, "sleep"),
            (WM_NCDESTROY, _) => unsafe {
                WTSUnRegisterSessionNotification(hwnd);
                RemoveWindowSubclass(hwnd, Some(callback), SUBCLASS_ID);
                drop(Box::from_raw(data as *mut tauri::AppHandle));
            },
            _ => {}
        }
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }

    pub fn install(window: &tauri::WebviewWindow) -> Result<(), String> {
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
        let data = Box::into_raw(Box::new(window.app_handle().clone()));
        // Called during setup on the thread that owns hwnd.
        unsafe {
            if SetWindowSubclass(hwnd, Some(callback), SUBCLASS_ID, data as usize) == 0 {
                drop(Box::from_raw(data));
                return Err("无法监听系统电源事件".into());
            }
            if WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0 {
                RemoveWindowSubclass(hwnd, Some(callback), SUBCLASS_ID);
                drop(Box::from_raw(data));
                return Err("无法监听 Windows 会话事件".into());
            }
        }
        Ok(())
    }
}
#[cfg(windows)]
pub use native::install;
#[cfg(not(windows))]
pub fn install(_: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn waking_does_not_clear_session_lock() {
        let state = SystemState::default();
        state.update(1, true);
        state.update(4, true);
        state.update(4, false);
        assert!(state.blocked());
        state.update(1, false);
        assert!(!state.blocked());
    }
}
