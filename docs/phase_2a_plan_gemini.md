# Phase 2a: Session-Centric Data Design (Gemini Version)

## Objective
Implement a robust 3-table SQLite schema that tracks individual stream sessions, raids, and kills. This enables comparisons like "Today's Survival Rate" vs "All-Time Survival Rate."

## 1. Database Schema (SQLite)

```sql
-- 1. SESSIONS (Tracking the stream session)
CREATE TABLE sessions (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    note TEXT
);

-- 2. RAIDS (Tracking the match within a session)
CREATE TABLE raids (
    raid_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    map_name TEXT NOT NULL,
    character_type TEXT NOT NULL CHECK(character_type IN ('pmc', 'scav')),
    game_mode TEXT NOT NULL CHECK(game_mode IN ('pve', 'pvp')),
    queue_time_seconds INTEGER,
    status TEXT NOT NULL,
    extract_location TEXT,
    stash_time_seconds INTEGER,
    review_time_seconds INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

-- 3. KILLS (Events inside a raid)
CREATE TABLE kills (
    kill_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    killed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enemy_type TEXT NOT NULL CHECK(enemy_type IN ('scav', 'pmc', 'boss', 'raider')),
    weapon_used TEXT,
    headshot BOOLEAN,
    FOREIGN KEY (raid_id) REFERENCES raids(raid_id) ON DELETE CASCADE
);

CREATE INDEX idx_raids_session_id ON raids(session_id);
CREATE INDEX idx_raids_status ON raids(status);
CREATE INDEX idx_kills_raid_id ON kills(raid_id);
```

## 2. Updated Rust Models (`src/models.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub session_id: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Raid {
    pub raid_id: Option<i64>,
    pub session_id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub map_name: String,
    pub character_type: String,
    pub game_mode: String,
    pub queue_time_seconds: Option<i32>,
    pub status: String,
    pub extract_location: Option<String>,
    pub stash_time_seconds: Option<i32>,
    pub review_time_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Kill {
    pub kill_id: Option<i64>,
    pub raid_id: i64,
    pub killed_at: DateTime<Utc>,
    pub enemy_type: String,
    pub weapon_used: Option<String>,
    pub headshot: Option<bool>,
}
```

## 3. Workflow Logic

### A. Automatic Session Creation
When the application starts, it will:
1. Check if the most recent session in the database ended less than 1 hour ago.
2. If yes: Resume that session (re-use `session_id`).
3. If no (or DB empty): Create a new `sessions` record and use the new `session_id`.

### B. Stash Time Calculation
`stash_time_seconds` for Raid N will be calculated as `RaidN.started_at - RaidN-1.ended_at`. 
If it's the first raid of a session, it will be `Raid1.started_at - Session.started_at`.