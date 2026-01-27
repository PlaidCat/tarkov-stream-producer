# Tarkov Stream Producer - Project Plan

This document outlines the development plan for the Tarkov Stream Producer application.
**Note:** Each task is designed to be 1-2 hours of implementation time. Track actual time in `.time_tracking.md`.

## Phase 1: Project Foundation ✅

- [x] Set up a new Rust project using Cargo.
- [x] Choose and integrate a logging framework.
- [x] Choose and integrate a testing framework and set up a basic test.
- [x] Set up `cargo-tarpaulin` for code coverage analysis.
- [x] Establish a basic CI/CD pipeline (e.g., using GitHub Actions) to build and test on both Linux and Windows.
- [x] **Database Integration:** Choose and integrate a Rust SQL library/ORM (`sqlx` with SQLite).
- [x] **Database Integration:** Create basic connection and unit tests.

### Phase 1.b: Windows Debug Executable Production ✅ (Completed: 2025-12-08, 0.5h actual)
- [x] Add `[profile.release]` with `debug = true` to Cargo.toml (0.25h)
- [x] Create `.github/workflows/release.yml` for tagged release builds (0.5h)
- [x] Test workflow by creating and pushing a version tag (0.25h)
- [x] Verify Windows executable artifact is produced and downloadable (0.25h)
- [x] Document completion in CLAUDE.md (0.25h)

**Note:** See `docs/phase_1b_plan.md` for detailed implementation plan.

## Phase 2: Core Data & API Foundation

**Focus:** Build essential tracking system with manual control via REST API. Chat bot integration deferred to Phase 5 due to unmaintained dependencies.

**Architecture Decision (2025-12-24):** Expanded from 2-table to 4-table design with session tracking and state transitions. See `docs/phase_2a_complete_schema.md` for full schema.

### Phase 2a: Planning & Schema Design ✅ (Completed: 2025-12-24)
- [x] Document required data (raid stats, kills, player info)
- [x] Design Raid and Kill structs with relationships
- [x] Expand to 4-table design with sessions and state transitions
- [x] Create complete database schema documentation (see `docs/phase_2a_complete_schema.md`)
- [x] Design Rust data structures (enums + structs for 4 tables)
- [x] Design analytics queries (session comparisons, time breakdowns)

### Phase 2a (Core): Implementation (7-10 hours total, revised from 2.5-3.5h)
- [x] Create `migrations/20251226000000_initial_schema.sql` with 4-table schema (0.25h est, ~1.3h actual)
- [x] Create `src/models.rs` with all structs and enums (0.75-1h, revised from 0.5h)
- [x] Update `src/db.rs` to use `sqlx::migrate!()` instead of inline schema (0.5h, revised from 0.25h)
- [x] Implement CRUD for StreamSession (1-1.5h, revised from 0.5h)
- [x] Implement CRUD for Raid (1-1.5h, revised from 0.75h)
- [x] Implement CRUD for RaidStateTransition (0.75-1h, revised from 0.5h) - Create/Log implemented
- [x] Implement CRUD for Kill (0.5-0.75h)
- [x] Write basic unit tests for all CRUD operations (1.5-2h, revised from 1h) - Session & Raid lifecycle tests added

### Phase 2a-Extended: Analytics & Time Tracking ✅ COMPLETED (4-5 hours total, revised)
- [x] Add database helper functions to src/db.rs (0.5h) - COMPLETED 2026-01-18
  - get_session_by_id()
  - get_all_sessions()
  - get_first_raid_for_session()
- [x] Implement `calculate_time_before_first_raid()` function (0.5h) - COMPLETED 2026-01-18
  - Calculate average "dead time" before first raid across all sessions
  - Track most recent session separately
  - Fixed compilation errors in stats.rs (removed unused HashMap import, variables)
- [x] Write test for `calculate_time_before_first_raid()` (0.5h) - COMPLETED 2026-01-18
  - Used optional timestamp parameters (follows existing pattern)
  - 3 sessions with 10min, 30min, 5min delays
  - Verified average (15min) and session count calculations
- [x] Code cleanup and consistency fixes (0.3h) - COMPLETED 2026-01-18
  - Fixed error handling: replaced .expect() with ? in test functions
  - Fixed typos in error messages
  - Removed unused imports and variables
- [x] Implement `calculate_time_in_state()` function (0.75h) - COMPLETED 2026-01-20
  - HashMap-based accumulation of durations between state transitions
  - Handle multiple visits to same state using Entry API
  - Convert to Vec<StateTime> for return
- [x] Write comprehensive test for state transitions (0.75h) - COMPLETED 2026-01-20
  - Realistic 7-state raid flow (pre_raid_setup → survived)
  - 3 scav kills during raid
  - Full duration assertions for all states
