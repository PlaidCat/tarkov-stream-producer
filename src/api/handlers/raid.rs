use axum::{extract::State, Json};
use http::StatusCode;
use crate::api::{state::AppState, error::AppError};
use crate::api::dto::*;
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

pub async fn get_current_raid(
    State(state): State<AppState>,
) -> Result<Json<RaidResponse>, AppError> {
    //If there is an active raid there should be an active session I don't see a reason to check
    let raid = db::get_active_raid(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    match raid {
        Some(r) => Ok(Json(RaidResponse::from(r) ) ),
        None => Err(AppError::NotFound("No Active Raid".into())),
    }
}

pub async fn transition_state(
    State(state): State<AppState>,
    Json(req): Json<StateTransitionRequest>,
) -> Result<StatusCode, AppError> {
    let session = db::get_active_session(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    let _session = session.ok_or_else(||
        AppError::NotFound("No active session found, Cannot transition a raid".into())
    )?;

    let raid = db::get_active_raid(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    let raid = raid.ok_or_else(||
        AppError::NotFound("No active raid found, Cannot transition a raid".into())
    )?;

    let transitioned_at = req.transitioned_at
        .map(|s| time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339))
        .transpose() // turns Option<Result> into Result<Option>
        .map_err(|e| AppError::BadRequest(format!("Invalid timestamp: {}", e)) )?;

    let _transition_id = db::log_state_transition(
        &state.pool, 
        raid.raid_id, 
        &req.to_state,
        transitioned_at,
    ).await.map_err(AppError::DatabaseError)?;

    Ok(StatusCode::OK)

}

pub async fn end_raid(
    State(state): State<AppState>,
    Json(req): Json<EndRaidRequest>,
) -> Result<StatusCode, AppError> {
    let session = db::get_active_session(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    let _session = session.ok_or_else(||
        AppError::NotFound("No active session found, Cannot end a raid".into())
    )?;

    let raid = db::get_active_raid(&state.pool)
        .await.map_err(AppError::DatabaseError)?;

    let raid = raid.ok_or_else(||
        AppError::NotFound("No Active raid".into())
    )?;

    let transitioned_at = req.ended_at
        .map(|s| time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339))
        .transpose() // turns Option<Result> into Result<Option>
        .map_err(|e| AppError::BadRequest(format!("Invalid timestamp: {}", e)) )?;

    let _transition_id = db::log_state_transition(
        &state.pool, 
        raid.raid_id, 
        &req.final_state,
        transitioned_at,
    ).await.map_err(AppError::DatabaseError)?;

    db::end_raid(&state.pool, 
        raid.raid_id,
        transitioned_at,
        req.extract_location).await.map_err(AppError::DatabaseError)?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use http::StatusCode;
    use tower::ServiceExt;
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

        let response = app.clone()
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

        let response = app.clone()
            .oneshot(
                Request::post("/api/raid/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to_state": "in_raid"}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = app
            .oneshot(
                Request::post("/api/raid/end")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "final_state": "survived"
                    }"#)).unwrap(),
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

    #[tokio::test]
    async fn test_get_current_raid_success() {
        let pool = setup_test_db().await.expect("setup db");
        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");

        db::create_raid(
            &pool, session_id, "Factory",
            crate::models::CharacterType::PMC,
            crate::models::GameMode::PVP, None
        ).await.expect("raid");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.get("map_name").unwrap(), "Factory");
        assert_eq!(json.get("current_state").unwrap(), "stash_management");
    }

    #[tokio::test]
    async fn test_get_current_raid_not_found() {
        let pool = setup_test_db().await.expect("setup db");

        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");
       
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_current_raid_after_ending() {
        let pool = setup_test_db().await.expect("setup db");

        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None
            , None)
            .await.expect("session");

        let raid_id = db::create_raid(
            &pool, session_id, "Factory",
            crate::models::CharacterType::PMC,
            crate::models::GameMode::PVP, None
        ).await.expect("raid");

        db::end_raid(&pool, raid_id, None, None).await.expect("end raid");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_transition_state_success() {
        let pool = setup_test_db().await.expect("setup db");
        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");

        db::create_raid(
            &pool, session_id, "Factory",
            crate::models::CharacterType::PMC,
            crate::models::GameMode::PVP, None
        ).await.expect("raid");

        let app = api_router().with_state(AppState::new(pool.clone()));

        let response = app
            .oneshot(
                Request::post("/api/raid/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to_state": "in_raid"}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify state changed in DB
        let raid = db::get_active_raid(&pool).await.unwrap().unwrap();
        assert_eq!(raid.current_state, "in_raid");
    }

    #[tokio::test]
    async fn test_transition_state_no_active_raid() {
        let pool = setup_test_db().await.expect("setup db");
        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/raid/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to_state": "in_raid"}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_end_raid_success() {
        let pool = setup_test_db().await.expect("setup db");
        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");

        db::create_raid(
            &pool, session_id, "Factory",
            crate::models::CharacterType::PMC,
            crate::models::GameMode::PVP, None
        ).await.expect("raid");

        let app = api_router().with_state(AppState::new(pool.clone()));

        let response = app
            .oneshot(
                Request::post("/api/raid/end")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "final_state": "survived",
                        "extract_location": "Gate 3"
                    }"#)).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify No Active Raid
        let raid = db::get_active_raid(&pool).await.unwrap();
        assert!(raid.is_none());
    }

    #[tokio::test]
    async fn test_end_raid_no_active_raid() {
        let pool = setup_test_db().await.expect("setup db");
        let session_id = db::create_session(
            &pool, crate::models::SessionType::Stream, None, None)
            .await.expect("session");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/raid/end")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "final_state": "survived"
                    }"#)).unwrap(),
            ).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_full_raid_flow() {
        let pool = setup_test_db().await.expect("setup db");
        db::create_session(&pool, SessionType::Stream, None, None)
            .await.expect("session");

        let app = api_router().with_state(AppState::new(pool.clone()));

        // 1. Start raid → 201
        let response = app.clone()
            .oneshot(
                Request::post("/api/raid")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "map_name": "Factory",
                        "character_type": "pmc",
                        "game_mode": "pvp"}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // 2. Verify initial state
        let response = app.clone()
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty()).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["current_state"], "stash_management");

        // 3. Transition → in_raid
        let response = app.clone()
            .oneshot(
                Request::post("/api/raid/transition")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"to_state": "in_raid"}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 4. Verify state changed
        let response = app.clone()
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty()).unwrap(),
            ).await.unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["current_state"], "in_raid");

        // 5. End raid
        let response = app.clone()
            .oneshot(
                Request::post("/api/raid/end")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "final_state": "survived",
                        "extract_location": "Gate 3"
                    }"#)).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 6. Verify no active raid
        let response = app
            .oneshot(
                Request::get("/api/raid/current")
                    .body(Body::empty()).unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
