# Phase 2b: REST API Plan

## Overview
This phase builds the Web API layer using **Axum**. This API will serve as the backend for:
1.  **Manual Control Dashboard:** A web interface for you to manually start raids, log kills, and change states.
2.  **Stream Deck:** Buttons to trigger quick actions (e.g., "Start Raid").
3.  **OBS Overlays:** Fetching live stats (K/D, Survival Rate) to display on stream.

## Technology Stack
- **Framework:** `axum` (Ergonomic, modular, built on Tokio).
- **Serialization:** `serde` + `serde_json` (JSON handling).
- **State Management:** `Arc<SqlitePool>` shared across threads.
- **CORS:** `tower-http` (To allow the frontend to talk to the backend).

## API Endpoints

### 1. System & Health
| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/health` | Returns 200 OK if server is up and DB is connected. |
| `GET` | `/version` | Returns current app version. |

### 2. Session Management
*Context: The "Stream" itself.*

| Method | Path | Body (JSON) | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/session/current` | - | Returns the currently active session (or 404). |
| `POST` | `/api/session` | `{ "type": "stream", "notes": "..." }` | Starts a new streaming session. |
| `POST` | `/api/session/end` | - | Ends the current session. |

### 3. Raid Lifecycle (The "State Machine")
*Context: Tracking what is happening in Tarkov right now.*

| Method | Path | Body (JSON) | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/raid/current` | - | Returns active raid + current state (e.g., "in_raid"). |
| `POST` | `/api/raid` | `{ "map": "customs", "mode": "pmc" }` | Starts a new raid. Auto-transitions to "queue". |
| `POST` | `/api/raid/transition` | `{ "state": "in_raid", "timestamp": "..." }` | Moves state (Queue -> In Raid -> Death). |
| `PUT` | `/api/raid/:id` | `{ "extract": "Dorms V-Ex", "outcome": "survived" }` | Updates raid details (e.g., extract location). |
| `POST` | `/api/raid/end` | - | Finalizes the raid (sets `ended_at`). |

### 4. Kill Management
*Context: Usually entered post-raid or during quiet moments.*

| Method | Path | Body (JSON) | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/raid/:id/kills` | - | List all kills for a specific raid. |
| `POST` | `/api/raid/:id/kills` | `{ "enemy": "pmc", "weapon": "M4", "headshot": true }` | Adds a kill. |
| `DELETE` | `/api/kills/:id` | - | Removes a kill (if entered by mistake). |

### 5. Stats & Analytics (Read-Only)
*Context: For OBS Browser Sources or the Dashboard.*

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/stats/session` | Summary of current session (Raids, SR%, K/D, Total Kills). |
| `GET` | `/api/stats/raid/:id` | Specific raid report. |

## Data Structures (JSON)

**Start Raid Request:**
```json
{
  "map_name": "interchange",
  "character_type": "pmc", // or "scav"
  "game_mode": "pvp"       // or "pve"
}
```

**Add Kill Request:**
```json
{
  "enemy_type": "pmc",
  "weapon_used": "M4A1",
  "headshot": true,
  "timestamp": "2026-01-10T14:30:00Z" // Optional, defaults to now
}
```

## Implementation Steps (Plan)
1.  **Dependencies:** Add `axum`, `serde`, `serde_json`, `tower-http` to `Cargo.toml`.
2.  **Server Setup:** Create `src/api.rs` to configure the Axum router and middleware.
3.  **Handlers:** Implement handler functions for each endpoint group (Session, Raid, Kills).
4.  **State Injection:** Pass the `SqlitePool` to handlers via Axum's `State` extractor.
5.  **Integration:** Run the server in `main.rs` alongside the main application logic.

## Future Web Interface (Preview)
For the manual input interface, we can serve a static **HTML/JS** Single Page Application (SPA) from a `/web` folder directly via this API.
- **Dashboard:** Shows "Current Status" (Queueing, In Raid).
- **Control Panel:** Big buttons for "Start Raid", "Died", "Extracted".
- **Kill Logger:** A form to quickly tap "Scav + Headshot" or "PMC + Thorax".
