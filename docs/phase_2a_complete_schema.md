# Phase 2a: Complete Database Schema with Sessions

## Overview

This document defines the complete database schema for Tarkov Stream Producer with session tracking and state transitions.

**Architecture:** 4-table design supporting granular time tracking and session-level analytics.

```
stream_sessions (1) ──< raids (many) ──< raid_state_transitions (many)
                                     └──< kills (many)
```

## Table Relationships

- **stream_sessions**: Top-level grouping for a streaming session or play session
- **raids**: Individual raid attempts, linked to a session
- **raid_state_transitions**: Every state change within a raid (stash → queue → raid → etc.)
- **kills**: Individual kills within a raid

## Complete SQL Schema

```sql_claude
-- ============================================================
-- Stream Sessions Table
-- ============================================================
-- Tracks distinct streaming/play sessions for analytics
-- Enables "this stream vs overall" comparisons

CREATE TABLE stream_sessions (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    session_type TEXT CHECK(session_type IN ('stream', 'practice', 'casual')),
    notes TEXT  -- Optional: "Saturday evening raid stream", viewer count, etc.
);

CREATE INDEX idx_sessions_started_at ON stream_sessions(started_at);

-- ============================================================
-- Raids Table
-- ============================================================
-- Core entity: one record per raid attempt
-- Links to session for grouping, tracks high-level raid metadata

CREATE TABLE raids (
    raid_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    map_name TEXT NOT NULL,
    character_type TEXT NOT NULL CHECK(character_type IN ('pmc', 'scav')),
    game_mode TEXT NOT NULL CHECK(game_mode IN ('pve', 'pvp')),
    current_state TEXT NOT NULL DEFAULT 'stash_management',
    extract_location TEXT,

    FOREIGN KEY (session_id) REFERENCES stream_sessions(session_id) ON DELETE SET NULL
);

-- Indexes for common queries
CREATE INDEX idx_raids_session_id ON raids(session_id);
CREATE INDEX idx_raids_started_at ON raids(started_at);
CREATE INDEX idx_raids_current_state ON raids(current_state);
CREATE INDEX idx_raids_game_mode ON raids(game_mode);

-- ============================================================
-- Raid State Transitions Table
-- ============================================================
-- Tracks every state change within a raid for granular time analysis
-- Enables queries like "How much time in queue vs stash vs raid?"

CREATE TABLE raid_state_transitions (
    transition_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    from_state TEXT,  -- NULL for first transition in raid
    to_state TEXT NOT NULL,
    transitioned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (raid_id) REFERENCES raids(raid_id) ON DELETE CASCADE
);

CREATE INDEX idx_transitions_raid_id ON raid_state_transitions(raid_id);
CREATE INDEX idx_transitions_to_state ON raid_state_transitions(to_state);
CREATE INDEX idx_transitions_time ON raid_state_transitions(transitioned_at);

-- ============================================================
-- Kills Table
-- ============================================================
-- Tracks individual kills within a raid

CREATE TABLE kills (
    kill_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    killed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enemy_type TEXT NOT NULL,  -- No CHECK constraint for extensibility (Phase 4 may discover new types)
    weapon_used TEXT,
    headshot BOOLEAN,

    FOREIGN KEY (raid_id) REFERENCES raids(raid_id) ON DELETE CASCADE
);

CREATE INDEX idx_kills_raid_id ON kills(raid_id);
CREATE INDEX idx_kills_enemy_type ON kills(enemy_type);
```

## Known States

The `current_state` field and `raid_state_transitions.to_state` use these values:

### Active States (Transitional)
- `idle` - Main menu, not actively playing
- `stash_management` - Character/Traders/Hideout/Flea ("stash tetris")
- `pre_raid_setup` - Character select → Map select → Insurance
- `queuing` - In matchmaking queue
- `deploying_cancellable` - Loading screens (can still back out)
- `deploying_committed` - Past point of no return (PVE: "Starting Local Game", PVP: matched)
- `raid_active` - In-game FPS gameplay
- `raid_ending` - Transfer screen or end screen appearing
- `post_raid_review` - Kill List → Statistics → Experience screens