- [x] Implement session comparison queries (0.5h) - COMPLETED 2026-01-24
  - "This stream vs all-time" stats - compare_session_to_mode_global()
  - PVE vs PVP comparisons - get_mode_stats_for_session()
  - Tests: test_session_comparison(), test_game_mode_filtering()
- [x] Write test for edge cases (0.75h) - COMPLETED 2026-01-27
  - Test "backwards" transitions (queue → stash) - test_backwards_transition_queue_cancel() ✅
  - Test reconnect during raid - test_reconnect_during_raid() ✅
- [x] Implement `calculate_time_between_raids()` function (0.5h) - COMPLETED 2026-01-27
  - Calculate average stash time between raids within a session
  - Track shortest/longest gaps for analytics
  - Measure "time wasted" reorganizing gear between raids
  - BetweenRaidsTime struct with avg_gap, shortest_gap, longest_gap, gap_count
  - Smart filtering: skips active raids, cross-session gaps, negative gaps
- [x] Write test for `calculate_time_between_raids()` (0.5h) - COMPLETED 2026-01-27
  - test_calculate_time_between_raids() - 4 raids with varying gaps
  - test_calculate_time_between_raids_with_active_raid() - handles active raids
  - test_calculate_time_between_raids_edge_cases() - 0/1 raid, all active
  - test_calculate_time_between_raids_global() - cross-session aggregation
  - test_calculate_time_between_raids_negative_gap() - overlapping data handling

### Phase 2b: REST API with Web UI (14-18 hours total)
**Note:** Axum framework with Askama templates for manual control and web-based kill entry.
**See:** `docs/phase_2b_rest_api_plan.md` for complete implementation details.

#### Phase 2b.1: Core Infrastructure (2-3 hours)
- [ ] Add dependencies to Cargo.toml (0.25h)
  - axum, tower, tower-http, serde, serde_json, askama, askama_axum, validator, http
- [ ] Create module structure: src/api/, src/web/ (0.25h)
- [ ] Implement src/api/state.rs - AppState wrapper for SqlitePool (0.5h)
- [ ] Implement src/api/error.rs - AppError with HTTP status mapping (1h)
  - DatabaseError (500), NotFound (404), Conflict (409), ValidationError (422), BadRequest (400)
- [ ] Update src/main.rs - Axum server setup, router mounting (1h)
- [ ] Implement GET /health endpoint (0.25h)
- [ ] Test: `cargo run`, curl health check (0.25h)

#### Phase 2b.2: Session Endpoints (1.5 hours)
- [ ] Define session DTOs in src/api/dto.rs (0.25h)
  - CreateSessionRequest, SessionResponse
- [ ] Implement src/api/handlers/session.rs (1h)
  - POST /api/session - create session
  - GET /api/session/current - get active session
  - POST /api/session/end - end session
- [ ] Wire routes in src/api/routes.rs (0.25h)
- [ ] Test with curl: full session lifecycle (0.25h)

#### Phase 2b.3: Raid Endpoints (2.5 hours)
- [ ] Define raid DTOs in src/api/dto.rs (0.5h)
  - CreateRaidRequest, StateTransitionRequest, EndRaidRequest, RaidResponse
- [ ] Implement src/api/handlers/raid.rs (1.5h)
  - POST /api/raid - start raid
  - GET /api/raid/current - get active raid
  - POST /api/raid/transition - change state
  - POST /api/raid/end - end raid
- [ ] Wire routes in src/api/routes.rs (0.25h)
- [ ] Test: start raid → transitions → end raid flow (0.5h)

#### Phase 2b.4: Kill Endpoints (1.5 hours)
- [ ] Define kill DTOs in src/api/dto.rs (0.25h)
  - AddKillRequest, BatchKillsRequest, KillResponse
- [ ] Implement src/api/handlers/kill.rs (1h)
  - POST /api/raid/:raid_id/kills - add single kill
  - POST /api/raid/current/kills/batch - add multiple kills
  - GET /api/raid/:raid_id/kills - list kills for raid
- [ ] Wire routes with path parameters (0.25h)
- [ ] Test: single kill, batch kills, retrieve kills (0.25h)

#### Phase 2b.5: Stats Endpoints (1.5 hours)
- [ ] Add aggregation queries to src/db.rs (0.5h)
  - Session stats (total raids, survival rate, K/D)
  - Raid details with state durations
- [ ] Define stats DTOs in src/api/dto.rs (0.25h)
  - SessionStatsResponse, RaidStatsResponse
