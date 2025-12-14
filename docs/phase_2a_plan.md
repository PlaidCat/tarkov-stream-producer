# Phase 2a: Data Structure Planning - Required Data Documentation

## Task
Document required data structures for Tarkov Stream Producer tracking system.

## User Context
- Building a system to track Escape from Tarkov gameplay stats
- Manual control via REST API and Stream Deck
- Display stats in OBS overlays
- Track queue times and character type (PMC vs SCAV)

## Core Data Entities

### 1. Raid Entity
Tracks individual raid sessions from queue to extraction.

**Fields:**
- `raid_id` - Primary key (integer/UUID)
- `started_at` - Timestamp when raid started (queue entered)
- `ended_at` - Timestamp when raid ended (nullable for in-progress raids)
- `map_name` - String (e.g., "Customs", "Woods", "Shoreline")
- `character_type` - Enum: "pmc" or "scav"
- `game_mode` - Enum: "pve" or "pvp" (affects difficulty/stats tracking)
- `queue_time_seconds` - Integer (time from queue start to match found)
- `status` - String: "queuing", "in_progress", "survived", "died", "mia", etc. (extensible for future states)
- `extract_location` - Optional string (where player extracted)
- `stash_time_seconds` - Optional integer (time in inventory/stash management before raid)
- `review_time_seconds` - Optional integer (time viewing post-raid screens)

**Derived/Computed:**
- `kill_count` - COUNT of associated kills (via SQL query)
- `raid_duration` - `ended_at - started_at`

**Notes:**
- `survived` is redundant - derived from `status == "survived"`
- Status covers all end states: survived, died, MIA
- Queue time measured from entering queue to loading map
- **PVE vs PVP tracking**: `game_mode` allows separate stat analysis (PVE has no player kills)
- **Extensible states**: `status` field intentionally flexible to accommodate discovered states (e.g., "transfer", "reconnecting", "deploying", etc.)
- **Time tracking granularity**: Optional stash/review times for detailed session analysis

### 2. Kill Entity
Tracks individual kills within a raid.

**Fields:**
- `kill_id` - Primary key (integer/UUID)
- `raid_id` - Foreign key to Raid
- `killed_at` - Timestamp
- `enemy_type` - Enum: "scav", "pmc", "boss", "raider"
- `weapon_used` - Optional string
- `headshot` - Optional boolean

**Relationships:**
- Many kills belong to one raid
- Cascade delete when raid is deleted

### 3. Display Stats (Derived)
What OBS overlays should show (computed from database):

**Current Raid Stats:**
- Kills this raid
- Character type (PMC/SCAV)
- Raid duration
- Map name

**Session Stats (today/stream):**
- Total raids
- Total kills
- Survival rate %
- Average queue time

**All-Time Stats:**
- Total raids
- Total kills
- K/D ratio
- Favorite map (most played)
- PMC vs SCAV ratio
- PVE vs PVP breakdown (raids, survival rate, avg kills)
- Time in stash management vs in-raid time

## API Endpoints (Preview)
Based on these entities, the API will need:

- `POST /raid/start` - Create raid (queuing state)
  - Body: `{ character_type: "pmc"|"scav", game_mode: "pve"|"pvp", map_name: string }`

- `POST /raid/matched` - Update raid when match found
  - Records queue_time, changes status to "in_progress"

- `POST /raid/kill` - Add kill to current raid
  - Body: `{ enemy_type: string, weapon_used?: string, headshot?: bool }`

- `POST /raid/end` - Finalize raid
  - Body: `{ status: "survived"|"died"|"mia", extract_location?: string }`

- `GET /stats/current` - Current raid stats
- `GET /stats/session` - Session stats
- `GET /stats/all-time` - All-time stats

## Database Schema (SQLite)

