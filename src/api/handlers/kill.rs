use axum::{extract::{Path, State}, Json};
use http::StatusCode;
use crate::api::{dto::*, error::AppError, state::AppState};
use crate::db;

pub async fn add_kill(
    State(state): State<AppState>,
    Path(raid_id): Path<i64>,
    Json(req): Json<AddKillRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Verify the raid exists
    db::get_raid_by_id(&state.pool, raid_id)
        .await.map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound(format!("Raid {} not found", raid_id)))?;

    let killed_at = req.killed_at
        .map(|s| time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339))
        .transpose()
        .map_err(|e| AppError::BadRequest(format!("Invalid timestamp: {}", e)))?;

    let kill_id = db::add_kill(
        &state.pool,
        raid_id,
        &req.enemy_type,
        req.weapon_used,
        req.headshot,
        req.distance_meters,
        killed_at,
    ).await.map_err(AppError::DatabaseError)?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"kill_id": kill_id}))))
}

pub async fn get_kills(
    State(state): State<AppState>,
    Path(raid_id): Path<i64>,
) -> Result<Json<Vec<KillResponse>>, AppError> {
    db::get_raid_by_id(&state.pool, raid_id)
        .await.map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound(format!("Raid {} not found", raid_id)))?;

    let kills = db::get_kills_for_raid(&state.pool, raid_id)
        .await.map_err(AppError::DatabaseError)?;

    Ok(Json(kills.into_iter().map(KillResponse::from).collect()))
}

pub async fn add_batch_kills(
    State(state): State<AppState>,
    Json(req): Json<BatchKillsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let raid = db::get_active_raid(&state.pool)
        .await.map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("No active raid".into()))?;

    let mut kill_ids: Vec<i64> = Vec::new();
    for kill in req.kills{
        let killed_at = kill.killed_at
            .map(|s| time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339))
            .transpose()
            .map_err(|e| AppError::BadRequest(format!("Invalid timestamp: {}", e)))?;

        let kill_id = db::add_kill(
            &state.pool,
            raid.raid_id,
            &kill.enemy_type,
            kill.weapon_used,
            kill.headshot, 
            kill.distance_meters, 
            killed_at
        ).await.map_err(AppError::DatabaseError)?;

        kill_ids.push(kill_id);
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({"kill_ids": kill_ids}))))
}

