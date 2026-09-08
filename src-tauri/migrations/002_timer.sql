CREATE TABLE timer_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    snapshot TEXT NOT NULL
);
CREATE TABLE timer_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    phase TEXT NOT NULL CHECK (phase IN ('focus', 'short_break', 'long_break')),
    planned_seconds INTEGER NOT NULL CHECK (planned_seconds > 0),
    elapsed_seconds INTEGER NOT NULL CHECK (elapsed_seconds >= 0 AND elapsed_seconds <= planned_seconds),
    completed INTEGER NOT NULL CHECK (completed IN (0, 1)),
    ended_at INTEGER NOT NULL DEFAULT (unixepoch())
);
