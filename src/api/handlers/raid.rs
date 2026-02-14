use axum::{extract::State, Json};
use http::StatusCode;
use crate::api::{state::AppState, dto::CreateRaidRequest, error::AppError};
use crate::api::dto;
use crate::db;

pub async fn create_raid(
    State(state): State<AppState>,
    Json(req): Json<CreateRaidRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let session = db::get_active_session(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    let session = session.ok_or_else(||
        AppError::NotFound("No active session found. Cannot start a raid.".into())
    )?;

    let active_raid = db::get_active_raid(&state.pool).await
        .map_err(AppError::DatabaseError)?;

    if active_raid.is_some() {
        return Err(AppError::Conflict("Raid already in progress".into()));
    }

    let raid_id = db::create_raid(
        &state.pool,
        session.session_id,
        &req.map_name,
        req.character_type,
        req.game_mode,
        None, //started_at defaults to now
    ).await.map_err(AppError::DatabaseError)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({"raid_id": raid_id})),
    ))

}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use http::StatusCode;
    use tower::ServiceExt;
    use crate::api;
    use crate::api::{state::AppState, routes::api_router};
    use crate::db::{self, tests::setup_test_db};
    use crate::models::SessionType;

    #[tokio::test]
    async fn test_create_raid_success() {
        let pool = setup_test_db().await.expect("setup db");

        // 1. Create a PreRequisite Sesssion
        db::create_session(&pool, SessionType::Stream, None, None)
            .await.expect("create session");

        let app = api_router().with_state(AppState::new(pool));

        // 2. Send POST /api/raid
        let response = app
            .oneshot(
                Request::post("/api/raid")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "map_name": "Customs",
                        "character_type": "pmc",
                        "game_mode": "pve"}"#)
                    ).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // 3. Verify Response Body
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("raid_id").is_some());
    }

    #[tokio::test]
    async fn test_create_raid_no_active_session() {
        let pool = setup_test_db().await.expect("setup db");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/raid")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "map_name": "Customs",
                        "character_type": "pmc",
                        "game_mode": "pve"}"#)
                    ).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_raid_conflict_active_raid() {
        let pool = setup_test_db().await.expect("setup db");

        let session_id = db::create_session(&pool, SessionType::Stream, None, None).await.expect("session");

        db::create_raid(&pool, session_id, "Woods",
            crate::models::CharacterType::PMC, 
            crate::models::GameMode::PVP, 
            None)
        .await.expect("raid");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/raid")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "map_name": "Customs",
                        "character_type": "pmc",
                        "game_mode": "pve"}"#)
                    ).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
