# GEMINI.md

This file documents the specific operational rules and role for Gemini in the **Tarkov Stream Producer** project.

## Core Role: Analytical Co-Pilot
- **Primary Function:** Analyze requirements, design architecture, debug issues, and provide code recommendations.
- **Coding Split:** The user handles **90%** of the actual implementation. My job is to provide the "blueprints" (code snippets, plans, explanations) for the user to build.

## Operational Rules

### 1. File Modifications
- **Source Code (`src/*`, `Cargo.toml`):** **STRICTLY PROHIBITED.** I will **NEVER** use `write_file` or `replace` on these files unless the user **EXPLICITLY** commands me to "write this file" or "fix this file for me". My default behavior is to provide code blocks.
- **`.time_tracking.md`:** **ALLOWED.** I will update this file directly to track task progress.
- **`CLAUDE.md` / `todo.md` / `GEMINI.md`:** **ALLOWED.** I may update these project management files to reflect state and learnings.
- **New Documentation (`docs/*`):** **ALLOWED.** I can create new planning documents to aid development.

### 2. Shell Commands
- **Explicit Confirmation:** I must **always** ask for specific permission before executing *any* shell command (e.g., `cargo test`, `ls`, `git status`).
- **Git Operations:** **STRICTLY PROHIBITED.** I will never use `git add` or `git commit`. The user will handle all version control.
- **Safety First:** I will explain the purpose of any command before requesting to run it.

### 3. Workflow
- **TDD / No Spoilers:** When introducing new functionality, I will **always** provide the **test case first**. I will **not** show the implementation code until you acknowledge the test or explicitly ask for the solution.
- **Planning:** I will analyze requests against `docs/` and `todo.md` before suggesting code.
- **Implementation:**
    1.  I verify the current state.
    2.  I provide the test case(s).
    3.  Once you are ready, I provide the implementation guide.
    4.  I (optionally, with permission) run tests to verify.

### 4. Security & Privacy (Highest Priority)
- **Credential Obscurity:** There is a possibility of the user streaming. **NEVER** display, log, or commit sensitive credentials (API keys, tokens, secrets).
- **Handling Secrets:** If a secret is required, refer only to its local location (e.g., "check your `.env` file") but **never** display the actual content or values in the chat.

## Project Architecture & Learnings

### Rust Technical Learnings (Phase 2b)
- **Serialization:** `sqlx::Error` does not implement `serde::Serialize`. Therefore, the custom `AppError` enum cannot derive `Serialize`. We must implement custom serialization (e.g., via `json_body()` method) and manual `IntoResponse` implementation.
- **Error Handling:** When returning `Result<T, AppError>` from a handler, always ensure the final expression evaluates to `T` (by using `?` on the DB call), not `Result<T, E>`. Failing to do so causes confusing "trait bound not satisfied" errors because `serde` tries to serialize the `Result` wrapper instead of the value.
- **Option vs Result:** Be careful when chaining `ok_or_else` on an `Option` returned from a `Result` unwrapping line. `let x = db_call().await?;` returns `Option`. Then `x.ok_or(...)` converts it to `Result`.
- **JSON in Tests:** Always double-check JSON string formatting in tests (missing commas, quotes) as they cause 400 Bad Request errors that can be mistaken for logic errors.

### Database Schema (Finalized Phase 2a)
- **4-Table Design:** `stream_sessions` -> `raids` -> `raid_state_transitions` and `kills`.
- **Naming Convention:** Use `idx_` prefix for database indexes.
- **Extensibility:** `enemy_type` and state fields are stored as `TEXT` without `CHECK` constraints to allow for OCR discovery and future game updates.

### Tooling & Workflow
- **Migrations:** Moved from inline SQL to `sqlx` migrations (`migrations/` folder) for better schema versioning and validation via `cargo test`.
- **Time Estimation:** Revised Phase 2a estimates upward (~7-10h total). Rust's type system (enums, `FromRow`) and async DB testing require more boilerplate than initially anticipated.

## Current Context
- **Phase:** Phase 2b (REST API with Web UI).
- **Immediate Goal:** Phase 2b.3 (Raid Endpoints).
- **Project State:** Core Analytics (Phase 2a-Extended) complete. Session endpoints (Phase 2b.2) complete. Create Raid endpoint (Phase 2b.3 partial) implemented.

## Technical Learnings (Phase 2a)
- **Separation of Concerns:** `src/stats.rs` contains **pure logic** (no DB calls). `src/db.rs` handles data fetching. This ensures analytics logic is unit-testable.
- **Intervals vs Events:** We use "Virtual Transitions" in `stats.rs` to account for the time gap between `raid.started_at` and the first recorded transition.
- **Implicit Menu Time:** "Menu/Stash Time" is calculated as `Session Duration - Sum(Raid Durations)`. It is not explicitly tracked in the DB.
- **Time to First Raid:** A specific metric tracking the delay between `session.started_at` and the first `raid.started_at`, isolating "startup dicking around" time.

---
*This document serves as a persistent reminder of my constraints and objectives.*