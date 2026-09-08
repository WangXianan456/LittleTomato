use crate::database;
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
    fn seconds(self) -> u64 {
        match self {
            Self::Focus => 1500,
            Self::ShortBreak => 300,
            Self::LongBreak => 900,
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
            }
            "reset" => {
                record = matches!(s.status, Status::Running | Status::Paused);
                *s = Snapshot {
                    rounds: s.rounds,
                    ..Snapshot::default()
                };
            }
            "next" if s.status == Status::Completed => {
                let phase = if s.phase == Phase::Focus {
                    if s.rounds.is_multiple_of(4) {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    }
                } else {
                    Phase::Focus
                };
                *s = Snapshot {
                    phase,
                    planned_seconds: phase.seconds(),
                    remaining_seconds: phase.seconds(),
                    rounds: s.rounds,
                    status: Status::Running,
                    ..Snapshot::default()
                };
            }
            "skip" if s.status == Status::Completed && s.phase == Phase::Focus => {
                *s = Snapshot {
                    rounds: s.rounds,
                    ..Snapshot::default()
                };
            }
            _ => return Err("计时状态已变化，请重试。".into()),
        }
        next.anchor = Instant::now();
        next.persist(path, record.then_some(&previous))?;
        *self = next;
        Ok(self.snapshot.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_db() -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "little-tomato-test-{}-{}",
            std::process::id(),
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
