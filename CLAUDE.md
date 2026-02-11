# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in
this repository.

## Project Overview

Tarkov Stream Producer is a Rust application designed to track Escape from Tarkov
gameplay statistics and display them on stream. The project is in early development
(Phase 1) with plans to evolve from manual control via REST API to automated screen
analysis using OCR/vision.

**Current Status:** Phase 2a (Core) completed with 4-table database schema and CRUD operations.
Currently working on Phase 2a-Extended (analytics and time tracking). Web API (Phase 2b),
OBS integration, and automated screen analysis are planned future phases.

## CRITICAL: Security & Streaming Protection (HIGHEST PRIORITY)

**The user may stream their screen at any time. Credentials and secrets must NEVER be visible.**

### Credential Protection Rules
- **NEVER display the contents of credential files** in terminal output or tool results
- **NEVER use `cat`, `Read`, or any tool that displays credential file contents**
- **Location references ONLY**: May mention that credentials exist at `./temp_bot/.env` or similar paths
- **Content is FORBIDDEN**: Never show tokens, passwords, API keys, client secrets, or any credential values

### Files Containing Credentials (Reference Location Only)
- `./temp_bot/.env` - Twitch bot credentials (OAuth token, client ID, client secret, bot ID)
- Any file matching patterns: `*.env`, `*.key`, `*.pem`, `*.secret`, `credentials.*`
- Database connection strings with passwords

### Acceptable Actions
- ✅ Mention file paths: "Update your OAuth token in `./temp_bot/.env`"
- ✅ Describe structure: "The .env file needs `TWITCH_TOKEN` and `CLIENT_SECRET` fields"
- ✅ Check file existence: `test -f ./temp_bot/.env && echo "exists"`

### Forbidden Actions
- ❌ Reading credential files: `cat .env`, `Read tool on .env`
- ❌ Displaying secrets in output: `echo $TWITCH_TOKEN`
- ❌ Showing redacted credentials: Even `oauth:xxxx...` patterns can leak information

**Violation of these rules could expose credentials on stream. This is the HIGHEST PRIORITY rule.**

### Dangerous Shell Commands
- **NEVER run `rm` command** - User will handle all file/directory deletion themselves
- **NEVER run destructive commands** without explicit user request (e.g., `git reset --hard`, `git clean -f`)

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
│   ├── main.rs          # Entry point with tracing setup
│   ├── models.rs        # Data structures (Session, Raid, Kill, enums)
│   ├── db.rs            # Database CRUD operations and migrations
│   └── stats.rs         # Analytics functions (in progress)
├── migrations/
│   └── 20251226000000_initial_schema.sql  # 4-table schema
├── .github/workflows/
│   ├── ci.yml           # CI pipeline for Linux/Windows
│   └── release.yml      # Release builds for tagged versions
├── docs/
│   ├── phase_1b_plan.md           # Phase 1.b implementation plan
│   ├── phase_2a_complete_schema.md # Complete 4-table schema documentation
│   └── phase_2b_rest_api_plan.md  # Phase 2b REST API plan
├── Cargo.toml           # Project manifest
├── dev.db               # Development SQLite database (gitignored)
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

Session Time Tracking (Phase 2a-Extended) ✅ COMPLETE

- **Session overhead time**: Gap between session start and first raid start
- Tracks "stream setup", "just chatting", or menu time before first raid begins
- Implementation in `src/stats.rs` with `calculate_time_before_first_raid()` ✅
- No schema changes needed - calculated from `session.started_at` and first `raid.started_at`
- **Completed (2026-01-18):** Database helper functions in `src/db.rs`:
  - `get_session_by_id()` - fetch single session by ID
  - `get_all_sessions()` - fetch all sessions ordered by started_at DESC
  - `get_first_raid_for_session()` - fetch first raid for a session
- **Completed (2026-01-18):** `calculate_time_before_first_raid()` implementation
  - Calculates average delay across all sessions
  - Tracks total session count
  - Tracks most recent session delay separately
  - Full test coverage with controlled timestamps
- **Completed (2026-01-20):** `calculate_time_in_state()` implementation
  - HashMap-based accumulation for time spent in each state
  - Handles multiple visits to same state using Entry API
  - Full test with realistic 7-state raid flow
- **Completed (2026-01-24):** Session comparison queries
  - `compare_session_to_mode_global()` - "this stream vs all-time" stats
  - `get_mode_stats_for_session()` - PVE vs PVP filtering
  - Calculate K/D, survival rate, raid counts
- **Completed (2026-01-26):** `calculate_time_between_raids()` implementation
  - Tracks average stash time between raids
  - Handles active raids, session boundaries, negative gaps
  - Global and per-session statistics
- **Phase 2a-Extended COMPLETE:** 2026-01-26

