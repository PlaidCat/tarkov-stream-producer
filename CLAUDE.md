# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in
this repository.

## Project Overview

Tarkov Stream Producer is a Rust application designed to track Escape from Tarkov
gameplay statistics and display them on stream. The project is in early development
(Phase 1) with plans to evolve from manual control via REST API to automated screen
analysis using OCR/vision.

**Current Status:** Basic logging and testing infrastructure in place. Database
integration, web API, OBS integration, and automated screen analysis are planned
future phases.

## Hardware Environment

### Dev/Training System (Arch Linux)
- **CPU**: AMD Ryzen 9 9950X 16-Core (32 threads)
- **RAM**: 96GB DDR5
- **GPU**: AMD Radeon RX 7900 XTX (24GB VRAM)
- **ROCm**: 7.1.1 (for ML training in Phase 4)
- **Storage**: 3.6TB NVMe + 3x 1TB drives
- **OS**: Arch Linux
- **Purpose**: Development, testing, model training (Phase 4)

### Gaming PC (Windows - Dual Boot)
- **Hardware**: Same as Dev System (9950X, 96GB, 7900 XTX)
- **OS**: Windows (dual-boot with Arch Linux)
- **Purpose**: Running Escape from Tarkov
- **Restrictions**: Anticheat software prevents running detection services alongside game
- **Note**: Game footage captured via Elgato on separate Streaming PC

### Streaming/Production PC (Windows 11)
- **CPU**: AMD Ryzen 9 5900X (24 threads)
- **RAM**: 32GB DDR4
- **GPU**: NVIDIA GeForce RTX 5070 (16GB VRAM)
- **Capture**: Elgato 4K X (captures Gaming PC output)
- **Storage**: NVMe SSD
- **OS**: Windows 11
- **Purpose**: OBS streaming, video detection inference (Phase 4)
- **Software**: OBS Studio, detection service (Python), Rust app

**Architecture Notes:**
- Gaming PC runs game in isolation (anticheat compliance)
- Elgato 4K X captures HDMI output from Gaming PC
- Streaming PC receives clean Elgato feed for detection + adds overlays for stream
- Model training on Dev System (AMD/ROCm), inference on Streaming PC (NVIDIA/TensorRT)

## Project Structure

tarkov_stream_producer/
├── src/
│   └── main.rs          # Entry point with tracing setup
├── .github/workflows/
│   ├── ci.yml           # CI pipeline for Linux/Windows
│   └── release.yml      # Release builds for tagged versions
├── docs/
│   └── phase_1b_plan.md # Phase 1.b implementation plan
├── Cargo.toml           # Project manifest
└── target/              # Build artifacts (gitignored)

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build in release mode
cargo build --release

Testing

# Run all tests
cargo test --all

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Code coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Lcov

# Code coverage with HTML report (opens in browser)
cargo tarpaulin --out Html

Logging

The application uses tracing and tracing-subscriber for structured logging. Control
log level via the RUST_LOG environment variable:

# Run with debug logging
RUST_LOG=debug cargo run

# Default is INFO level if RUST_LOG is not set
cargo run

Architecture Notes

Logging Infrastructure

- Uses tracing crate for structured logging
- EnvFilter configured to default to INFO level
- Log level controllable via RUST_LOG environment variable

CI/CD Pipeline

- GitHub Actions runs on push and pull requests
- Tests executed on both Ubuntu and Windows
- Code coverage measured using cargo-tarpaulin (set as dev-dependency, not installed
system-wide yet per todo.md)
- Coverage target: 50% for core logic

Release Builds (Phase 1.b) ✅

- Windows debug executables produced on tagged releases (v* tags)
- Artifacts named: `tarkov_stream_producer-windows-{git-sha}.exe`
- Release builds use optimized code with debug symbols (`profile.release.debug = true`)
- 7-day artifact retention on GitHub Actions
- Enables cross-platform debugging on Arch Linux without Windows dev environment
- Workflow uses `dtolnay/rust-toolchain@stable` for consistency with CI
- **Completed:** 2025-12-08

Database Migrations (Phase 2a) ✅

- Uses `sqlx` migrations with dedicated `migrations/` directory
- Migration files named with timestamp: `YYYYMMDDHHMMSS_description.sql`
- Applied automatically via `sqlx::migrate!()` macro at runtime
- SQL syntax validation in CI pipeline (Linux only, skipped on Windows)
- Validation command: `sqlite3 :memory: < migrations/*.sql`
- **Design pattern:** Avoid CHECK constraints on extensible fields (e.g., `enemy_type`, `status`)
  to allow discovery of new values without schema migrations
- **Started:** 2025-12-26

Session Time Tracking (Phase 2a-Extended)

- **Session overhead time**: Gap between session start and first raid start
- Tracks "stream setup", "just chatting", or menu time before first raid begins
- Implemented in `src/stats.rs` with `calculate_time_before_first_raid()`
- No schema changes needed - calculated from `session.started_at` and first `raid.started_at`
- Database helper functions in `src/db.rs`: `get_session_by_id()`, `get_first_raid_for_session()`
- Useful metric: "How much time do I waste before actually starting raids?"
- Aggregates across all sessions for historical analysis

Planned Architecture (Future Phases)

Phase 2: Core Data & API Foundation

- Rust structs for game state (Raid, Kill) with sqlx integration
- REST endpoints for manual control (POST /raid/start, /raid/kill, /raid/end)
- Stream Deck integration via HTTP requests
- Web framework to be chosen (actix-web or axum under consideration)

Phase 3: OBS Integration

- OBS stats display (text files or obs-websocket)
- Real-time stat overlays during gameplay

Phase 4: Automation

- Cross-platform screen capture
- OCR/vision for automated event detection (Tesseract initially)
- Replace manual API calls with automated detection

Phase 5: Chat Bot Integration (Deferred)

- Twitch bot for chat commands (!stats, !kd) - deferred due to unmaintained libraries
- YouTube Live Chat integration - requires different architecture (REST API vs IRC)
- Multi-platform chat abstraction when both platforms are implemented

## User Preferences

The user handles 90% of the coding themselves. When working with this user:
- Always ask for confirmation before running shell commands
- Provide code recommendations and suggestions
- Focus on analysis, explanations, and providing code snippets for the user to implement

### File Modification Permissions
- **CLAUDE.md** - Claude may update with confirmation first
- **todo.md** - Claude may update with confirmation first
- **.time_tracking.md** - Claude may update directly to track task progress
- **All other files** - Do not use Write or Edit tools. Provide recommendations only; the user will make changes themselves

### Time Tracking
- Use `date +"%Y-%m-%d %H:%M"` to get current timestamp for time tracking entries
- Update `.time_tracking.md` as tasks progress to maintain accurate time records
