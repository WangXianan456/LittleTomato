use crate::{database, settings::Settings};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}
impl Phase {
    fn seconds(self, settings: &Settings) -> u64 {
        match self {
            Self::Focus => settings.focus_minutes as u64 * 60,
            Self::ShortBreak => settings.short_break_minutes as u64 * 60,
            Self::LongBreak => settings.long_break_minutes as u64 * 60,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ready,
    Running,
    Paused,
    Completed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: Phase,
    pub status: Status,
    pub planned_seconds: u64,
    pub elapsed_ms: u64,
    pub remaining_seconds: u64,
    pub rounds: u32,
    pub recovery: bool,
    #[serde(default)]
    pub recovery_reason: Option<String>,
    #[serde(default = "default_rounds")]
    pub rounds_before_long_break: u32,
}
fn default_rounds() -> u32 {
    4
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: Phase::Focus,
            status: Status::Ready,
            planned_seconds: 1500,
            elapsed_ms: 0,
            remaining_seconds: 1500,
            rounds: 0,
            recovery: false,
            recovery_reason: None,
            rounds_before_long_break: 4,
        }
    }
}
impl Snapshot {
    fn ready(rounds: u32, settings: &Settings) -> Self {
        Self {
            rounds,
            planned_seconds: Phase::Focus.seconds(settings),
            remaining_seconds: Phase::Focus.seconds(settings),
            rounds_before_long_break: settings.rounds_before_long_break,
            ..Self::default()
        }
    }
}
#[derive(Clone)]
pub struct Timer {
    pub snapshot: Snapshot,
    anchor: Instant,
}
impl Timer {
    pub fn load(path: &Path) -> Result<Self, String> {
        let connection = database::open(path)?;
        let saved: Option<String> = connection
            .query_row("SELECT snapshot FROM timer_state WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let mut snapshot: Snapshot = match saved {
            Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string())?,
            None => Snapshot::default(),
        };
        if matches!(snapshot.status, Status::Running | Status::Paused) {
            snapshot.status = Status::Paused;
            snapshot.recovery = true;
            snapshot.recovery_reason = Some("restart".into());
        }
        if snapshot.status == Status::Ready {
            snapshot = Snapshot::ready(snapshot.rounds, &crate::settings::load(path)?);
        }
        let timer = Self {
            snapshot,
            anchor: Instant::now(),
        };
        timer.persist(path, None)?;
        Ok(timer)
    }
    fn persist(&self, path: &Path, record: Option<&Snapshot>) -> Result<(), String> {
        let mut connection = database::open(path)?;
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        if let Some(s) = record {
            let phase = serde_json::to_value(s.phase).map_err(|e| e.to_string())?;
            tx.execute("INSERT INTO timer_history(phase,planned_seconds,elapsed_seconds,completed) VALUES(?1,?2,?3,?4)", params![phase.as_str(), s.planned_seconds, s.elapsed_ms / 1000, s.status == Status::Completed]).map_err(|e| e.to_string())?;
        }
        tx.execute("INSERT INTO timer_state(id,snapshot) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET snapshot=excluded.snapshot", [serde_json::to_string(&self.snapshot).map_err(|e| e.to_string())?]).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    fn advance(&mut self, ms: u64) -> bool {
        let s = &mut self.snapshot;
        if s.status != Status::Running {
            return false;
        }
        // A stalled process must not count an unobserved absence as focus time.
        if ms > 3000 {
            s.status = Status::Paused;
            s.recovery = true;
            s.recovery_reason = Some("unresponsive".into());
            return false;
        }
        s.elapsed_ms = (s.elapsed_ms + ms).min(s.planned_seconds * 1000);
        s.remaining_seconds = (s.planned_seconds * 1000 - s.elapsed_ms).div_ceil(1000);
        if s.remaining_seconds == 0 {
            s.status = Status::Completed;
            if s.phase == Phase::Focus {
                s.rounds += 1;
            }
            return true;
        }
        false
    }
    pub fn tick(&mut self, path: &Path) -> Result<(), String> {
        let mut next = self.clone();
        if next.snapshot.status == Status::Ready {
            let settings = crate::settings::load(path)?;
            let ready = Snapshot::ready(next.snapshot.rounds, &settings);
            if ready.planned_seconds != next.snapshot.planned_seconds
                || ready.rounds_before_long_break != next.snapshot.rounds_before_long_break
            {
                next.snapshot = ready;
                next.persist(path, None)?;
            }
        }
        let now = Instant::now();
        let completed = next.advance(now.duration_since(next.anchor).as_millis() as u64);
        next.anchor = now;
        if self.snapshot.status == Status::Running {
            next.persist(path, completed.then_some(&next.snapshot))?;
        }
        *self = next;
        Ok(())
    }
    pub fn action(&mut self, path: &Path, action: &str) -> Result<Snapshot, String> {
        self.tick(path)?;
        let settings = crate::settings::load(path)?;
        let mut next = self.clone();
        let previous = next.snapshot.clone();
        let s = &mut next.snapshot;
        let mut record = false;
        match action {
            "start" if s.status == Status::Ready => s.status = Status::Running,
            "pause" if s.status == Status::Running => s.status = Status::Paused,
            "resume" if s.status == Status::Paused => {
                s.status = Status::Running;
                s.recovery = false;
                s.recovery_reason = None;
            }
            "reset" => {
                record = matches!(s.status, Status::Running | Status::Paused);
                *s = Snapshot::ready(s.rounds, &settings);
            }
            "next" if s.status == Status::Completed => {
                let phase = if s.phase == Phase::Focus {
                    if s.rounds.is_multiple_of(s.rounds_before_long_break.max(1)) {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    }
                } else {
                    Phase::Focus
                };
                *s = Snapshot {
                    phase,
                    planned_seconds: phase.seconds(&settings),
                    remaining_seconds: phase.seconds(&settings),
                    rounds_before_long_break: settings.rounds_before_long_break,
                    rounds: s.rounds,
                    status: Status::Running,
                    ..Snapshot::default()
                };
            }
            "skip" if s.status == Status::Completed && s.phase == Phase::Focus => {
                *s = Snapshot::ready(s.rounds, &settings);
            }
            _ => return Err("计时状态已变化，请重试。".into()),
        }
        next.anchor = Instant::now();
        next.persist(path, record.then_some(&previous))?;
        *self = next;
        Ok(self.snapshot.clone())
    }

    pub fn interrupt(&mut self, path: &Path, reason: &str) -> Result<(), String> {
        if self.snapshot.status != Status::Running {
            return Ok(());
        }
        let mut next = self.clone();
        // Keep the last observed checkpoint: a resume event may arrive after sleep.
        next.snapshot.status = Status::Paused;
        next.snapshot.recovery = true;
        next.snapshot.recovery_reason = Some(reason.into());
        next.anchor = Instant::now();
        next.persist(path, None)?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_db() -> std::path::PathBuf {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "little-tomato-test-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("test.sqlite3");
        database::initialize(&path).unwrap();
        path
    }
    #[test]
    fn restart_requires_resume_and_actions_are_validated() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        timer.action(&path, "start").unwrap();
        assert!(timer.action(&path, "start").is_err());
        timer.advance(2000);
        timer.persist(&path, None).unwrap();
        let saved_elapsed = timer.snapshot.elapsed_ms;
        let mut recovered = Timer::load(&path).unwrap();
        assert_eq!(recovered.snapshot.status, Status::Paused);
        assert!(recovered.snapshot.recovery);
        assert_eq!(recovered.snapshot.elapsed_ms, saved_elapsed);
        recovered.action(&path, "resume").unwrap();
        recovered.action(&path, "pause").unwrap();
        assert!(!recovered.snapshot.recovery);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn settings_apply_to_new_phases_but_preserve_active_plan() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        timer.action(&path, "start").unwrap();
        let settings = Settings {
            focus_minutes: 40,
            short_break_minutes: 7,
            long_break_minutes: 20,
            rounds_before_long_break: 2,
            ..Settings::default()
        };
        crate::settings::save(&path, &settings).unwrap();
        timer.tick(&path).unwrap();
        assert_eq!(timer.snapshot.planned_seconds, 1500);
        assert_eq!(timer.snapshot.rounds_before_long_break, 4);
        timer.action(&path, "pause").unwrap();
        let restored = Timer::load(&path).unwrap();
        assert_eq!(restored.snapshot.planned_seconds, 1500);
        timer.action(&path, "reset").unwrap();
        assert_eq!(timer.snapshot.planned_seconds, 2400);
        assert_eq!(timer.snapshot.rounds_before_long_break, 2);
        timer.snapshot.status = Status::Completed;
        timer.snapshot.rounds = 2;
        timer.action(&path, "next").unwrap();
        assert_eq!(timer.snapshot.phase, Phase::LongBreak);
        assert_eq!(timer.snapshot.planned_seconds, 1200);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn ready_state_updates_and_invalid_settings_preserve_saved_values() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        let settings = Settings {
            focus_minutes: 45,
            pet_size: 240,
            ..Settings::default()
        };
        crate::settings::save(&path, &settings).unwrap();
        timer.tick(&path).unwrap();
        assert_eq!(timer.snapshot.remaining_seconds, 2700);
        assert_eq!(crate::settings::load(&path).unwrap(), settings);
        let invalid = Settings {
            focus_minutes: 0,
            ..settings.clone()
        };
        assert!(crate::settings::save(&path, &invalid).is_err());
        assert_eq!(crate::settings::load(&path).unwrap(), settings);
        let c = database::open(&path).unwrap();
        c.execute_batch("CREATE TRIGGER reject_settings BEFORE UPDATE ON settings BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        assert!(crate::settings::save(&path, &Settings::default()).is_err());
        assert_eq!(crate::settings::load(&path).unwrap(), settings);
        drop(c);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn native_interrupt_persists_without_counting_absence() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        timer.action(&path, "start").unwrap();
        timer.advance(1500);
        let elapsed = timer.snapshot.elapsed_ms;
        timer.anchor = Instant::now() - std::time::Duration::from_secs(60);
        timer.interrupt(&path, "sleep").unwrap();
        assert_eq!(timer.snapshot.status, Status::Paused);
        assert_eq!(timer.snapshot.elapsed_ms, elapsed);
        assert_eq!(timer.snapshot.recovery_reason.as_deref(), Some("sleep"));
        timer.interrupt(&path, "locked").unwrap();
        assert_eq!(timer.snapshot.recovery_reason.as_deref(), Some("sleep"));
        let restored = Timer::load(&path).unwrap();
        assert_eq!(restored.snapshot.elapsed_ms, elapsed);
        timer.action(&path, "resume").unwrap();
        assert!(!timer.snapshot.recovery);
        assert!(timer.snapshot.recovery_reason.is_none());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn interruption_does_not_change_idle_or_completed_sessions() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        timer.interrupt(&path, "locked").unwrap();
        assert_eq!(timer.snapshot.status, Status::Ready);
        timer.snapshot.status = Status::Completed;
        timer.interrupt(&path, "sleep").unwrap();
        assert_eq!(timer.snapshot.status, Status::Completed);
        assert!(!timer.snapshot.recovery);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn interruption_write_failure_is_retryable() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        timer.action(&path, "start").unwrap();
        let c = database::open(&path).unwrap();
        c.execute_batch("CREATE TRIGGER reject_write BEFORE UPDATE ON timer_state BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        assert!(timer.interrupt(&path, "locked").is_err());
        assert_eq!(timer.snapshot.status, Status::Running);
        c.execute_batch("DROP TRIGGER reject_write;").unwrap();
        timer.interrupt(&path, "locked").unwrap();
        assert_eq!(timer.snapshot.status, Status::Paused);
        drop(c);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn four_rounds_lead_to_long_break_and_single_history_records() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        for round in 1..=4 {
            timer.action(&path, "start").unwrap();
            timer.snapshot.elapsed_ms = 1_499_000;
            timer.anchor = Instant::now() - std::time::Duration::from_secs(1);
            timer.tick(&path).unwrap();
            timer.tick(&path).unwrap();
            assert_eq!(timer.snapshot.rounds, round);
            assert_eq!(timer.snapshot.status, Status::Completed);
            timer.action(&path, "next").unwrap();
            assert_eq!(
                timer.snapshot.phase,
                if round == 4 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            );
            timer.action(&path, "reset").unwrap();
        }
        let connection = database::open(&path).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM timer_history WHERE completed=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
        drop(connection);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn failed_write_keeps_in_memory_state() {
        let path = test_db();
        let mut timer = Timer::load(&path).unwrap();
        let connection = database::open(&path).unwrap();
        connection.execute_batch("CREATE TRIGGER reject_write BEFORE UPDATE ON timer_state BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(timer.action(&path, "start").is_err());
        assert_eq!(timer.snapshot.status, Status::Ready);
        drop(connection);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn monotonic_progress_and_pause() {
        let mut timer = Timer {
            snapshot: Snapshot {
                status: Status::Running,
                ..Snapshot::default()
            },
            anchor: Instant::now(),
        };
        timer.advance(1250);
        assert_eq!(timer.snapshot.remaining_seconds, 1499);
        timer.snapshot.status = Status::Paused;
        timer.advance(1000);
        assert_eq!(timer.snapshot.elapsed_ms, 1250);
    }
    #[test]
    fn stalled_process_requires_confirmation() {
        let mut timer = Timer {
            snapshot: Snapshot {
                status: Status::Running,
                ..Snapshot::default()
            },
            anchor: Instant::now(),
        };
        timer.advance(60_000);
        assert_eq!(timer.snapshot.elapsed_ms, 0);
        assert!(timer.snapshot.recovery);
    }
    #[test]
    fn completion_is_emitted_once() {
        let mut timer = Timer {
            snapshot: Snapshot {
                status: Status::Running,
                elapsed_ms: 1_499_000,
                ..Snapshot::default()
            },
            anchor: Instant::now(),
        };
        assert!(timer.advance(1000));
        assert!(!timer.advance(1000));
        assert_eq!(timer.snapshot.rounds, 1);
    }
}
