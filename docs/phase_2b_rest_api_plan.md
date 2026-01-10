# REST API Implementation Plan - Phase 2b

## Overview
Build a REST API with Axum framework and simple HTML forms using Askama templates for manual Tarkov gameplay tracking. Local-only (127.0.0.1), no authentication required.

## Technology Stack
- **Web Framework**: Axum 0.7 (tokio-native, minimal boilerplate)
- **Templating**: Askama 0.12 + askama_axum 0.4
- **Serialization**: serde + serde_json
- **Validation**: validator 0.18

## Module Structure

```
src/
├── main.rs                 # Axum server setup
├── models.rs               # Existing (no changes)
├── db.rs                   # Existing (no changes)
├── api/
│   ├── mod.rs             # Module exports
│   ├── routes.rs          # Router configuration
│   ├── state.rs           # AppState (wraps SqlitePool)
│   ├── error.rs           # AppError with HTTP status mapping
│   ├── dto.rs             # Request/response models
│   └── handlers/
│       ├── mod.rs
│       ├── session.rs     # Session lifecycle
│       ├── raid.rs        # Raid lifecycle + state transitions
│       ├── kill.rs        # Single + batch kill tracking
│       └── stats.rs       # Aggregation queries
└── web/
    ├── mod.rs
    ├── routes.rs          # HTML form routes
    ├── handlers.rs        # Form submissions
    └── templates/
        ├── layout.html    # Base template
        ├── index.html     # Dashboard
        ├── raid_start.html
        ├── kill_form.html
        └── stats.html
```

## Dependencies to Add

```toml
[dependencies]
# Web framework
axum = "0.7"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Templating
askama = { version = "0.12", features = ["with-axum"] }
askama_axum = "0.4"

# Validation
validator = { version = "0.18", features = ["derive"] }

# HTTP types
http = "1.0"
```

## API Endpoints

### Session Management
- `POST /api/session` - Create session (body: `{session_type, notes?}`)
- `GET /api/session/current` - Get active session + stats
- `POST /api/session/end` - End active session

### Raid Lifecycle
- `POST /api/raid` - Start raid (body: `{map_name, character_type, game_mode}`)
- `GET /api/raid/current` - Get active raid with kill count
- `POST /api/raid/transition` - Change state (body: `{raid_id?, to_state, timestamp?}`)
- `POST /api/raid/end` - End raid (body: `{raid_id?, extract_location?, timestamp?}`)

### Kill Tracking
- `POST /api/raid/:raid_id/kills` - Add single kill
- `POST /api/raid/current/kills/batch` - Add multiple kills (body: `{kills: [...]}`)
- `GET /api/raid/:raid_id/kills` - List kills for raid

### Stats
- `GET /api/stats/session/current` - Current session aggregations
- `GET /api/stats/raid/:raid_id` - Individual raid details with durations

### System
- `GET /health` - Health check (returns 200 + DB status)

## Error Handling Pattern

```rust
// src/api/error.rs
pub enum AppError {
    DatabaseError(sqlx::Error),     // 500
    NotFound(String),                // 404
    Conflict(String),                // 409 (e.g., active raid exists)
    ValidationError(String),         // 422
    BadRequest(String),              // 400
}

// Implements IntoResponse for HTTP mapping
// Returns JSON: { "error": { "code": "...", "message": "..." } }
```

## State Management

```rust
// src/api/state.rs
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

// Used via Axum extraction in handlers
async fn handler(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let result = db::operation(&state.pool).await?;
    // ...
}
```

## Web UI Structure

### Dashboard (`templates/index.html`)
- Show active session (type, start time) OR session start form
- Show active raid (map, character, state, kills) OR raid start link
- Quick actions: state transition buttons, end raid button

### Raid Start Form (`templates/raid_start.html`)
- Select map (dropdown)
- Select character type: PMC/Scav (radio buttons)
- Select game mode: PVE/PVP (radio buttons)
- Submit -> POST /api/raid

### Kill Entry Form (`templates/kill_form.html`)
- Dynamic form for batch kill entry
- Each row: enemy type (dropdown), weapon (text), headshot (checkbox)
- "Add Kill" button (JavaScript adds rows)
- Submit all -> POST /api/raid/current/kills/batch

### Stats Page (`templates/stats.html`)
- Current session stats (raids, survival rate, K/D, avg kills)
- Recent raids list with click-through to details

## Implementation Sequence

### Phase 1: Core Infrastructure (2-3h)
1. Add dependencies to Cargo.toml
2. Create module directories and mod.rs files
3. Implement `AppState` and `AppError` with HTTP mapping
4. Update main.rs: Axum server setup, router mounting
5. Implement GET /health endpoint
6. Test: `cargo run`, curl http://127.0.0.1:3000/health

### Phase 2: Session Endpoints (1.5h)
1. Define DTOs: CreateSessionRequest, SessionResponse
2. Implement handlers::session::{create, get_current, end}
3. Wire routes in api/routes.rs
4. Test with curl: full session lifecycle

