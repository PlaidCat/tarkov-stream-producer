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
- [/] Implement CRUD for RaidStateTransition (0.75-1h, revised from 0.5h) - Create/Log implemented
- [ ] Implement CRUD for Kill (0.5-0.75h)
- [/] Write basic unit tests for all CRUD operations (1.5-2h, revised from 1h) - Session & Raid lifecycle tests added

### Phase 2a-Extended: Analytics & Time Tracking (1.5-2 hours total)
- [ ] Implement `calculate_time_in_state()` function (0.75h)
  - Query to sum time between state transitions
  - Handle multiple visits to same state
- [ ] Implement session comparison queries (0.5h)
  - "This stream vs all-time" stats
  - PVE vs PVP comparisons
- [ ] Write comprehensive tests for state transitions (0.75h)
  - Test "backwards" transitions (queue → stash)
  - Test time calculations
  - Test edge cases (reconnects, cancels)

### Phase 2b: Web API for Manual Control (5-7 hours total)
**Note:** Updated to support 4-table schema with sessions and state transitions.

- [ ] Choose web framework (actix-web vs axum) and add dependency (0.5h)
- [ ] Set up basic HTTP server with health check endpoint (1h)
- [ ] Implement session endpoints (1h)
  - `POST /session/start` - Create new streaming session
  - `POST /session/end` - End current session
  - `GET /session/current` - Get active session info
- [ ] Implement raid lifecycle endpoints (2h)
  - `POST /raid/start` - Create raid (with session_id)
  - `POST /raid/transition` - Record state change
  - `POST /raid/kill` - Add kill to raid
  - `POST /raid/end` - Finalize raid with outcome
- [ ] Implement stats query endpoints (1h)
  - `GET /stats/session/:id` - Session summary
  - `GET /stats/current` - Active session stats
  - `GET /stats/all-time` - Historical stats
- [ ] Add request validation and error handling (1h)
- [ ] Write integration tests for all endpoints (1.5h)

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
