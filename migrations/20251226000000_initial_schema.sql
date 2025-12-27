-- ============================================================
-- Stream Sessions Table
-- ============================================================
CREATE TABLE stream_sessions (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    session_type TEXT CHECK(session_type IN ('stream', 'practice', 'casual')),
    notes TEXT
);

CREATE INDEX idx_sessions_started_at ON stream_sessions(started_at);

-- ============================================================
-- Raids Table
-- ============================================================
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

CREATE INDEX idx_raids_session_id ON raids(session_id);
CREATE INDEX idx_raids_started_at ON raids(started_at);
CREATE INDEX idx_raids_current_state ON raids(current_state);
CREATE INDEX idx_raids_games_mode ON raids(game_mode);

-- ============================================================
-- Raid State Transitions Table
-- ============================================================
CREATE TABLE raid_state_transitions (
    transition_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    from_state TEXT,
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
CREATE TABLE kills (
    kill_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    killed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enemy_type TEXT NOT NULL,
    weapon_used TEXT,
    headshot BOOLEAN,

    FOREIGN KEY (raid_id) REFERENCES raids(raid_id) ON DELETE CASCADE
);

CREATE INDEX idx_kills_raid_id ON kills(raid_id);
CREATE INDEX idx_kills_enemy_type ON kills(enemy_type);
