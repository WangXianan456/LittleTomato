CREATE TABLE IF NOT EXISTS settings (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL,
                 updated_at INTEGER NOT NULL
             );

             CREATE TABLE IF NOT EXISTS timer_sessions (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 kind TEXT NOT NULL CHECK (kind IN ('focus', 'break')),
                 planned_seconds INTEGER NOT NULL CHECK (planned_seconds > 0),
                 elapsed_seconds INTEGER NOT NULL CHECK (elapsed_seconds >= 0),
                 completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1)),
                 started_at INTEGER NOT NULL,
                 ended_at INTEGER NOT NULL
             );

             CREATE TABLE IF NOT EXISTS pet_state (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 personality TEXT NOT NULL DEFAULT 'gentle',
                 mood TEXT NOT NULL DEFAULT 'calm',
                 affinity INTEGER NOT NULL DEFAULT 0 CHECK (affinity >= 0),
                 updated_at INTEGER NOT NULL
             );

             INSERT OR IGNORE INTO pet_state (id, updated_at)
             VALUES (1, unixepoch());

             PRAGMA user_version = 1;