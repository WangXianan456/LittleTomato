use crate::{AppState, database, windows};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{Emitter, Manager, State};
// auto-launch writes Windows Run values as a command line: quote each path.
fn startup_arg(value: &str) -> String {
    if !cfg!(windows) {
        return value.to_string();
    }
    let mut result = String::from("\"");
    let mut slashes = 0;
    for ch in value.chars() {
        if ch == '\\' {
            slashes += 1;
            continue;
        }
        result.push_str(&"\\".repeat(if ch == '\"' { slashes * 2 + 1 } else { slashes }));
        result.push(ch);
        slashes = 0;
    }
    result.push_str(&"\\".repeat(slashes * 2));
    result.push('\"');
    result
}
fn autolaunch(app: &tauri::AppHandle) -> Result<auto_launch::AutoLaunch, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let args = std::env::var_os("LITTLE_TOMATO_RUNTIME_DIR")
        .map(|path| vec!["--runtime-dir".into(), startup_arg(&path.to_string_lossy())])
        .unwrap_or_default();
    auto_launch::AutoLaunchBuilder::new()
        .set_app_name(&app.package_info().name)
        .set_app_path(&startup_arg(&executable.to_string_lossy()))
        .set_args(&args)
        .build()
        .map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    #[default]
    Tomato,
    Peach,
    Sprout,
    Cloud,
    Cat,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Settings {
    pub character: Character,
    pub focus_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub rounds_before_long_break: u32,
    pub pet_size: u32,
    pub always_on_top: bool,
    pub reduced_motion: bool,
    pub theme: Theme,
    pub launch_on_startup: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            character: Character::Tomato,
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            rounds_before_long_break: 4,
            pet_size: 160,
            always_on_top: true,
            reduced_motion: false,
            theme: Theme::System,
            launch_on_startup: false,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        for (value, min, max, label) in [
            (self.focus_minutes, 1, 180, "专注时长"),
            (self.short_break_minutes, 1, 60, "短休息时长"),
            (self.long_break_minutes, 1, 120, "长休息时长"),
            (self.rounds_before_long_break, 2, 12, "长休息轮数"),
            (self.pet_size, 80, 240, "桌宠大小"),
        ] {
            if !(min..=max).contains(&value) {
                return Err(format!("{label}需在 {min}–{max} 之间。"));
            }
        }
        Ok(())
    }
}
pub fn load(path: &Path) -> Result<Settings, String> {
    let result = match database::setting(path, "app_settings")? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|_| "设置暂时无法读取，请恢复推荐设置。".to_string())?,
        None => Settings::default(),
    };
    result.validate()?;
    Ok(result)
}
pub fn save(path: &Path, settings: &Settings) -> Result<(), String> {
    settings.validate()?;
    database::set_setting(
        path,
        "app_settings",
        &serde_json::to_string(settings).map_err(|e| e.to_string())?,
    )
}

#[tauri::command]
pub fn get_settings(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<Settings, String> {
    let mut settings = load(&state.database_path)?;
    settings.launch_on_startup = autolaunch(&app)?
        .is_enabled()
        .map_err(|_| "暂时无法读取开机启动状态，请重试。".to_string())?;
    Ok(settings)
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    settings: Settings,
    warning: Option<String>,
}
#[tauri::command]
pub fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mut settings: Settings,
) -> Result<SaveResult, String> {
    settings.validate()?;
    let launcher = autolaunch(&app)?;
    let previous = launcher
        .is_enabled()
        .map_err(|_| "无法确认开机启动状态，请稍后重试。".to_string())?;
    let mut warning = None;
    if previous != settings.launch_on_startup {
        let result = if settings.launch_on_startup {
            launcher.enable()
        } else {
            launcher.disable()
        };
        if result.is_err() {
            settings.launch_on_startup = previous;
            warning = Some("其他设置已保存，开机启动未能修改，请稍后重试。".into());
        }
    }
    if let Err(error) = save(&state.database_path, &settings) {
        if settings.launch_on_startup != previous {
            let rollback = if previous {
                launcher.enable()
            } else {
                launcher.disable()
            };
            if rollback.is_err() {
                return Err("设置保存失败，开机启动可能已改变，请重新打开设置检查。".into());
            }
        }
        eprintln!("Save settings: {error}");
        return Err("设置没有保存成功，请稍后重试。".into());
    }
    if apply(&app, &settings).is_err() {
        warning = Some("设置已保存，部分外观将在下次启动时应用。".into());
    }
    let _ = app.emit("settings-changed", &settings);
    Ok(SaveResult { settings, warning })
}
pub fn apply(app: &tauri::AppHandle, settings: &Settings) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(settings.always_on_top)?;
        window.set_size(tauri::LogicalSize::new(
            300.0,
            420.0 + (settings.pet_size as f64 - 160.0) * 1.1,
        ))?;
        let position = window.outer_position()?;
        windows::place(&window, Some((position.x, position.y)))?;
    }
    Ok(())
}
#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or("设置面板暂时无法打开")?;
    // Fit the ordinary panel to smaller or high-DPI work areas.
    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        window
            .set_size(tauri::LogicalSize::new(
                (area.size.width as f64 / scale - 40.0).min(720.0),
                (area.size.height as f64 / scale - 70.0).min(720.0),
            ))
            .map_err(|e| e.to_string())?;
        window.center().map_err(|e| e.to_string())?;
    }
    window.show().map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_and_invalid_fields() {
        let mut s = Settings::default();
        assert!(s.validate().is_ok());
        s.focus_minutes = 0;
        assert!(s.validate().is_err());
        s.focus_minutes = 180;
        assert!(s.validate().is_ok());
        s.pet_size = 241;
        assert!(s.validate().is_err());
        assert!(serde_json::from_str::<Settings>(r#"{"theme":"unknown"}"#).is_err());
        assert!(serde_json::from_str::<Settings>(r#"{"focusMinutes":1.5}"#).is_err());
        assert!(serde_json::from_str::<Settings>(r#"{"arbitrary":true}"#).is_err());
    }
    #[test]
    fn missing_fields_use_recommended_defaults() {
        let s: Settings = serde_json::from_str(r#"{"focusMinutes":40}"#).unwrap();
        assert_eq!(s.focus_minutes, 40);
        assert_eq!(s.pet_size, 160);
        assert_eq!(s.theme, Theme::System);
        assert_eq!(s.character, Character::Tomato);
    }
    #[test]
    fn character_settings_roundtrip_and_reject_unknown_character() {
        for id in ["tomato", "peach", "sprout", "cloud", "cat"] {
            let json = format!(r#"{{"character":"{id}"}}"#);
            let settings: Settings = serde_json::from_str(&json).unwrap();
            assert_eq!(serde_json::to_value(&settings).unwrap()["character"], id);
        }
        assert!(serde_json::from_str::<Settings>(r#"{"character":"missing"}"#).is_err());
    }
    #[test]
    #[cfg(windows)]
    fn startup_quotes_paths_with_spaces_and_trailing_backslashes() {
        assert_eq!(
            startup_arg(r"D:\My Apps\Tomato.exe"),
            r#""D:\My Apps\Tomato.exe""#
        );
        assert_eq!(startup_arg(r"D:\My Data\"), r#""D:\My Data\\""#);
    }
}
