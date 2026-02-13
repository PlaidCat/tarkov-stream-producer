# Phase 2b: REST API - TDD Implementation Plan

**New file:** `docs/phase_2b_tdd_plan.md` (keeps existing `phase_2b_rest_api_plan.md`)

## Actions Before Starting
1. Delete existing empty `src/api/` directory
2. Create `docs/phase_2b_tdd_plan.md` with this plan content
3. Update `todo.md` Phase 2b.1 section to reference TDD steps

## Overview

Build a REST API with Axum framework using strict Test-Driven Development (TDD). Local-only (127.0.0.1), no authentication required.

**TDD Cycle (Red-Green-Refactor):**
1. **Red**: Write one failing test
2. **Green**: Write minimal code to make it pass
3. **Refactor**: Clean up while keeping tests green
4. Repeat

**Starting Point:** Delete existing empty `src/api/` directory and rebuild from first test.

## Technology Stack (from existing Cargo.toml)
- **Web Framework**: Axum 0.8
- **Middleware**: Tower 0.5, tower-http 0.6 (cors, trace)
- **Serialization**: serde 1.0, serde_json 1.0
- **Validation**: validator 0.20

## Target File Structure
```
src/
├── api/
│   ├── mod.rs           # Module exports
│   ├── error.rs         # AppError enum + IntoResponse
│   ├── state.rs         # AppState wrapper
│   ├── routes.rs        # Router configuration
│   ├── dto.rs           # Request/Response types
│   └── handlers/
│       ├── mod.rs
│       ├── health.rs    # GET /health
│       ├── session.rs   # Session endpoints
│       ├── raid.rs      # Raid endpoints
│       ├── kill.rs      # Kill endpoints
│       └── stats.rs     # Stats endpoints
```

## Existing Test Patterns to Follow
- In-memory SQLite via `setup_test_db()` from `src/db.rs`
- `#[tokio::test]` for async tests
- `Option<OffsetDateTime>` parameters for deterministic timestamps
- Tests inline using `#[cfg(test)] mod tests { ... }`
- Return `Result<(), sqlx::Error>` with `?` operator

---

## Phase 2b.1: Core Infrastructure (TDD)

### Step 1.1: AppError variants exist
**File:** `src/api/error.rs`

**Test (write first):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_variants_exist() {
        // Verify all error variants can be constructed
        let _ = AppError::NotFound("test".to_string());
        let _ = AppError::Conflict("test".to_string());
        let _ = AppError::ValidationError("test".to_string());
        let _ = AppError::BadRequest("test".to_string());
        // DatabaseError tested separately (needs sqlx::Error)
    }
}
```

**Implementation (after test fails):**
```rust
use axum::response::{IntoResponse, Response};
use http::StatusCode;

pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Conflict(String),
    ValidationError(String),
    BadRequest(String),
}
```

---

### Step 1.2: HTTP status code mapping
**Test:**
```rust
#[test]
fn test_error_status_codes() {
    assert_eq!(AppError::NotFound("x".into()).status_code(), StatusCode::NOT_FOUND);
    assert_eq!(AppError::Conflict("x".into()).status_code(), StatusCode::CONFLICT);
    assert_eq!(AppError::ValidationError("x".into()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(AppError::BadRequest("x".into()).status_code(), StatusCode::BAD_REQUEST);
}
```

**Implementation:**
```rust
impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}
```

---

### Step 1.3: Error JSON body format
**Test:**
```rust
#[test]
fn test_error_json_body() {
    let error = AppError::NotFound("User not found".into());
    let body = error.error_body();

    assert_eq!(body.error.code, "NOT_FOUND");
    assert_eq!(body.error.message, "User not found");
}
```

**Implementation:**
```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn error_body(&self) -> ErrorBody {
        let (code, message) = match self {
            AppError::DatabaseError(e) => ("DATABASE_ERROR".to_string(), e.to_string()),
            AppError::NotFound(msg) => ("NOT_FOUND".to_string(), msg.clone()),
            AppError::Conflict(msg) => ("CONFLICT".to_string(), msg.clone()),
            AppError::ValidationError(msg) => ("VALIDATION_ERROR".to_string(), msg.clone()),
            AppError::BadRequest(msg) => ("BAD_REQUEST".to_string(), msg.clone()),
        };
        ErrorBody {
            error: ErrorDetail { code, message },
        }
    }
}
```

---

### Step 1.4: IntoResponse implementation
**Test:**
```rust
#[test]
fn test_into_response_status() {
    let error = AppError::NotFound("test".into());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

**Implementation:**
```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = serde_json::to_string(&self.error_body()).unwrap();
        (status, [("content-type", "application/json")], body).into_response()
    }
}
```

---

### Step 1.5: AppState construction
**File:** `src/api/state.rs`

**Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::setup_test_db;

    #[tokio::test]
    async fn test_app_state_new() {
        let pool = setup_test_db().await.expect("setup db");
        let state = AppState::new(pool.clone());
        assert!(!state.pool.is_closed());
    }
}
```

**Implementation:**
```rust
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
```

---

### Step 1.6: Health endpoint returns 200
**File:** `src/api/handlers/health.rs`

**Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::state::AppState;
    use crate::db::setup_test_db;
    use axum::{body::Body, http::Request, Router};
    use tower::ServiceExt;
    use http::StatusCode;

    fn health_router(state: AppState) -> Router {
        Router::new()
            .route("/health", axum::routing::get(health_check))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_returns_ok() {
        let pool = setup_test_db().await.expect("setup db");
        let app = health_router(AppState::new(pool));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

**Implementation:**
```rust
use axum::{extract::State, Json};
use serde::Serialize;
use crate::api::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = if state.pool.is_closed() {
        "disconnected"
    } else {
        "connected"
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status.to_string(),
    })
}
```

---

### Step 1.7: Health response body format
**Test:**
```rust
#[tokio::test]
async fn test_health_response_body() {
    let pool = setup_test_db().await.expect("setup db");
    let app = health_router(AppState::new(pool));

    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["database"], "connected");
}
```

---

### Step 1.8: API router mounts health
**File:** `src/api/routes.rs`

**Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::state::AppState;
    use crate::db::setup_test_db;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_api_router_has_health() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);
    }
}
```

**Implementation:**
```rust
use axum::Router;
use crate::api::state::AppState;
use crate::api::handlers::health::health_check;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
}
```

---

### Step 1.9: Integration checkpoint
After completing steps 1.1-1.8:

```bash
# Verify all tests pass
cargo test --all

# Update main.rs to start server (you implement)
# Then test manually:
cargo run &
curl http://127.0.0.1:3000/health
# Expected: {"status":"ok","database":"connected"}
```

---

## Phase 2b.2: Session Endpoints (TDD)

### Step 2.1: CreateSessionRequest deserializes
**File:** `src/api/dto.rs`

**Test:**
```rust
#[test]
fn test_create_session_request_deserialize() {
    let json = r#"{"session_type": "stream", "notes": "Test session"}"#;
    let req: CreateSessionRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.session_type, "stream");
    assert_eq!(req.notes, Some("Test session".into()));
}

#[test]
fn test_create_session_request_notes_optional() {
    let json = r#"{"session_type": "practice"}"#;
    let req: CreateSessionRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.session_type, "practice");
    assert_eq!(req.notes, None);
}
```

---

### Step 2.2: POST /api/session creates session