```sql
-- Raids table
CREATE TABLE raids (
    raid_id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP,
    map_name TEXT NOT NULL,
    character_type TEXT NOT NULL CHECK(character_type IN ('pmc', 'scav')),
    game_mode TEXT NOT NULL CHECK(game_mode IN ('pve', 'pvp')),
    queue_time_seconds INTEGER,
    status TEXT NOT NULL,  -- No CHECK constraint = extensible for future states
    extract_location TEXT,
    stash_time_seconds INTEGER,      -- Optional: time in inventory management
    review_time_seconds INTEGER       -- Optional: time viewing post-raid screens
);

-- Kills table
CREATE TABLE kills (
    kill_id INTEGER PRIMARY KEY AUTOINCREMENT,
    raid_id INTEGER NOT NULL,
    killed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enemy_type TEXT NOT NULL CHECK(enemy_type IN ('scav', 'pmc', 'boss', 'raider')),
    weapon_used TEXT,
    headshot BOOLEAN,
    FOREIGN KEY (raid_id) REFERENCES raids(raid_id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX idx_raids_status ON raids(status);
CREATE INDEX idx_raids_started_at ON raids(started_at);
CREATE INDEX idx_raids_game_mode ON raids(game_mode);
CREATE INDEX idx_kills_raid_id ON kills(raid_id);
```

## Next Steps
1. Review and finalize data structure with user
2. Create Rust structs with sqlx derives
3. Write database migration files
4. Implement CRUD operations

## Extensibility Strategy

### State Machine Design
The `status` field is intentionally **flexible** (no CHECK constraint) to support discovery of new states during development and OCR implementation (Phase 4).

**Known States (Initial):**
- `idle` - Main menu, not actively playing
- `stash_management` - Character/Traders/Hideout/Flea ("stash tetris")
- `pre_raid_setup` - Character select → Map select → Insurance
- `deploying_cancellable` - Loading screens (can back out)
- `deploying_committed` - Past point of no return (PVE: "Starting Local Game", PVP: after matching)
- `raid_active` - In-game FPS gameplay
- `raid_ending` - Transfer screen or end screen appearing
- `post_raid_review` - Kill List → Statistics → Experience screens
- `survived` - Terminal state (raid ended successfully)
- `died` - Terminal state (player died)
- `mia` - Terminal state (missing in action)

**Future States (To Be Discovered):**
- `transfer` - Map-to-map movement within raid
- `reconnecting` - Connection lost, attempting reconnect
- Additional conditional states discovered during gameplay

### State Transition Validation
Application-level validation will enforce valid state transitions, with logging for unexpected transitions to aid in discovering new states:

```rust
// Example state transition logic
if transitions.can_transition(&current_state, &new_state) {
    current_state = new_state;
} else {
    warn!("Unexpected transition: {:?} -> {:?}", current_state, new_state);
    // Log for investigation, optionally allow in discovery mode
}
```

### Discovery Mode
During Phase 4 (OCR implementation), unknown states will be automatically detected and logged:
- Screenshot saved for later labeling
- State logged with detected text hints
- Added to training data for future model updates

## Query Examples

### PVE vs PVP Survival Rate
```sql
SELECT
    game_mode,
    COUNT(*) as total_raids,
    SUM(CASE WHEN status = 'survived' THEN 1 ELSE 0 END) as survived,
    ROUND(100.0 * SUM(CASE WHEN status = 'survived' THEN 1 ELSE 0 END) / COUNT(*), 2) as survival_rate
FROM raids
GROUP BY game_mode;
```

### Average Kills by Game Mode
```sql
SELECT
    r.game_mode,
    AVG(COALESCE(k.kill_count, 0)) as avg_kills
FROM raids r
LEFT JOIN (
    SELECT raid_id, COUNT(*) as kill_count
    FROM kills
    GROUP BY raid_id
) k ON r.raid_id = k.raid_id
WHERE r.status IN ('survived', 'died', 'mia')
GROUP BY r.game_mode;
```

### Time Breakdown Analysis
```sql
SELECT
    SUM(stash_time_seconds) / 3600.0 as hours_in_stash,
    SUM(queue_time_seconds) / 3600.0 as hours_queuing,
    SUM(COALESCE(ended_at - started_at, 0)) / 3600.0 as hours_in_raids
FROM raids;
```

## Open Questions
- Track loot/money extracted? (adds complexity - defer to later phase?)
- Track deaths/cause of death details? (covered by status for now)
- Track player loadout/gear? (defer to later phase?)
- Squad tracking for team raids? (defer to later phase?)
- Track map transfers as separate raids or single raid with multiple maps? (TBD)
