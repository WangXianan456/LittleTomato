use crate::database;

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(key: i32) -> i16;
}

fn pointer_pressed() -> bool {
    #[cfg(target_os = "windows")]
    // Query only; keep the existing hit target during a mouse press or native drag.
    unsafe {
        GetAsyncKeyState(1) < 0 || GetAsyncKeyState(2) < 0
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent};

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PointerPose {
    x: f64,
    y: f64,
    window_x: f64,
    window_y: f64,
    pressed: bool,
}

#[derive(Clone, Deserialize)]
pub struct HitArea {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
}
impl HitArea {
    fn contains(&self, x: f64, y: f64) -> bool {
        let x = x - self.x;
        let y = y - self.y;
        if x < 0.0 || y < 0.0 || x > self.width || y > self.height {
            return false;
        }
        let r = self
            .radius
            .min(self.width / 2.0)
            .min(self.height / 2.0)
            .max(0.0);
        let dx = x - x.clamp(r, self.width - r);
        let dy = y - y.clamp(r, self.height - r);
        dx * dx + dy * dy <= r * r
    }
}
#[derive(Default)]
pub struct Interaction {
    pub areas: Mutex<Vec<HitArea>>,
}
#[tauri::command]
pub fn set_hit_areas(
    state: tauri::State<'_, Interaction>,
    areas: Vec<HitArea>,
) -> Result<(), String> {
    if areas.len() > 32
        || areas.iter().any(|a| {
            ![a.x, a.y, a.width, a.height, a.radius]
                .iter()
                .all(|n| n.is_finite())
                || a.width < 0.0
                || a.height < 0.0
        })
    {
        return Err("Invalid hit areas".into());
    }
    *state.areas.lock().map_err(|e| e.to_string())? = areas;
    Ok(())
}
pub fn place(window: &WebviewWindow, saved: Option<(i32, i32)>) -> tauri::Result<()> {
    let monitors = window.available_monitors()?;
    let size = window.outer_size()?;
    if monitors.is_empty() {
        return Ok(());
    }
    // The pet stays wherever it is dropped, edges included. The only guard
    // rail keeps a small strip on the desktop so it can never be lost past
    // the last monitor; there is no margin and no edge snapping.
    const KEEP: i32 = 24;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for m in &monitors {
        let a = m.work_area();
        min_x = min_x.min(a.position.x);
        min_y = min_y.min(a.position.y);
        max_x = max_x.max(a.position.x + a.size.width as i32);
        max_y = max_y.max(a.position.y + a.size.height as i32);
    }
    let (x, y) = match saved {
        Some((x, y)) => (x, y),
        None => {
            let primary = window.primary_monitor()?;
            let fallback = monitors.first().map(|m| m.work_area());
            let a = match primary.as_ref().map(|m| m.work_area()) {
                Some(a) => a,
                None => match fallback {
                    Some(a) => a,
                    None => return Ok(()),
                },
            };
            let margin = (12.0 * window.scale_factor().unwrap_or(1.0)) as i32;
            (
                a.position.x + a.size.width as i32 - size.width as i32 - margin,
                a.position.y + a.size.height as i32 - size.height as i32 - margin,
            )
        }
    };
    let x = x.clamp(min_x - size.width as i32 + KEEP, max_x - KEEP);
    let y = y.clamp(min_y - size.height as i32 + KEEP, max_y - KEEP);
    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}
pub fn setup(window: WebviewWindow, path: &Path) -> Result<(), String> {
    let saved = database::setting(path, "window_position")?
        .and_then(|json| serde_json::from_str(&json).ok());
    place(&window, saved).map_err(|e| e.to_string())?;
    let close_window = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = close_window.hide();
        }
    });
    let path = path.to_owned();
    std::thread::spawn(move || {
        let mut ignored = false;
        let mut last_position = window.outer_position().ok();
        let mut moved_at = Instant::now();
        let mut dirty = false;
        let mut checked = Instant::now();
        let mut last_pose = None;
        loop {
            std::thread::sleep(Duration::from_millis(32));
            let Ok(position) = window.outer_position() else {
                break;
            };
            if last_position != Some(position) {
                last_position = Some(position);
                moved_at = Instant::now();
                dirty = true;
            }
            if dirty && moved_at.elapsed() > Duration::from_millis(500) && !pointer_pressed() {
                if let Err(e) = place(&window, Some((position.x, position.y))) {
                    eprintln!("Window placement: {e}");
                }
                if let Ok(p) = window.outer_position() {
                    if let Err(e) = database::set_setting(
                        &path,
                        "window_position",
                        &format!("[{},{}]", p.x, p.y),
                    ) {
                        eprintln!("Save placement: {e}");
                    }
                    last_position = Some(p);
                }
                dirty = false;
            }
            if checked.elapsed() > Duration::from_secs(3) && !dirty && !pointer_pressed() {
                let _ = place(&window, Some((position.x, position.y)));
                checked = Instant::now();
            }
            if !window.is_visible().unwrap_or(false) {
                last_pose = None;
                continue;
            }
            if let (Ok(cursor), Ok(scale)) = (window.cursor_position(), window.scale_factor()) {
                let pose = PointerPose {
                    x: (cursor.x - position.x as f64) / scale,
                    y: (cursor.y - position.y as f64) / scale,
                    window_x: position.x as f64 / scale,
                    window_y: position.y as f64 / scale,
                    pressed: pointer_pressed(),
                };
                if last_pose.as_ref() != Some(&pose) {
                    let _ = window.emit("pet-pointer", &pose);
                    last_pose = Some(pose);
                }
                let state = window.state::<Interaction>();
                let Ok(areas) = state.areas.lock() else {
                    continue;
                };
                let hit = areas.is_empty()
                    || areas.iter().any(|a| {
                        a.contains(
                            (cursor.x - position.x as f64) / scale,
                            (cursor.y - position.y as f64) / scale,
                        )
                    });
                // Tauri dispatches window calls to the UI thread. Never hold a
                // mutex that an IPC command on that thread may also need.
                drop(areas);
                if ignored == hit
                    && !pointer_pressed()
                    && window.set_ignore_cursor_events(!hit).is_ok()
                {
                    ignored = !hit;
                }
            }
        }
    });
    Ok(())
}
pub fn show(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let p = w.outer_position().ok().map(|p| (p.x, p.y));
        let _ = place(&w, p);
        let _ = w.show();
        let _ = w.set_focus();
    }
}
pub fn toggle(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show(app);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transparent_corners_are_not_interactive() {
        let a = HitArea {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 60.0,
            radius: 20.0,
        };
        assert!(!a.contains(10.0, 20.0));
        assert!(a.contains(60.0, 50.0));
        assert!(!a.contains(0.0, 50.0));
    }
}