### Terminal States
- `survived` - Raid ended successfully (extracted)
- `died` - Player died in raid
- `mia` - Missing in action (didn't extract in time)

### Future/Edge Case States
- `transfer` - Map-to-map movement (Streets → Labs transfer)
- `reconnecting` - Connection lost, attempting reconnect
- `error` - Game error/crash mid-raid

**Note:** No CHECK constraint on status field - allows discovery of new states during Phase 4 OCR implementation.

## Example Data Flow

### Session Lifecycle

```sql_claude
-- User presses "Start Stream" button on Stream Deck
INSERT INTO stream_sessions (session_type, notes)
VALUES ('stream', 'Saturday evening raids');
-- Returns session_id = 42

-- User starts first raid
INSERT INTO raids (session_id, map_name, character_type, game_mode, current_state)
VALUES (42, 'Customs', 'pmc', 'pvp', 'stash_management');
-- Returns raid_id = 100

-- State transitions during raid
INSERT INTO raid_state_transitions (raid_id, from_state, to_state, transitioned_at)
VALUES
    (100, NULL, 'stash_management', '2025-12-24 19:00:00'),
    (100, 'stash_management', 'pre_raid_setup', '2025-12-24 19:05:00'),
    (100, 'pre_raid_setup', 'queuing', '2025-12-24 19:07:00'),
    (100, 'queuing', 'deploying_committed', '2025-12-24 19:09:00'),
    (100, 'deploying_committed', 'raid_active', '2025-12-24 19:11:00'),
    (100, 'raid_active', 'raid_ending', '2025-12-24 19:25:00'),
    (100, 'raid_ending', 'post_raid_review', '2025-12-24 19:25:30'),
    (100, 'post_raid_review', 'survived', '2025-12-24 19:27:00');

-- Update raid with final state
UPDATE raids
SET current_state = 'survived', ended_at = '2025-12-24 19:27:00', extract_location = 'Crossroads'
WHERE raid_id = 100;

-- User continues streaming, starts another raid
INSERT INTO raids (session_id, map_name, character_type, game_mode, current_state)
VALUES (42, 'Woods', 'pmc', 'pvp', 'stash_management');
-- Returns raid_id = 101
-- ... more state transitions and kills ...

-- User presses "End Stream" button
UPDATE stream_sessions SET ended_at = CURRENT_TIMESTAMP WHERE session_id = 42;
```

## Analytics Queries

### Query 1: This Stream vs All-Time Queue Time

```sql_claude
-- Current session stats
SELECT
    'This Stream' as period,
    COUNT(DISTINCT r.raid_id) as total_raids,
    ROUND(AVG(
        SELECT SUM(
            COALESCE(
                (SELECT MIN(t2.transitioned_at)
                 FROM raid_state_transitions t2
                 WHERE t2.raid_id = t1.raid_id
                 AND t2.transitioned_at > t1.transitioned_at),
                r.ended_at
            ) - t1.transitioned_at
        )
        FROM raid_state_transitions t1
        WHERE t1.raid_id = r.raid_id AND t1.to_state = 'queuing'
    ), 2) as avg_queue_seconds
FROM raids r
WHERE r.session_id = 42

UNION ALL

-- All-time stats
SELECT
    'All Time' as period,
    COUNT(DISTINCT r.raid_id) as total_raids,
    ROUND(AVG(
        SELECT SUM(
            COALESCE(
                (SELECT MIN(t2.transitioned_at)
                 FROM raid_state_transitions t2
                 WHERE t2.raid_id = t1.raid_id
                 AND t2.transitioned_at > t1.transitioned_at),
                r.ended_at
            ) - t1.transitioned_at
        )
        FROM raid_state_transitions t1
        WHERE t1.raid_id = r.raid_id AND t1.to_state = 'queuing'
    ), 2) as avg_queue_seconds
FROM raids r;
```

### Query 2: Time Breakdown for Current Session

```sql_claude
-- How much time spent in each state during this stream?
WITH state_durations AS (
    SELECT
        t1.raid_id,
        t1.to_state,
        COALESCE(
            (SELECT MIN(t2.transitioned_at)
             FROM raid_state_transitions t2
             WHERE t2.raid_id = t1.raid_id
             AND t2.transitioned_at > t1.transitioned_at),
            (SELECT ended_at FROM raids WHERE raid_id = t1.raid_id)
        ) - t1.transitioned_at as duration_seconds
    FROM raid_state_transitions t1
    WHERE t1.raid_id IN (
        SELECT raid_id FROM raids WHERE session_id = 42
    )
)
SELECT
    to_state,
    ROUND(SUM(duration_seconds) / 60.0, 1) as total_minutes,
    ROUND(100.0 * SUM(duration_seconds) / (SELECT SUM(duration_seconds) FROM state_durations), 1) as percentage
FROM state_durations
GROUP BY to_state
ORDER BY total_minutes DESC;
```

**Example Result:**
```
to_state          | total_minutes | percentage
------------------|---------------|------------
raid_active       | 90.5          | 45.2%
stash_management  | 45.0          | 22.5%
queuing           | 30.2          | 15.1%
post_raid_review  | 20.0          | 10.0%
pre_raid_setup    | 14.3          | 7.2%
```

### Query 3: Session Summary Stats

```sql_claude
-- Complete session summary for OBS overlay
SELECT
    s.session_id,
    s.started_at,
    s.ended_at,
    COUNT(r.raid_id) as total_raids,
    SUM(CASE WHEN r.current_state = 'survived' THEN 1 ELSE 0 END) as survived_count,
    ROUND(100.0 * SUM(CASE WHEN r.current_state = 'survived' THEN 1 ELSE 0 END) / COUNT(r.raid_id), 1) as survival_rate,
    COUNT(k.kill_id) as total_kills,
    ROUND(CAST(COUNT(k.kill_id) AS FLOAT) / COUNT(r.raid_id), 2) as avg_kills_per_raid,
    SUM(CASE WHEN r.character_type = 'pmc' THEN 1 ELSE 0 END) as pmc_raids,
    SUM(CASE WHEN r.character_type = 'scav' THEN 1 ELSE 0 END) as scav_raids
FROM stream_sessions s
LEFT JOIN raids r ON r.session_id = s.session_id
LEFT JOIN kills k ON k.raid_id = r.raid_id
WHERE s.session_id = 42
GROUP BY s.session_id;
```

### Query 4: Survival Rate Trend (Last 10 Sessions)

```sql_claude
-- Track improvement over time
SELECT
    s.session_id,
    DATE(s.started_at) as date,
    COUNT(r.raid_id) as raids,
    ROUND(100.0 * SUM(CASE WHEN r.current_state = 'survived' THEN 1 ELSE 0 END) / COUNT(r.raid_id), 1) as survival_rate,
    ROUND(CAST(COUNT(k.kill_id) AS FLOAT) / COUNT(r.raid_id), 2) as kd_ratio
FROM stream_sessions s
LEFT JOIN raids r ON r.session_id = s.session_id
LEFT JOIN kills k ON k.raid_id = r.raid_id
GROUP BY s.session_id
ORDER BY s.started_at DESC
LIMIT 10;
```

### Query 5: PVE vs PVP Comparison (Current Session)

```sql_claude
-- Compare PVE and PVP performance this stream
SELECT
    r.game_mode,
    COUNT(r.raid_id) as raids,
    ROUND(100.0 * SUM(CASE WHEN r.current_state = 'survived' THEN 1 ELSE 0 END) / COUNT(r.raid_id), 1) as survival_rate,
    COUNT(k.kill_id) as total_kills,
    ROUND(CAST(COUNT(k.kill_id) AS FLOAT) / COUNT(r.raid_id), 2) as avg_kills_per_raid
FROM raids r
LEFT JOIN kills k ON k.raid_id = r.raid_id
WHERE r.session_id = 42
GROUP BY r.game_mode;
```

### Query 6: Session Overhead Time (Pre-Raid Setup)

```sql_claude
-- Calculate time between session start and first raid start
-- Tracks "stream setup", "just chatting", or menu time before first raid
-- Useful for identifying how much time is spent before actually playing

-- Single session overhead
SELECT
    s.session_id,
    s.started_at as session_start,
    r.started_at as first_raid_start,
    CAST((julianday(r.started_at) - julianday(s.started_at)) * 86400 AS INTEGER) as overhead_seconds,
    ROUND((julianday(r.started_at) - julianday(s.started_at)) * 1440, 1) as overhead_minutes
FROM stream_sessions s
LEFT JOIN raids r ON r.session_id = s.session_id
WHERE s.session_id = 42
  AND r.raid_id = (
      SELECT MIN(raid_id)
      FROM raids
      WHERE session_id = s.session_id
  );

-- All-time average overhead
SELECT
    COUNT(DISTINCT s.session_id) as total_sessions,
    ROUND(AVG(CAST((julianday(first_raid.started_at) - julianday(s.started_at)) * 1440 AS FLOAT)), 1) as avg_overhead_minutes,
    MIN(CAST((julianday(first_raid.started_at) - julianday(s.started_at)) * 1440 AS INTEGER)) as min_overhead_minutes,
    MAX(CAST((julianday(first_raid.started_at) - julianday(s.started_at)) * 1440 AS INTEGER)) as max_overhead_minutes
FROM stream_sessions s
INNER JOIN (
    SELECT session_id, MIN(raid_id) as first_raid_id
    FROM raids
    GROUP BY session_id
) first_raid_lookup ON s.session_id = first_raid_lookup.session_id
INNER JOIN raids first_raid ON first_raid.raid_id = first_raid_lookup.first_raid_id;
```

**Example Result:**
```
total_sessions | avg_overhead_minutes | min_overhead_minutes | max_overhead_minutes
---------------|---------------------|---------------------|---------------------
25             | 18.5                | 3                   | 45
```

**Interpretation:** On average, 18.5 minutes pass between starting a stream session and beginning the first raid. This tracks "dicking around" time in menus, stream setup, or just chatting.

## Rust Data Structures

```rust_claude
use time::PrimitiveDateTime;

// ============================================================
// Enums
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum CharacterType {
    PMC,
    Scav,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum GameMode {
    PVE,
    PVP,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum SessionType {
    Stream,
    Practice,
    Casual,
}

// Note: EnemyType removed - using String for extensibility
// Phase 4 OCR may discover new enemy types (cultist, rogue, bloodhound, etc.)

// ============================================================
// Structs
// ============================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StreamSession {
    pub session_id: i64,
    pub started_at: PrimitiveDateTime,
    pub ended_at: Option<PrimitiveDateTime>,
    pub session_type: Option<SessionType>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Raid {
    pub raid_id: i64,
    pub session_id: Option<i64>,
    pub started_at: PrimitiveDateTime,
    pub ended_at: Option<PrimitiveDateTime>,
    pub map_name: String,
    pub character_type: CharacterType,
    pub game_mode: GameMode,
    pub current_state: String,  // String for extensibility
    pub extract_location: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RaidStateTransition {
    pub transition_id: i64,
    pub raid_id: i64,
    pub from_state: Option<String>,
    pub to_state: String,
    pub transitioned_at: PrimitiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Kill {
    pub kill_id: i64,
    pub raid_id: i64,
    pub killed_at: PrimitiveDateTime,
    pub enemy_type: String,  // String for extensibility (no enum)
    pub weapon_used: Option<String>,
    pub headshot: Option<bool>,
}
```

## CRUD Operations Summary

### Stream Sessions
- `create_session(type, notes)` → session_id
- `end_session(session_id)` → updates ended_at
- `get_current_session()` → latest session without ended_at
- `get_session_stats(session_id)` → aggregate stats

### Raids
- `create_raid(session_id, map, char_type, mode)` → raid_id
- `get_active_raid()` → raid without ended_at
- `finalize_raid(raid_id, final_state, extract_location)` → updates ended_at
- `get_raids_for_session(session_id)` → Vec<Raid>

### State Transitions
- `record_transition(raid_id, from_state, to_state)` → transition_id
- `get_transitions_for_raid(raid_id)` → Vec<RaidStateTransition>
- `get_time_in_state(raid_id, state)` → total_seconds

### Kills
- `add_kill(raid_id, enemy_type, weapon, headshot)` → kill_id
- `get_kills_for_raid(raid_id)` → Vec<Kill>
- `get_kills_for_session(session_id)` → Vec<Kill>

## API Endpoints (Phase 2b Preview)

```rust_claude
// Session management
POST /session/start        { session_type: "stream", notes: "Saturday raids" }
POST /session/end          { session_id: 42 }

// Raid lifecycle
POST /raid/start           { session_id: 42, map_name: "Customs", character_type: "pmc", game_mode: "pvp" }
POST /raid/transition      { raid_id: 100, to_state: "queuing" }
POST /raid/kill            { raid_id: 100, enemy_type: "scav", weapon_used: "AK-74M", headshot: true }
POST /raid/end             { raid_id: 100, final_state: "survived", extract_location: "Crossroads" }

// Stats queries
GET  /stats/session/:id           # Session summary
GET  /stats/session/:id/compare   # This session vs all-time
GET  /stats/current               # Current active session stats
GET  /stats/all-time              # All-time stats
```

## Migration Strategy

### Step 1: Create migration file
`migrations/20241224000000_initial_schema.sql`

### Step 2: Update db initialization
```rust_claude
// In src/main.rs or src/db.rs
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run migrations");
```

### Step 3: Remove old inline CREATE TABLE code
Delete the old `init_schema()` function.

## Testing Strategy

### Unit Tests (src/db.rs)
1. Session lifecycle: create → raids → end
2. Raid lifecycle: create → transitions → kills → finalize
3. State transition recording (including "backwards" transitions)
4. Time calculation queries
5. Stats aggregation queries

### Integration Tests (tests/api_tests.rs)
1. Full workflow: Start session → multiple raids → end session
2. Stream Deck simulation: button presses in realistic order
3. Edge cases: queue cancel, reconnect, game crash

## Next Steps

1. ✅ Schema design complete
2. ⬜ Create migration file with this schema
3. ⬜ Implement Rust structs in `src/models.rs`
4. ⬜ Implement CRUD operations in `src/db.rs`
5. ⬜ Write unit tests
6. ⬜ Phase 2b: Build REST API endpoints
