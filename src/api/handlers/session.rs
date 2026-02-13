use axum::extract::State;
use axum::Json;
use http::StatusCode;
use crate::api::state::AppState;
use crate::api::dto::CreateSessionRequest;
use crate::db;

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), crate::api::error::AppError> {
    let session_id = db::create_session(
        &state.pool, 
        req.session_type, 
        req.notes,
        None,
    ).await.map_err(crate::api::error::AppError::DatabaseError)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "session_id": session_id })),
    ))
}

pub async fn get_current_session(
    State(state): State<AppState>
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

pub async fn end_current_session(
    State(state): State<AppState>
) -> Result<Json<serde_json::Value>, crate::api::error::AppError> {
    let session = db::get_active_session(&state.pool)
        .await.map_err(crate::api::error::AppError::DatabaseError)?;

    match session {
        Some(s) => {
            db::end_session(&state.pool, s.session_id)
                .await.map_err(crate::api::error::AppError::DatabaseError)?;
            Ok(Json(serde_json::json!({
                "status": "success",
                "session_id": s.session_id,
                "message": "session ended"
            })))
        },
        None => Err(crate::api::error::AppError::NotFound("No Active session to end".into())),
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use http::StatusCode;
    use tower::ServiceExt;
    use crate::api;
    use crate::api::state::AppState;
    use crate::api::routes::api_router;
    use crate::db::tests::setup_test_db;
    use crate::db;

    #[tokio::test]
    async fn test_create_session_returns_create() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

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

    #[tokio::test]
    async fn test_get_current_session_returns_404_when_none() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/session/current")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_current_session_returns_session() {
        let pool = setup_test_db().await.expect("setup db");

        // Create a Session directyly via db layer
        let session_id = db::create_session(
            &pool,
            crate::models::SessionType::Stream,
            Some("Test Session".into()),
            None,
        ).await.expect("create session");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/session/current")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["session_id"], session_id);
        assert_eq!(json["session_type"], "stream");
    }

    #[tokio::test]
    async fn test_end_session_returns_ok() {
        let pool = setup_test_db().await.expect("setup db");

        //create a session directly via db layer
        let session_id = db::create_session(
            &pool,
            crate::models::SessionType::Stream,
            Some("Test Session to end".into()),
            None,
        ).await.expect("create session");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/session/end")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_end_session_returns_404_when_nond() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/session/end")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