**Testing Pattern Learned:**
- Use optional timestamp parameters (`Option<OffsetDateTime>`) for CRUD functions
- Follows existing pattern from `log_state_transition()`, `add_kill()`, `end_raid()`
- Production code passes `None` to use real timestamps
- Tests pass `Some(timestamp)` for deterministic, fast tests
- No need for raw SQL in tests - clean API for both production and testing

REST API Development (Phase 2b) - IN PROGRESS

- **Framework choice:** Axum 0.8 selected for REST API
- **Development approach:** Test-Driven Development (TDD) with Red-Green-Refactor cycle
- **Started:** 2026-02-03
- **Current status:** Phase 2b.1 (Core Infrastructure) ✅ COMPLETE, Phase 2b.1-Refine next
- **Completed (2026-02-03):**
  - Step 1.1: AppError enum with variants (NotFound, Conflict, ValidationError, BadRequest, DatabaseError)
  - Step 1.2: status_code() method for HTTP status mapping
  - Step 1.3: json_body() method returning `{"error": "message", "type": "error_type"}`
  - Step 1.4: IntoResponse trait implementation for automatic error conversion
- **Completed (2026-02-08):**
  - Step 1.5: AppState struct with SqlitePool + Clone derive
  - Step 1.6: health_check() handler with State extractor
  - Step 1.7: Health response body format verification
- **Completed (2026-02-09):**
  - Step 1.8: api_router() in src/api/routes.rs mounting health endpoint
  - Step 1.9: Integration checkpoint — main.rs wired as async server, curl /health verified
- **Next:** Phase 2b.1-Refine (health ping, env var config, TraceLayer, cleanup), then Phase 2b.2 (Session Endpoints)

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

## Development Notes & Learnings

### Rust Error Handling Best Practices (2026-01-18)
- **Production code**: Always use `?` operator to propagate errors up the call stack
- **Test functions**: Use `?` in functions that return `Result<(), Error>` for better error messages
  - `.expect()` can be used for setup functions when you want custom panic messages
  - But `?` generally provides better debugging information
- **Never mix**: Don't use `.expect()` in functions that return `Result` - be consistent
- **Unused results**: Prefix variables with `_` (e.g., `_states`) to indicate intentionally unused

### Testing with Timestamps
- Add `Option<OffsetDateTime>` parameters to CRUD functions for testability
- Use `timestamp.unwrap_or_else(|| OffsetDateTime::now_utc())` pattern
- Production code passes `None`, tests pass `Some(timestamp)` for deterministic behavior
- Follows consistent pattern across: `create_session()`, `create_raid()`, `log_state_transition()`, `add_kill()`, `end_raid()`

### HashMap and Accumulation Patterns (2026-01-20)
- **Use HashMap for accumulation**: When accumulating values by key (e.g., time per state), use `HashMap<K, V>`
- **Entry API pattern**: More efficient than manual `get_mut()` + `insert()`
  ```rust
  map.entry(key.clone())
      .and_modify(|v| *v = *v + new_value)  // Update if exists
      .or_insert(new_value);                 // Insert if new
  ```
- **Only one HashMap lookup** vs two with `if let Some() { } else { }` pattern
- **Why clone for keys**: HashMap needs ownership of keys, so clone when key is borrowed via reference

