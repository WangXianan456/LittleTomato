use rusqlite::{Connection, OptionalExtension};
use std::{fs, path::Path};

pub fn open(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|e| e.to_string())?;
    connection
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")
        .map_err(|e| e.to_string())?;
    Ok(connection)
}

pub fn initialize(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut connection = open(path)?;
    let version: usize = connection
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let migrations = [
        include_str!("../migrations/001_initial.sql"),
        include_str!("../migrations/002_timer.sql"),
    ];
    if version > migrations.len() {
        return Err("数据库版本比当前程序更新，请使用新版小番茄。".into());
    }
    if version > 0 && version < migrations.len() {
        let backup = path.with_extension(format!(
            "before-v{}-{}.sqlite3",
            migrations.len(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        connection
            .execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
            .map_err(|e| e.to_string())?;
    }
    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    for sql in migrations.iter().skip(version) {
        transaction.execute_batch(sql).map_err(|e| e.to_string())?;
    }
    transaction
        .pragma_update(None, "user_version", migrations.len())
        .map_err(|e| e.to_string())?;
    transaction.commit().map_err(|e| e.to_string())
}

pub fn setting(path: &Path, key: &str) -> Result<Option<String>, String> {
    open(path)?
        .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())
}

pub fn set_setting(path: &Path, key: &str, value: &str) -> Result<(), String> {
    open(path)?.execute("INSERT INTO settings(key,value,updated_at) VALUES(?1,?2,unixepoch()) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at", [key,value]).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> std::path::PathBuf {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tomato-migration-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        dir.join("test.sqlite3")
    }
    #[test]
    fn upgrade_preserves_legacy_data_and_backs_up_wal() {
        let path = path();
        let c = open(&path).unwrap();
        c.execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        c.execute("INSERT INTO settings VALUES('test','preserved',0)", [])
            .unwrap();
        initialize(&path).unwrap();
        initialize(&path).unwrap();
        assert_eq!(
            setting(&path, "test").unwrap().as_deref(),
            Some("preserved")
        );
        let backups: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains("before-v2"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            setting(&backups[0].path(), "test").unwrap().as_deref(),
            Some("preserved")
        );
        drop(c);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn failed_upgrade_rolls_back_schema_and_version() {
        let path = path();
        let c = open(&path).unwrap();
        c.execute_batch(include_str!("../migrations/001_initial.sql"))
            .unwrap();
        c.execute_batch("CREATE TABLE timer_history(id INTEGER);")
            .unwrap();
        assert!(initialize(&path).is_err());
        let version: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
        let count: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='timer_state'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
        drop(c);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
