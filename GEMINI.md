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

## Project Architecture & Learnings

### Database Schema (Finalized Phase 2a)
- **4-Table Design:** `stream_sessions` -> `raids` -> `raid_state_transitions` and `kills`.
- **Naming Convention:** Use `idx_` prefix for database indexes.
- **Extensibility:** `enemy_type` and state fields are stored as `TEXT` without `CHECK` constraints to allow for OCR discovery and future game updates.

### Tooling & Workflow
- **Migrations:** Moved from inline SQL to `sqlx` migrations (`migrations/` folder) for better schema versioning and validation via `cargo test`.
- **Time Estimation:** Revised Phase 2a estimates upward (~7-10h total). Rust's type system (enums, `FromRow`) and async DB testing require more boilerplate than initially anticipated.

## Current Context
- **Phase:** Phase 2a (Core Implementation).
- **Immediate Goal:** Fix type mismatches (`PrimitiveDateTime` vs `OffsetDateTime`) and complete CRUD operations in `src/db.rs`.
- **Project State:** `src/models.rs` needs update to `OffsetDateTime`. `src/db.rs` has partial session CRUD.

## Technical Learnings
- **SQLx Compile-Time Checks:** `sqlx::query!` macros require a live database connection (via `DATABASE_URL`) at compile time to verify SQL syntax and types.
- **SQLx Type Mapping:** SQLite `TIMESTAMP` maps to `time::OffsetDateTime` by default in SQLx. Using `PrimitiveDateTime` in structs causes `From<OffsetDateTime>` trait bound errors.
- **Nullable IDs:** When using `RETURNING session_id` or querying IDs that SQLx thinks might be nullable (e.g. from autoincrement), use `column AS "column!"` syntax to force non-nullable types in Rust.
- **Unit Testing:** `sqlx` tests requiring async must return `Result<(), Error>` and end with `Ok(())`. `assert!` expects booleans; use `assert_eq!` for value comparison.

---
*This document serves as a persistent reminder of my constraints and objectives.*