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
- `queue_time_seconds` - Integer (time from queue start to match found)
- `status` - Enum: "queuing", "in_progress", "survived", "died", "mia"
- `extract_location` - Optional string (where player extracted)

**Derived/Computed:**
- `kill_count` - COUNT of associated kills (via SQL query)
- `raid_duration` - `ended_at - started_at`

**Notes:**
- `survived` is redundant - derived from `status == "survived"`
- Status covers all end states: survived, died, MIA
- Queue time measured from entering queue to loading map

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

## API Endpoints (Preview)
Based on these entities, the API will need:

- `POST /raid/start` - Create raid (queuing state)
  - Body: `{ character_type: "pmc"|"scav", map_name: string }`

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
    queue_time_seconds INTEGER,
    status TEXT NOT NULL CHECK(status IN ('queuing', 'in_progress', 'survived', 'died', 'mia')),
    extract_location TEXT
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
CREATE INDEX idx_kills_raid_id ON kills(raid_id);
```

## Next Steps
1. Review and finalize data structure with user
2. Create Rust structs with sqlx derives
3. Write database migration files
4. Implement CRUD operations

## Open Questions
- Track loot/money extracted? (adds complexity - defer to later phase?)
- Track deaths/cause of death details? (covered by status for now)
- Track player loadout/gear? (defer to later phase?)
- Squad tracking for team raids? (defer to later phase?)