**Test (write first):**
```rust
#[tokio::test]
async fn test_create_session_success() {
    let pool = setup_test_db().await.expect("setup db");
    let app = session_router().with_state(AppState::new(pool));

    let response = app
        .oneshot(
            Request::post("/api/session")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"session_type": "stream"}"#))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

**What you'll see (Red):**
- 404 Not Found because `/api/session` route doesn't exist yet
- Also notice the Step 2.3 test moved to `session::tests` — good move

---

**Now go Green. You need two things:**

**1. The handler function**

Add this to `src/api/handlers/sessions.rs`:

```rust
use axum::{extract::State, Json};
use serde::Serialize;
use crate::{api::state::AppState, db};

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: i64,
    pub session_type: String,
    pub notes: Option<String>,
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> (StatusCode, Json<CreateSessionResponse>) {
    let session_id = db::create_session(&state.pool, payload.session_type, payload.notes, None)
        .await
        .expect("Failed to create session");

    let response = CreateSessionResponse {
        session_id,
        session_type: payload.session_type.to_string(),
        notes: payload.notes,
    };

    (StatusCode::CREATED, Json(response))
}
```

**What's happening:**
- `State(state)` — Axum extractor gets your `AppState` with the DB pool
- `Json(payload)` — Axum automatically parses JSON request body into `CreateSessionRequest`
- `db::create_session()` — calls your Phase 2a database function
- Returns tuple: `(StatusCode::CREATED, Json(response))` — 201 status + JSON body

---

**2. Wire the route in `src/api/routes.rs`:**

```rust
use crate::api::handlers::sessions::create_session;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/session", axum::routing::post(create_session))
        .layer(TraceLayer::new_for_http())
}
```

**Key differences between POST and GET handlers:**
- POST needs `Json(payload)` to parse request body
- GET (for `/current`) needs **no body extractor** — just `State`

---

**Run the tests. You should see:**
```bash
cargo test test_create_session_success
```

**Expected:** Test passes with 201 Created status

### Step 2.3: GET /api/session/current returns 404 when none

**Test:**
```rust
#[tokio::test]
async fn test_get_current_session_none() {
    let pool = setup_test_db().await.expect("setup db");
    let app = session_router().with_state(AppState::new(pool));

    let response = app
        .oneshot(Request::get("/api/session/current").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

---

**Now go Green. You need:**

**1. The handler function**

Add `get_current_session` to `src/api/handlers/sessions.rs` alongside `create_session`:

```rust
use axum::{extract::State, Json};
use crate::{api::state::AppState, db};

pub async fn get_current_session(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, crate::api::error::AppError> {
    let session = db::get_active_session(&state.pool)
        .await
        .map_err(crate::api::error::AppError::DatabaseError)?;

    match session {
        Some(s) => Ok(Json(serde_json::json!({
            "session_id": s.session_id,
            "session_type": s.session_type,
            "started_at": s.started_at.to_string(),
            "ended_at": s.ended_at.map(|t| t.to_string()),
            "notes": s.notes,
        }))),
        None => Err(crate::api::error::AppError::NotFound(
            "No active session".into()
        )),
    }
}
```

**What's different from `create_session`:**
- No `Json(req)` extractor — GET requests don't have a body to parse, so we only need `State`
- `match session` — `get_active_session()` returns `Option<StreamSession>`. We use `match` to handle both cases:
  - `Some(s)` → build the JSON response (200 OK, which is the default for `Json<>`)
  - `None` → return `AppError::NotFound`, which your Phase 2b.1 error handling automatically converts to a 404 response
- This is where Step 2.3 becomes real — once this route is wired up, the 404 comes from your `AppError::NotFound` instead of Axum's "no matching route"

---

**2. Wire the route in `src/api/routes.rs`:**

```rust
use crate::api::handlers::sessions::{create_session, get_current_session};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/session", axum::routing::post(create_session))
        .route("/api/session/current", axum::routing::get(get_current_session))
        .layer(TraceLayer::new_for_http())
}
```

**Run the tests — both Step 2.3 and 2.4 should pass now.**

### Step 2.4: GET /api/session/current returns session
**Test:**
```rust
#[tokio::test]
async fn test_get_current_session_exists() {
    let pool = setup_test_db().await.expect("setup db");

    // Create session directly in DB first
    crate::db::create_session(&pool, SessionType::Stream, None, None).await.unwrap();

    let app = session_router().with_state(AppState::new(pool));
    let response = app
        .oneshot(Request::get("/api/session/current").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

### Step 2.5: POST /api/session/end ends session
**Test:**
```rust
#[tokio::test]
async fn test_end_session_success() {
    let pool = setup_test_db().await.expect("setup db");

    // Create session first
    crate::db::create_session(&pool, SessionType::Stream, None, None).await.unwrap();

    let app = session_router().with_state(AppState::new(pool.clone()));
    let response = app
        .oneshot(Request::post("/api/session/end").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify ended - no current session
    let app2 = session_router().with_state(AppState::new(pool));
    let check = app2
        .oneshot(Request::get("/api/session/current").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(check.status(), StatusCode::NOT_FOUND);
}
```

---

### Step 2.6: POST /api/session/end returns 404 when none
**Test:**
```rust
#[tokio::test]
async fn test_end_session_no_active() {
    let pool = setup_test_db().await.expect("setup db");
    let app = session_router().with_state(AppState::new(pool));

    let response = app
        .oneshot(Request::post("/api/session/end").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

---

## Phase 2b.3-2b.5: Raid, Kill, Stats Endpoints

Follow the same pattern for each endpoint group:

1. **DTO tests** - Deserialize request bodies
2. **Success case** - Happy path returns expected status
3. **Error cases** - Missing prerequisites return appropriate errors
4. **State verification** - Confirm database changes

### Raid Endpoints (Phase 2b.3)
- `POST /api/raid` - requires active session (400 without), conflicts if raid active (409)
- `GET /api/raid/current` - 404 when none, 200 with data when active
- `POST /api/raid/transition` - 404 when no raid, validates state values
- `POST /api/raid/end` - 404 when no raid, 200 on success

### Kill Endpoints (Phase 2b.4)
- `POST /api/raid/:id/kills` - 404 for invalid raid, 201 on success
- `POST /api/raid/current/kills/batch` - 404 when no raid, handles empty array
- `GET /api/raid/:id/kills` - 404 for invalid raid, returns array

### Stats Endpoints (Phase 2b.5)
- `GET /api/stats/session/current` - 404 when no session
- `GET /api/stats/raid/:id` - 404 for invalid raid

---

## Verification Checklist

After each step:
```bash
cargo test --all           # All tests pass
cargo clippy              # No warnings
cargo fmt --check         # Formatting correct
```

After Phase 2b.1 complete:
```bash
cargo run &
curl http://127.0.0.1:3000/health
```

After all API endpoints:
```bash
cargo tarpaulin --out Lcov  # Check coverage >= 50%
```

---

## Summary of TDD Steps

| Step | Test | Implementation |
|------|------|----------------|
| 1.1 | AppError variants exist | Define enum |
| 1.2 | Status code mapping | `status_code()` method |
| 1.3 | JSON body format | `error_body()` method |
| 1.4 | IntoResponse | Implement trait |
| 1.5 | AppState construction | Define struct + new() |
| 1.6 | Health returns 200 | Handler function |
| 1.7 | Health body format | JSON response |
| 1.8 | Router mounts health | api_router() |
| 1.9 | Integration test | Manual curl test |
| 2.1 | DTO deserialize | CreateSessionRequest |
| 2.2 | POST /api/session | create handler |
| 2.3 | GET current 404 | get_current handler |
| 2.4 | GET current 200 | handler with DB |
| 2.5 | POST end success | end handler |
| 2.6 | POST end 404 | error case |

---

## Notes

- Each step is one Red-Green-Refactor cycle
- Write the test first, see it fail (Red)
- Write minimal code to pass (Green)
- Refactor if needed, tests stay green
- Commit after each logical group of steps
