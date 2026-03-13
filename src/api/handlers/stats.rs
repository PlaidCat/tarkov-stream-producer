use axum::{extract::{Path, State}, Json};
use http::StatusCode;
use crate::api::{dto::*, error::AppError, state::AppState};
use crate::db;
use crate::stats::{calculate_session_stats, calculate_time_in_state};

pub async fn get_current_session_stats(
    State(state): State<AppState>,
) -> Result<Json<SessionStatsResponse>, AppError> {
    let session = db::get_active_session(&state.pool)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("No session active".into()))?;

    let stats = calculate_session_stats(&state.pool, session.session_id)
        .await.map_err(AppError::DatabaseError)?;

    Ok(Json(SessionStatsResponse {
        total_raids: stats.total_raids,
        survived_raids: stats.survived_raids,
        survival_rate: stats.survival_rate,
        total_kills: stats.total_kills,
        kd_ratio: stats.kd_ratio,
        avg_raid_duration_seconds: stats.avg_raid_duration.as_seconds_f64(),
    }))
}

pub async fn get_raid_stats(
    State(state): State<AppState>,
    Path(raid_id): Path<i64>,
) -> Result<Json<RaidStatsResponse>, AppError> {
    //verify raid exists
    let _raid = db::get_raid_by_id(&state.pool, raid_id)
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Not a raid".into()))?;
    
    let state_times = calculate_time_in_state(&state.pool, raid_id)
        .await
        .map_err(AppError::DatabaseError)?;

    let state_durations = state_times
        .into_iter()
        .map(|st| StateTimeResponse {
            state: st.state,
            duration_seconds: st.duration.as_seconds_f64(),
        })
        .collect();

    Ok(Json(RaidStatsResponse {
        raid_id,
        state_durations,
    }))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{routes::api_router, state::AppState}; 
    use crate::db::tests::setup_test_db;
    use crate::db::log_state_transition;
    use crate::models::{CharacterType, GameMode, SessionType};
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn test_get_current_session_stats_no_session() {
        let pool = setup_test_db().await.expect("setup db");
       
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/stats/session/current")
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_current_session_stats_success() {
        let pool = setup_test_db().await.expect("setup db");
        let base_time = OffsetDateTime::now_utc();

        // Setup Session + raid + kill
        let session_id = crate::db::create_session(&pool, SessionType::Stream, None, Some(base_time)).await.unwrap();
        let raid_id = crate::db::create_raid(
            &pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, Some(base_time)).await.unwrap();
        crate::db::add_kill(&pool, raid_id, "scav", None, None, None, Some(base_time)).await.unwrap();
        crate::db::log_state_transition(
            &pool, raid_id, "survived", Some(base_time + time::Duration::minutes(10))).await.unwrap();
        crate::db::end_raid(&pool, raid_id, Some(base_time + time::Duration::minutes(10)), None).await.unwrap();


        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/stats/session/current")
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: SessionStatsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.total_raids, 1);
        assert_eq!(json.total_kills, 1);
        assert_eq!(json.survival_rate, 1.0);
    }

    #[tokio::test]
    async fn test_get_raid_stats_success() {
        let pool = setup_test_db().await.expect("setup db");
        let base_time = OffsetDateTime::now_utc();

        // Setup: session + raid + transitions
        let session_id = crate::db::create_session(&pool, SessionType::Stream, None, Some(base_time)).await.unwrap();
        let raid_id = crate::db::create_raid(
            &pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, Some(base_time)).await.unwrap();

        crate::db::log_state_transition(&pool, raid_id, "pre_raid_setup", Some(base_time)).await.unwrap();
        crate::db::log_state_transition(
            &pool, raid_id, "queuing", Some(base_time + time::Duration::minutes(2))).await.unwrap();
        crate::db::log_state_transition(
            &pool, raid_id, "raid_active", Some(base_time + time::Duration::minutes(5))).await.unwrap();
        crate::db::log_state_transition(
            &pool, raid_id, "survived", Some(base_time + time::Duration::minutes(25))).await.unwrap();
        crate::db::end_raid(&pool, raid_id, Some(base_time + time::Duration::minutes(25)), None).await.unwrap();

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get(format!("/api/stats/raid/{}", raid_id))
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: RaidStatsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.raid_id, raid_id);
        // pre_raid_setup: 2 min, queuing: 3 min, raid_active: 20 min
        assert_eq!(json.state_durations.len(), 3);

        let active_duration = json.state_durations.iter().find(|s| s.state == "raid_active").unwrap();
        assert_eq!(active_duration.duration_seconds, 1200.0); // 20 min
    }

    #[tokio::test]
    async fn test_get_raid_stats_not_found() {
        let pool = setup_test_db().await.expect("setup db");
        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/stats/raid/999")
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn test_get_current_session_stats_no_raids() {
        let pool = setup_test_db().await.expect("setup db");
        let base_time = OffsetDateTime::now_utc();

        // Create session but NO raids
        crate::db::create_session(&pool, SessionType::Stream, None, Some(base_time)).await.unwrap();

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get("/api/stats/session/current")
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: SessionStatsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.total_raids, 0);
        assert_eq!(json.survival_rate, 0.0);
        assert_eq!(json.kd_ratio, 0.0);
    }

    #[tokio::test]
    async fn test_get_raid_stats_no_transitions() {
        let pool = setup_test_db().await.expect("setup db");
        let base_time = OffsetDateTime::now_utc();

        let session_id = crate::db::create_session(&pool, SessionType::Stream, None, Some(base_time)).await.unwrap();
        let raid_id = crate::db::create_raid(&pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, Some(base_time)).await.unwrap();

        let app = api_router().with_state(AppState::new(pool));

        let response = app
            .oneshot(
                Request::get(format!("/api/stats/raid/{}", raid_id))
                    .body(Body::empty())
                    .unwrap()
            ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: RaidStatsResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.state_durations.len(), 0);
    }

    #[tokio::test]
    async fn test_session_stats_time_breakdown() -> Result<(), sqlx::Error> {
        let pool = setup_test_db().await?;
        let base_time = OffsetDateTime::now_utc();

        let session_id = crate::db::create_session(&pool, SessionType::Stream, None, Some(base_time)).await?;
        let raid_id = crate::db::create_raid(&pool, session_id, "Customs", CharacterType::PMC, GameMode::PVP, Some(base_time)).await?;

        // Full flow: pre_raid_setup(2m) → queuing(3m) → deploying_committed(1m) → raid_active(20m) → survived
        log_state_transition(&pool, raid_id, "pre_raid_setup", Some(base_time)).await?;
        log_state_transition(&pool, raid_id, "queuing", Some(base_time + time::Duration::minutes(2))).await?;
        log_state_transition(&pool, raid_id, "deploying_committed", Some(base_time + time::Duration::minutes(5))).await?;
        log_state_transition(&pool, raid_id, "raid_active", Some(base_time + time::Duration::minutes(6))).await?;
        log_state_transition(&pool, raid_id, "survived", Some(base_time + time::Duration::minutes(26))).await?;
        crate::db::end_raid(&pool, raid_id, Some(base_time + time::Duration::minutes(26)), None).await?;

        let stats = calculate_session_stats(&pool, session_id).await?;

        assert_eq!(stats.time_breakdown.total_raid_time, time::Duration::minutes(20));
        assert_eq!(stats.time_breakdown.total_queue_time, time::Duration::minutes(3));
        assert_eq!(stats.time_breakdown.avg_queue_time, time::Duration::minutes(3));

        Ok(())
    }
}
