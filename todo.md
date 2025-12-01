# Tarkov Stream Producer - Project Plan

This document outlines the development plan for the Tarkov Stream Producer application.

## Phase 1: Project Foundation

- [x] Set up a new Rust project using Cargo.
- [x] Choose and integrate a logging framework.
- [x] Choose and integrate a testing framework and set up a basic test.
- [x] Set up `cargo-tarpaulin` for code coverage analysis.
- [x] Establish a basic CI/CD pipeline (e.g., using GitHub Actions) to build and test on both Linux and Windows.
- [x] **Database Integration:** Choose and integrate a Rust SQL library/ORM (e.g., `sqlx` with SQLite).

## Phase 2: Core Logic and Data Structures

- [ ] Define the Rust structs to represent game state (e.g., `Raid`, `Player`, `Kill`, etc.).
- [ ] Implement the core logic for managing the game state (e.g., starting a raid, adding a kill, ending a raid).
- [ ] Write unit tests for the core logic to meet our 50% coverage goal.
- [ ] **Database Integration:** Design the database schema for storing raid statistics.
- [ ] **Database Integration:** Implement functions to save and retrieve raid data from the database.

## Phase 3: Web API (Manual Control)

- [ ] Choose a web framework (like `actix-web` or `axum`).
- [ ] Implement REST endpoints to control the game state (e.g., `POST /raid/start`, `POST /raid/kill`, `POST /raid/end`). These endpoints will interact with the database.
- [ ] This will be what your Stream Deck communicates with initially.

## Phase 4: OBS & Twitch Integration

- [ ] **OBS:** Decide on a method to display stats in OBS. We could generate text files that OBS reads, or use an OBS plugin like the `obs-websocket` plugin to update text sources directly.
- [ ] **Twitch Bot:** Choose a Twitch bot library for Rust. The bot will connect to the web API to fetch stats and respond to chat commands (e.g., `!stats`, `!kd`).

## Phase 5: Automated Screen Analysis

- [ ] **Screen Capture:** Research and implement a cross-platform screen capture library in Rust.
- [ ] **OCR/Vision:** This is the most complex part. We can start with a pre-trained OCR model (like Tesseract) to read text from the screen. Later, we can explore more advanced computer vision techniques or train a custom model to recognize specific in-game events.