### References vs Clones (2026-01-20)
- **`&vec[i]` creates a reference (borrow)**, not a clone
- **`let item = vec[i]`** would try to move/take ownership (often won't compile for Vec)
- **`let item = vec[i].clone()`** creates a copy
- **When to clone**: Only when you need ownership (e.g., HashMap keys, returning from functions)
- **Prefer borrowing**: More efficient, no memory allocation

### Iterator Chains (2026-01-20)
- **`into_iter()`**: Consumes collection, yields owned values - use when you don't need original
- **`.iter()`**: Borrows collection, yields references - collection still usable after
- **`.iter_mut()`**: Mutable borrows, yields mutable references
- **Tuple destructuring**: `|(key, value)|` in closures to split tuples
- **Field shorthand**: `Struct { field1, field2 }` when variable names match field names
- **Pattern**: `collection.into_iter().map(|x| transform(x)).collect()` is idiomatic

### Searching Collections (2026-01-20)
- **`.find()`**: O(n) search, fine for small collections (<10-20 items)
  ```rust
  let item = vec.iter().find(|x| x.field == "value");  // Returns Option<&T>
  ```
- **HashMap**: O(1) lookup, better for large collections or repeated searches
- **Binary search**: O(log n), requires sorted collection
- **Trade-off**: For small datasets like state transitions (~7-10 states), `.find()` is simpler and fast enough

### State Flow Clarification (2026-01-20)
- **Raids start in `pre_raid_setup`**, NOT `stash_management`
- **Session overhead time**: Gap between session start and first raid start (tracked by `calculate_time_before_first_raid()`)
- **Between raids**: After terminal state (`survived`/`died`/`mia`), implicit return to stash management
- **Next raid creation**: Starts fresh in `pre_raid_setup` state
- **Stash time between raids**: Will be tracked separately in future implementation

### Code Review Process
- User prefers to see exact line numbers and issues before corrections
- Always read the actual file to verify line numbers (git diff may not reflect current state)
- Get explicit confirmation before making corrections

### Rust Implementation Blocks (2026-02-03)
- **`impl` keyword**: Stands for "implementation" - defines methods for a type
- **Two forms**:
  1. `impl TypeName { }` - Add methods to your own types
  2. `impl TraitName for TypeName { }` - Implement a trait (interface) for a type
- **Instance methods**: Use `&self` parameter (borrows the instance)
- **Associated functions**: No `self` parameter (like static methods)
- **Example**:
  ```rust
  impl AppError {
      pub fn status_code(&self) -> http::StatusCode { /* ... */ }
  }
  ```

### Axum REST API Patterns (2026-02-09)
- **Router**: Traffic director that maps URL paths + HTTP methods to handler functions
  - `.route("/health", axum::routing::get(health_check))` — maps GET /health to handler
  - `Router<AppState>` — router that needs state before it can serve (incomplete)
  - `Router<()>` — fully wired router, ready to serve
  - `.with_state(AppState::new(pool))` fills the requirement: `Router<AppState>` → `Router<()>`
- **State injection**: `AppState` wraps `SqlitePool`, attached via `.with_state()`, automatically provided to any handler that uses `State(state): State<AppState>`
- **Testing without a server**: `tower::ServiceExt` provides `.oneshot()` for in-process HTTP testing
  - Requires `tower = { version = "0.5", features = ["util"] }` — the `"util"` feature is needed
  - Build fake requests with `Request::get("/path").body(Body::empty()).unwrap()`
  - No network, no port binding — fast and reliable tests
- **`#[tokio::main]`**: Required on `async fn main()` to set up the async runtime
- **Server startup pattern**: `tokio::net::TcpListener::bind()` → `axum::serve(listener, app)`
  - Bind to `127.0.0.1:3000` for localhost-only access (safe for streaming)

### SQLx Compile-Time Query Checking (2026-02-09)
- `sqlx::query!()` macro validates SQL against the database schema **at compile time**
- Requires `DATABASE_URL` environment variable to be set, OR an offline cache in `.sqlx/`
- **Development workflow**: `DATABASE_URL=sqlite://dev.db cargo run` (inline, per-command)
- **Offline cache**: `DATABASE_URL=sqlite://dev.db cargo sqlx prepare` updates `.sqlx/` directory
- `sqlite://` and `sqlite:` URL formats both work for SQLx
- **Stale database**: If `dev.db` has tables but no `_sqlx_migrations` tracking table, migrations fail with "table already exists" — delete `dev.db` and let migrations recreate it

### Fully Qualified Paths vs Imports (2026-02-03)
- **Fully qualified paths**: `impl axum::response::IntoResponse for AppError`
  - Avoids unused import warnings
  - Makes dependencies explicit at use site
  - Longer but clear
- **With imports**: Add `use axum::response::IntoResponse;` at top, then use short form `impl IntoResponse for AppError`
  - Cleaner code if type is used multiple times
  - Can trigger "unused import" if only used in impl signature
- **Choice**: Either approach is valid - pick based on:
  - One usage → fully qualified path (avoids warning)
  - Multiple usages → import at top (cleaner code)
- **Test imports**: Can import inside test module or function scope to avoid module-level unused warnings

## User Preferences

The user is a kernel engineer learning web/application development. TDD and REST APIs are new territory.
When working with this user:
- Always ask for confirmation before running shell commands
- Provide code recommendations and suggestions
- Focus on analysis, explanations, and providing code snippets for the user to implement
- **Explain web/REST/TDD concepts** as they come up — don't assume familiarity
- **Give refreshers on the "why"** — remind why we're doing a step, not just what to do
- **Walk through implementation step-by-step** — don't dump full solutions; guide the user to write the code themselves
- The user handles 90% of the coding themselves

### File Modification Permissions
- **CLAUDE.md** - Claude may update with confirmation first
- **todo.md** - Claude may update with confirmation first
- **.time_tracking.md** - Claude may update directly to track task progress
- **All other files** - Do not use Write or Edit tools. Provide recommendations only; the user will make changes themselves

### Time Tracking
- Use `date +"%Y-%m-%d %H:%M"` to get current timestamp for time tracking entries
- Update `.time_tracking.md` as tasks progress to maintain accurate time records
- **When the user begins working on a task**, immediately log the start time in `.time_tracking.md` before providing recommendations
