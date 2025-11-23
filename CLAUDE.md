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

## Project Structure

tarkov_stream_producer/
├── src/
│   └── main.rs          # Entry point with tracing setup
├── .github/workflows/
│   └── ci.yml           # CI pipeline for Linux/Windows
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

Planned Architecture (Future Phases)

Phase 2: Core Logic

- Rust structs for game state (Raid, Player, Kill)
- State management for raid lifecycle
- Database integration using sqlx with SQLite

Phase 3: Web API

- REST endpoints for manual control (POST /raid/start, /raid/kill, /raid/end)
- Stream Deck integration point
- Web framework to be chosen (actix-web or axum under consideration)

Phase 4: Integration

- OBS stats display (text files or obs-websocket)
- Twitch bot for chat commands (!stats, !kd)

Phase 5: Automation

- Cross-platform screen capture
- OCR/vision for automated event detection (Tesseract initially)

## User Preferences

The user handles 90% of the coding themselves. When working with this user:
- Always ask for confirmation before running shell commands
- Provide code recommendations and suggestions
- Focus on analysis, explanations, and providing code snippets for the user to implement

### File Modification Permissions
- **CLAUDE.md** - Claude may update with confirmation first
- **todo.md** - Claude may update with confirmation first
- **All other files** - Do not use Write or Edit tools. Provide recommendations only; the user will make changes themselves