pub async fn add_vibe_kills(
    State(state): State<AppState>,
    Path(raid_id): Path<i64>,
    Json(req): Json<VibeKillRequest>,
) -> Result<StatusCode, AppError> {
    let raid = db::get_raid_by_id(&state.pool, raid_id)
        .await.map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound(format!("Raid {} not found", raid_id)))?;

    for line in req.kills_text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Format: +MM:SS | type | headshot: true
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() < 2 { continue; }

        // Parse Time (+MM:SS or +HH:MM:SS)
        let time_str = parts[0].trim_start_matches('+');
        let time_parts: Vec<&str> = time_str.split(':').collect();
        let offset = match time_parts.len() {
            2 => {
                let minutes: i64 = time_parts[0].parse().unwrap_or(0);
                let seconds: i64 = time_parts[1].parse().unwrap_or(0);
                time::Duration::minutes(minutes) + time::Duration::seconds(seconds)
            }
            3 => {
                let hours: i64 = time_parts[0].parse().unwrap_or(0);
                let minutes: i64 = time_parts[1].parse().unwrap_or(0);
                let seconds: i64 = time_parts[2].parse().unwrap_or(0);
                time::Duration::hours(hours) + time::Duration::minutes(minutes) + time::Duration::seconds(seconds)
            }
            _ => continue,
        };

        // Calculate absolute time
        let killed_at = raid.started_at + offset;

        // 2. Parse Type
        let enemy_type = parts[1].to_lowercase();

        // 3. Parse Headshot (optional third part)
        let headshot = if parts.len() > 2 {
            Some(parts[2].to_lowercase().contains("true"))
        } else {
            None
        };

        db::add_kill(
            &state.pool,
            raid_id,
            &enemy_type,
            None, // weapon_used
            headshot,
            None, // distance
            Some(killed_at),
        ).await.map_err(AppError::DatabaseError)?;
    }

    Ok(StatusCode::CREATED)
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use http::StatusCode;
    use tower::ServiceExt;
    use crate::api::{state::AppState, routes::api_router};
    use crate::db::{self, tests::setup_test_db};
    use crate::models::{SessionType, CharacterType, GameMode};

    async fn setup_raid(pool: &sqlx::SqlitePool) -> i64 {
        let session_id = db::create_session(pool, SessionType::Stream, None, None)
            .await.expect("Session");
        db::create_raid(pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, None)
            .await.expect("raid")
    }

    #[tokio::test]
    async fn test_add_kill_sucess() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post(format!("/api/raid/{}/kills", raid_id))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"enemy_type": "scav", "headshot": true, "distance_meters": 42.5}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_add_kill_invalid_riad_id() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/api/raid/999999/kills")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"enemy_type": "scav", "headshot": true, "distance_meters": 42.5}"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_kills_for_raid_success() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;

        db::add_kill(&pool, raid_id, "scav", Some("AK74".into()), Some(true), Some(35.2), None)
            .await.expect("kill");

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get(format!("/api/raid/{}/kills", raid_id))
                    .body(Body::empty())
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["enemy_type"], "scav");
        assert_eq!(json[0]["distance_meters"], 35.2);
    }

    #[tokio::test]
    async fn test_add_batch_kills_success() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;

        let app = api_router().with_state(AppState::new(pool.clone()));

        let response = app
            .oneshot(
                Request::post("/api/raid/current/kills/batch")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{
                        "kills": [
                            {"enemy_type": "scav", "headshot": true, "distance_meters": 12.0},
                            {"enemy_type": "pmc", "weapon_used": "M4A1", "distance_meters": 58.3}
                        ]
                    }"#))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        //verify both kills were stored
        let kills = db::get_kills_for_raid(&pool, raid_id).await.expect("kills");
        assert_eq!(kills.len(), 2);
    }

    #[tokio::test]
    async fn test_add_batch_kills_no_active_raid() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::post("/app/raid/current/kills/batch")
                    .header("current-type", "application/json")
                    .body(Body::from(r#"{"kills": [{"enemy_type": "scav"}]}"#))
                    .unwrap(),
            ).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_add_vibe_kills_success() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;
        let app = api_router().with_state(AppState::new(pool.clone()));

        let body = serde_json::json!({
            "kills_text": "+05:20 | scav | headshot: true\n+10:00 | pmc"
        }).to_string();

        let response = app
            .oneshot(
                Request::post(format!("/api/raid/{}/kills/vibe", raid_id))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let kills = db::get_kills_for_raid(&pool, raid_id).await.expect("kills");
        assert_eq!(kills.len(), 2);
        assert_eq!(kills[0].enemy_type, "scav");
        assert_eq!(kills[0].headshot, Some(true));
        assert_eq!(kills[1].enemy_type, "pmc");
        assert_eq!(kills[1].headshot, None);
    }

    #[tokio::test]
    async fn test_add_vibe_kills_skips_malformed_lines() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;
        let app = api_router().with_state(AppState::new(pool.clone()));

        let body = serde_json::json!({
            "kills_text": "not valid\n\n+05:20 | scav"
        }).to_string();

        let response = app
            .oneshot(
                Request::post(format!("/api/raid/{}/kills/vibe", raid_id))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let kills = db::get_kills_for_raid(&pool, raid_id).await.expect("kills");
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0].enemy_type, "scav");
    }

    #[tokio::test]
    async fn test_add_vibe_kills_not_found() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let body = serde_json::json!({"kills_text": "+05:20 | scav"}).to_string();

        let response = app
            .oneshot(
                Request::post("/api/raid/999/kills/vibe")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_add_vibe_kills_hms_format() {
        let pool = setup_test_db().await.expect("setup db");
        let raid_id = setup_raid(&pool).await;
        let app = api_router().with_state(AppState::new(pool.clone()));

        let body = serde_json::json!({
            "kills_text": "+01:05:20 | scav | headshot: true"
        }).to_string();

        let response = app
            .oneshot(
                Request::post(format!("/api/raid/{}/kills/vibe", raid_id))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let kills = db::get_kills_for_raid(&pool, raid_id).await.expect("kills");
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0].enemy_type, "scav");
        assert_eq!(kills[0].headshot, Some(true));
    }
}
