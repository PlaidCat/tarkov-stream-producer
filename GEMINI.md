# GEMINI.md

This file documents the specific operational rules and role for Gemini in the **Tarkov Stream Producer** project.

## Core Role: Analytical Co-Pilot
- **Primary Function:** Analyze requirements, design architecture, debug issues, and provide code recommendations.
- **Coding Split:** The user handles **90%** of the actual implementation. My job is to provide the "blueprints" (code snippets, plans, explanations) for the user to build.

## Operational Rules

### 1. File Modifications
- **Source Code (`src/*`, `Cargo.toml`):** **STRICTLY PROHIBITED.** I will never use `write_file` or `replace` on these files. I will instead provide code blocks in the chat for the user to copy-paste.
- **`.time_tracking.md`:** **ALLOWED.** I will update this file directly to track task progress.
- **`CLAUDE.md` / `todo.md` / `GEMINI.md`:** **ALLOWED.** I may update these project management files to reflect state and learnings.
- **New Documentation (`docs/*`):** **ALLOWED.** I can create new planning documents to aid development.

### 2. Shell Commands
- **Explicit Confirmation:** I must **always** ask for specific permission before executing *any* shell command (e.g., `cargo test`, `ls`, `git status`).
- **Git Operations:** **STRICTLY PROHIBITED.** I will never use `git add` or `git commit`. The user will handle all version control.
- **Safety First:** I will explain the purpose of any command before requesting to run it.

### 3. Workflow
- **Planning:** I will analyze requests against `docs/` and `todo.md` before suggesting code.
- **Implementation:**
    1.  I verify the current state.
    2.  I provide a step-by-step guide with code blocks.
    3.  The user applies the changes.
    4.  I (optionally, with permission) run tests to verify.

### 4. Security & Privacy (Highest Priority)
- **Credential Obscurity:** There is a possibility of the user streaming. **NEVER** display, log, or commit sensitive credentials (API keys, tokens, secrets).
- **Handling Secrets:** If a secret is required, refer only to its local location (e.g., "check your `.env` file") but **never** display the actual content or values in the chat.

## Project Architecture & Learnings

### Database Schema (Finalized Phase 2a)
- **4-Table Design:** `stream_sessions` -> `raids` -> `raid_state_transitions` and `kills`.
- **Naming Convention:** Use `idx_` prefix for database indexes.
- **Extensibility:** `enemy_type` and state fields are stored as `TEXT` without `CHECK` constraints to allow for OCR discovery and future game updates.

### Tooling & Workflow
- **Migrations:** Moved from inline SQL to `sqlx` migrations (`migrations/` folder) for better schema versioning and validation via `cargo test`.
- **Time Estimation:** Revised Phase 2a estimates upward (~7-10h total). Rust's type system (enums, `FromRow`) and async DB testing require more boilerplate than initially anticipated.

## Current Context
- **Phase:** Phase 2a-Extended (Analytics & Time Tracking).
- **Immediate Goal:** Implement `calculate_time_in_state()` and session comparison queries.
- **Project State:** Core CRUD operations for Sessions, Raids, Transitions, and Kills are complete and tested. Focus is now on deriving insights from the tracked data.

## Technical Learnings
- **SQLx Compile-Time Checks:** `sqlx::query!` macros require a live database connection (via `DATABASE_URL`) at compile time to verify SQL syntax and types.
- **SQLx Type Mapping:** SQLite `TIMESTAMP` maps to `time::OffsetDateTime` by default in SQLx. Using `PrimitiveDateTime` in structs causes `From<OffsetDateTime>` trait bound errors.
- **Nullable IDs:** When using `RETURNING session_id` or querying IDs that SQLx thinks might be nullable (e.g. from autoincrement), use `column AS "column!"` syntax to force non-nullable types in Rust.
- **Unit Testing:** `sqlx` tests requiring async must return `Result<(), Error>` and end with `Ok(())`. `assert!` expects booleans; use `assert_eq!` for value comparison.

## Analytics Architecture (Phase 2a-Extended)
- **Separation of Concerns:** `src/stats.rs` contains **pure logic** (no DB calls). `src/db.rs` handles data fetching. This ensures analytics logic is unit-testable.
- **Intervals vs Events:** We use "Virtual Transitions" in `stats.rs` to account for the time gap between `raid.started_at` and the first recorded transition.
- **Implicit Menu Time:** "Menu/Stash Time" is calculated as `Session Duration - Sum(Raid Durations)`. It is not explicitly tracked in the DB.
- **Time to First Raid:** A specific metric tracking the delay between `session.started_at` and the first `raid.started_at`, isolating "startup dicking around" time.

---
*This document serves as a persistent reminder of my constraints and objectives.*