### Phase 3: Raid Endpoints (2.5h)
1. Define DTOs: CreateRaidRequest, StateTransitionRequest, EndRaidRequest, RaidResponse
2. Implement handlers::raid::{create, get_current, transition, end}
3. Wire routes
4. Test: start raid -> transitions -> end raid flow

### Phase 4: Kill Endpoints (1.5h)
1. Define DTOs: AddKillRequest, BatchKillsRequest, KillResponse
2. Implement handlers::kill::{create, batch_create, list}
3. Wire routes (including path parameter :raid_id)
4. Test: single kill, batch kills, retrieve kills

### Phase 5: Stats Endpoints (1.5h)
1. Add aggregation queries to db.rs (or new stats module)
2. Define DTOs: SessionStatsResponse, RaidStatsResponse
3. Implement handlers::stats::{current_session, raid_detail}
4. Test: verify calculations match expected values

### Phase 6: Web UI (3-4h)
1. Set up templates/ directory, configure Askama
2. Create layout.html (base template with nav)
3. Implement dashboard: show session/raid status, action buttons
4. Implement raid start form
5. Implement kill entry form with JavaScript for dynamic rows
6. Add basic CSS for usability
7. Test: manual workflow in browser

### Phase 7: Integration Testing (1-2h)
1. Write integration tests for endpoint behavior
2. Test error scenarios (no active session, duplicate raid)
3. Verify 50% coverage target
4. Run cargo tarpaulin

### Phase 8: Documentation (1h)
1. Update CLAUDE.md with API usage examples
2. Document Stream Deck button configuration
3. Add curl examples for common operations
4. User guide for manual kill entry workflow

**Total Estimated Time**: 14-18 hours

## Stream Deck Integration

Stream Deck HTTP Request buttons can POST to endpoints:

**Example: Start Customs PMC Raid**
```
URL: http://127.0.0.1:3000/api/raid
Method: POST
Content-Type: application/json
Body: {"map_name":"customs","character_type":"pmc","game_mode":"pve"}
```

**Example: Quick Kill - Scav**
```
URL: http://127.0.0.1:3000/api/raid/current/kills
Method: POST
Body: {"enemy_type":"scav"}
```

## Testing Strategy

- **Unit tests**: DTOs, validation, error mapping
- **Integration tests**: Full endpoint behavior with test database
- **Manual testing**: Web UI forms, Stream Deck buttons
- **Coverage target**: 50% (existing project standard)

Test pattern:
```rust
#[tokio::test]
async fn test_raid_lifecycle() {
    let pool = setup_test_db().await;
    let state = AppState::new(pool);
    let app = api_router().with_state(state);

    // POST /api/session
    // POST /api/raid
    // POST /api/raid/transition
    // POST /api/raid/:id/kills
    // POST /api/raid/end
    // GET /api/stats/session/current

    // Assert response codes and database state
}
```

## Critical Files

1. **src/main.rs** - Refactor for Axum server setup
2. **src/api/error.rs** - Core error type with HTTP status mapping
3. **src/api/dto.rs** - Request/response models
4. **src/api/routes.rs** - Router configuration
5. **src/api/handlers/raid.rs** - Most complex handler (raid lifecycle + state transitions)
6. **src/web/templates/kill_form.html** - Most complex template (dynamic batch entry)

## Design Decisions

### Why Axum?
- Tokio-native (matches existing async runtime)
- Minimal boilerplate, modern ergonomics
- Type-safe extractors, excellent error handling
- Growing ecosystem

### Why Separate DTOs?
- API versioning independent of schema
- Aggregate responses from multiple tables
- Validation requirements differ from DB constraints
- Clean separation: API contracts vs. DB models

### Why Both Single and Batch Kill Endpoints?
- Single: Stream Deck use case (real-time logging during raid)
- Batch: Web form use case (end-of-raid manual entry)
- Flexibility for different workflows

### Why Simple HTML Forms vs. SPA?
- MVP goal: functional manual input
- No build step complexity
- Easier to maintain alongside Rust backend
- Can upgrade to SPA later if needed

## Verification Steps

After implementation:

1. **Health Check**: `curl http://127.0.0.1:3000/health` → 200 OK
2. **Session Lifecycle**: Start session → create raid → end raid → end session (via curl)
3. **Kill Entry**: POST batch kills → GET raid kills → verify count/details
4. **Stats Accuracy**: Known data → query stats → manual calculation match
5. **Web UI Flow**: Browser → start session → start raid → log kills → view stats
6. **Stream Deck**: Configure button → trigger raid start → verify in DB
7. **Error Scenarios**: Duplicate raid → 409, invalid raid_id → 404, no session → 400
8. **Test Coverage**: `cargo tarpaulin` → ≥50%

## Future Enhancements (Out of Scope)

- WebSocket for real-time dashboard updates
- Advanced stats (per-map, time-based trends)
- Data export (CSV/JSON)
- Dark mode for web UI
- Stream Deck plugin for richer integration
- Authentication (if exposing over network)
