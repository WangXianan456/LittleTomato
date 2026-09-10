use crate::{AppState, database, timer::Snapshot};
use serde::Serialize;
use tauri::State;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    completed_sessions: i64,
    focused_seconds: i64,
}
#[tauri::command]
pub fn runtime_status(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    database::open(&state.database_path)?.query_row("SELECT COUNT(*), COALESCE(SUM(elapsed_seconds),0) FROM (SELECT elapsed_seconds FROM timer_sessions WHERE kind='focus' AND completed=1 UNION ALL SELECT elapsed_seconds FROM timer_history WHERE phase='focus' AND completed=1)", [], |r| Ok(RuntimeStatus{completed_sessions:r.get(0)?,focused_seconds:r.get(1)?})).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn timer_snapshot(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    if state.system.blocked() {
        timer.interrupt(&state.database_path, "system")?;
    } else {
        timer.tick(&state.database_path)?;
    }
    Ok(timer.snapshot.clone())
}
#[tauri::command]
pub fn timer_action(state: State<'_, AppState>, action: String) -> Result<Snapshot, String> {
    apply_action(&state, &action)
}
pub fn apply_action(state: &AppState, action: &str) -> Result<Snapshot, String> {
    if state.system.blocked() && matches!(action, "start" | "resume" | "next") {
        return Err("系统仍在锁定或休眠，返回桌面后再继续吧。".into());
    }
    state
        .timer
        .lock()
        .map_err(|e| e.to_string())?
        .action(&state.database_path, action)
}
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    if timer.snapshot.status == crate::timer::Status::Running {
        timer.action(&state.database_path, "pause")?;
    }
    drop(timer);
    app.exit(0);
    Ok(())
}