- [ ] Implement src/api/handlers/stats.rs (0.5h)
  - GET /api/stats/session/current - current session aggregations
  - GET /api/stats/raid/:raid_id - individual raid details
- [ ] Test: verify calculations match expected values (0.25h)

#### Phase 2b.6: Web UI (3-4 hours)
- [ ] Set up src/web/templates/ directory, configure Askama (0.25h)
- [ ] Create layout.html base template with navigation (0.5h)
- [ ] Implement dashboard (index.html) (1h)
  - Show active session status OR session start form
  - Show active raid details OR raid start link
  - Quick action buttons for state transitions
- [ ] Implement raid start form (raid_start.html) (0.5h)
  - Map selection, character type, game mode
- [ ] Implement kill entry form (kill_form.html) (1h)
  - Dynamic batch entry with JavaScript
  - Enemy type, weapon, headshot fields
- [ ] Add basic CSS styling for usability (0.5h)
- [ ] Test: manual workflow in browser (0.5h)

#### Phase 2b.7: Integration Testing (1-2 hours)
- [ ] Write integration tests for endpoint behavior (1h)
  - Test full raid lifecycle with state transitions
  - Test batch kill creation
  - Test stats calculations
- [ ] Test error scenarios (0.5h)
  - No active session, duplicate raid, invalid raid_id
- [ ] Run cargo tarpaulin, verify 50% coverage target (0.25h)

#### Phase 2b.8: Documentation (1 hour)
- [ ] Update CLAUDE.md with API usage examples (0.25h)
- [ ] Document Stream Deck button configuration (0.25h)
  - Example HTTP request payloads for common operations
- [ ] Add curl examples for common operations (0.25h)
- [ ] User guide for manual kill entry workflow (0.25h)

### Phase 2c: Stream Deck Integration (3-4 hours total)
- [ ] Research Stream Deck HTTP request capabilities (1h)
- [ ] Document button layout and API mappings in stream_deck_integration.md (1h)
- [ ] Configure Stream Deck buttons to call API endpoints (1h)
- [ ] Test end-to-end: Stream Deck → API → Database (1h)

## Phase 3: OBS Integration

### OBS Display (3-4 hours total)
- [ ] Research OBS integration methods (text files vs obs-websocket) (1h)
- [ ] Decide on approach and document in obs_integration.md (0.5h)
- [ ] Implement stats output system (text files or WebSocket client) (2h)
- [ ] Create example OBS scene with stat overlays (1h)
- [ ] Test full flow: Stream Deck → API → Database → OBS display (1h)

### End-to-End Testing (2 hours total)
- [ ] Test complete manual flow with all components (1h)
- [ ] Document known issues and future improvements (0.5h)
- [ ] Create user guide for operating the system (0.5h)

## Phase 4: Automated Screen Analysis

### Screen Capture (3-4 hours total)
- [ ] Research cross-platform screen capture libraries (1h)
- [ ] Implement basic screen capture functionality (2h)
- [ ] Add tests and verify performance (1h)

### OCR/Vision (6-8 hours total)
- [ ] Research OCR options (Tesseract, custom models) (1.5h)
- [ ] Integrate OCR library and test text extraction (2h)
- [ ] Implement raid start detection logic (1.5h)
- [ ] Implement kill detection logic (1.5h)
- [ ] Implement raid end detection logic (1.5h)
- [ ] Tune accuracy and handle edge cases (2h)

### Automation (2-3 hours total)
- [ ] Replace manual triggers with automated event detection (1.5h)
- [ ] Add confidence thresholds and fallback to manual mode (1h)
- [ ] Test automation with real gameplay (1h)

## Phase 5: Chat Bot Integration (Deferred)

**Note:** Chat bot integration deferred until better maintained libraries are available.

### Twitch Bot Integration (4-6 hours total) - DEFERRED
- [ ] Evaluate current state of Rust Twitch libraries
- [ ] Choose approach: twitch-irc (unmaintained), direct IRC, or Twitch API
- [ ] Implement basic connection and message handling
- [ ] Add command parsing (!stats, !kd, !raid)
- [ ] Integrate with existing database/API
- [ ] Write unit tests for command parsing

### YouTube Chat Integration (4-6 hours total) - DEFERRED
- [ ] Research YouTube Live Chat API
- [ ] Implement REST API polling for chat messages
- [ ] Add command parsing and response system
- [ ] Integrate with existing database/API
- [ ] Write unit tests

### Multi-Platform Abstraction (2-3 hours total) - OPTIONAL
- [ ] Design ChatPlatform trait for platform abstraction
- [ ] Refactor Twitch and YouTube implementations
- [ ] Add platform-agnostic command handling
