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

### Phase 2a: Data Structure & Database (4-5 hours total)
- [ ] Document required data (raid stats, kills, player info) (1h)
- [ ] Design Raid and Kill structs with relationships (1h)
- [ ] Create database schema and migration files (1h)
- [ ] Implement CRUD operations for Raid entity with tests (1h)
- [ ] Implement CRUD operations for Kill entity with tests (1h)

### Phase 2b: Web API for Manual Control (4-6 hours total)
- [ ] Choose web framework (actix-web vs axum) and add dependency (0.5h)
- [ ] Set up basic HTTP server with health check endpoint (1h)
- [ ] Implement POST /raid/start endpoint with database integration (1.5h)
- [ ] Implement POST /raid/kill endpoint with database integration (1h)
- [ ] Implement POST /raid/end endpoint with database integration (1.5h)
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
